//! 标签词表与内容打标。
//!
//! 标签是用户私有词表（`UNIQUE(owner_id, name)`），内容侧用整体替换（D-28）：
//! 前端一次提交多选结果，不必维护差集。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::content::dto::{
    ContentTagItem, ContentTagListData, ContentTagQuery, ContentTagRow, CreateContentTagRequest,
    SetContentTagsRequest, UpdateContentTagRequest,
};
use crate::modules::content::rules;
use crate::shared::{AppError, AppResult};

const TAG_COLUMNS: &str = "id, name, color, created_at";

/// 标签词表列表：排序固定 `name ASC`（词表顺序要稳定）
pub async fn list_tags(
    pool: &PgPool,
    owner_id: Uuid,
    query: ContentTagQuery,
) -> AppResult<ContentTagListData> {
    let (limit, offset) = query.window();
    let pattern = query.q.as_deref().map(rules::escape_like);

    let sql = format!(
        r#"
        SELECT {TAG_COLUMNS}
        FROM content_tags
        WHERE owner_id = $1
          AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' ESCAPE '\')
        ORDER BY name ASC, id
        LIMIT $3 OFFSET $4
        "#
    );

    let rows = sqlx::query_as::<_, ContentTagRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(pattern.clone())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM content_tags
        WHERE owner_id = $1
          AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' ESCAPE '\')
        "#,
    )
    .bind(owner_id)
    .bind(pattern)
    .fetch_one(pool)
    .await?;

    Ok(ContentTagListData {
        items: rows.into_iter().map(ContentTagItem::from).collect(),
        total,
    })
}

/// 创建标签；重名 409，不静默返回既有标签（避免掩盖同名不同义）
pub async fn create_tag(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateContentTagRequest,
) -> AppResult<ContentTagItem> {
    let name = rules::ensure_tag_name(&payload.name)?;
    let color = rules::normalize_tag_color(payload.color)?;

    let sql = format!(
        r#"
        INSERT INTO content_tags (owner_id, name, color)
        VALUES ($1, $2, $3)
        ON CONFLICT (owner_id, name) DO NOTHING
        RETURNING {TAG_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, ContentTagRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(name)
        .bind(color)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Conflict("同名标签已存在".to_string()))?;

    Ok(row.into())
}

/// 更新标签：改名撞已有名 → 409
pub async fn update_tag(
    pool: &PgPool,
    owner_id: Uuid,
    tag_id: Uuid,
    payload: UpdateContentTagRequest,
) -> AppResult<ContentTagItem> {
    let mut name: Option<String> = None;
    if let Some(value) = payload.name {
        name = Some(rules::ensure_tag_name(&value)?);
    }

    let color_touched = payload.color.is_some();
    let color = match payload.color {
        None => None,
        Some(value) => rules::normalize_tag_color(value)?,
    };

    // 改名时先查重：并发下仍可能撞 UNIQUE，靠 23505 兜住
    if let Some(next_name) = &name {
        let taken = sqlx::query_scalar::<_, i32>(
            "SELECT 1 FROM content_tags WHERE owner_id = $1 AND name = $2 AND id <> $3",
        )
        .bind(owner_id)
        .bind(next_name)
        .bind(tag_id)
        .fetch_optional(pool)
        .await?
        .is_some();

        if taken {
            return Err(AppError::Conflict("同名标签已存在".to_string()));
        }
    }

    let sql = format!(
        r#"
        UPDATE content_tags
        SET name = COALESCE($3, name),
            color = CASE WHEN $4::bool THEN $5 ELSE color END
        WHERE id = $1 AND owner_id = $2
        RETURNING {TAG_COLUMNS}
        "#
    );

    let color_value = color;

    let result = sqlx::query_as::<_, ContentTagRow>(sqlx::AssertSqlSafe(sql))
        .bind(tag_id)
        .bind(owner_id)
        .bind(name)
        .bind(color_touched)
        .bind(color_value)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(row)) => Ok(row.into()),
        Ok(None) => Err(AppError::NotFound("标签不存在".to_string())),
        Err(err) if is_unique_violation(&err) => {
            Err(AppError::Conflict("同名标签已存在".to_string()))
        }
        Err(err) => Err(err.into()),
    }
}

/// 删除标签：`learning_content_tags` 关联行随外键级联清理
pub async fn delete_tag(pool: &PgPool, owner_id: Uuid, tag_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query("DELETE FROM content_tags WHERE id = $1 AND owner_id = $2")
        .bind(tag_id)
        .bind(owner_id)
        .execute(pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("标签不存在".to_string()));
    }
    Ok(())
}

/// 内容已打标签：不分页（单内容上限 20）
pub async fn list_content_tags(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
) -> AppResult<ContentTagListData> {
    ensure_content(pool, owner_id, content_id).await?;

    let sql = r#"
        SELECT t.id, t.name, t.color, t.created_at
        FROM content_tags t
        JOIN learning_content_tags lct ON lct.tag_id = t.id
        WHERE lct.content_id = $1 AND t.owner_id = $2
        ORDER BY t.name ASC, t.id
    "#;

    let rows = sqlx::query_as::<_, ContentTagRow>(sql)
        .bind(content_id)
        .bind(owner_id)
        .fetch_all(pool)
        .await?;

    let items: Vec<ContentTagItem> = rows.into_iter().map(ContentTagItem::from).collect();
    let total = items.len() as i64;
    Ok(ContentTagListData { items, total })
}

/// 整体替换内容标签（幂等；空数组表示清空）
pub async fn set_content_tags(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
    payload: SetContentTagsRequest,
) -> AppResult<ContentTagListData> {
    ensure_content(pool, owner_id, content_id).await?;

    let mut tag_ids = payload.tag_ids;
    let requested = tag_ids.len();

    if requested > rules::MAX_TAGS {
        return Err(AppError::Validation(format!(
            "单个内容的标签数不能超过 {} 个",
            rules::MAX_TAGS
        )));
    }

    tag_ids.sort_unstable();
    tag_ids.dedup();
    if tag_ids.len() != requested {
        return Err(AppError::Validation("tag_ids 中存在重复的标签".to_string()));
    }

    let mut tx = pool.begin().await?;

    if !tag_ids.is_empty() {
        let owned = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM content_tags WHERE id = ANY($1) AND owner_id = $2",
        )
        .bind(&tag_ids)
        .bind(owner_id)
        .fetch_one(&mut *tx)
        .await?;

        if owned != requested as i64 {
            return Err(AppError::NotFound("标签不存在".to_string()));
        }
    }

    sqlx::query("DELETE FROM learning_content_tags WHERE content_id = $1")
        .bind(content_id)
        .execute(&mut *tx)
        .await?;

    for tag_id in &tag_ids {
        sqlx::query("INSERT INTO learning_content_tags (content_id, tag_id) VALUES ($1, $2)")
            .bind(content_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    list_content_tags(pool, owner_id, content_id).await
}

async fn ensure_content(pool: &PgPool, owner_id: Uuid, content_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_contents WHERE id = $1 AND owner_id = $2",
    )
    .bind(content_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::NotFound("内容不存在".to_string()));
    }
    Ok(())
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}
