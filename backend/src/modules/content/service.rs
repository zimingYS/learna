//! 学习内容本体的持久化：CRUD、树形约束、相似提示、归档与彻底删除。

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::modules::content::dto::{
    ContentCreateData, ContentDetailItem, ContentDetailRow, ContentHint, ContentItem,
    ContentListData, ContentQuery, ContentRow, ContentStatusFilter, CreateContentRequest,
    UpdateContentRequest,
};
use crate::modules::content::rules;
use crate::shared::{AppError, AppResult, ContentStatus, ContentType};

const LIST_COLUMNS: &str = r#"
    id, content_type, title, description, parent_id, body_format, source_url,
    created_source, status, created_at, updated_at
"#;

const DETAIL_COLUMNS: &str = r#"
    id, content_type, title, description, parent_id, body, body_format, source_url,
    created_source, status, created_at, updated_at
"#;

/// 内容列表：默认只看 `active`，排序固定 `updated_at DESC, id DESC`
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    query: ContentQuery,
) -> AppResult<ContentListData> {
    if query.parent_id.is_some() && query.root_only == Some(true) {
        return Err(AppError::Validation(
            "parent_id 与 root_only 不能同时使用".to_string(),
        ));
    }

    let (limit, offset) = query.window();
    let status = match query.status.unwrap_or(ContentStatusFilter::Active) {
        ContentStatusFilter::Active => ContentStatus::Active.as_str(),
        ContentStatusFilter::Archived => ContentStatus::Archived.as_str(),
        ContentStatusFilter::All => "all",
    };
    let pattern = query.q.as_deref().map(rules::escape_like);
    let root_only = query.root_only.unwrap_or(false);

    let sql = format!(
        r#"
        SELECT {LIST_COLUMNS}
        FROM learning_contents
        WHERE owner_id = $1
          AND ($2::text IS NULL OR content_type = $2)
          AND ($3::text = 'all' OR status = $3)
          AND ($4::uuid IS NULL OR parent_id = $4)
          AND (NOT $5::bool OR parent_id IS NULL)
          AND ($6::text IS NULL OR title ILIKE '%' || $6 || '%' ESCAPE '\')
        ORDER BY updated_at DESC, id DESC
        LIMIT $7 OFFSET $8
        "#
    );

    let rows = sqlx::query_as::<_, ContentRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(query.content_type.map(|value| value.as_str().to_string()))
        .bind(status)
        .bind(query.parent_id)
        .bind(root_only)
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let count_sql = "
        SELECT COUNT(*)
        FROM learning_contents
        WHERE owner_id = $1
          AND ($2::text IS NULL OR content_type = $2)
          AND ($3::text = 'all' OR status = $3)
          AND ($4::uuid IS NULL OR parent_id = $4)
          AND (NOT $5::bool OR parent_id IS NULL)
          AND ($6::text IS NULL OR title ILIKE '%' || $6 || '%' ESCAPE '\\')
    ";

    let total = sqlx::query_scalar::<_, i64>(count_sql)
        .bind(owner_id)
        .bind(query.content_type.map(|value| value.as_str().to_string()))
        .bind(status)
        .bind(query.parent_id)
        .bind(root_only)
        .bind(query.q.as_deref().map(rules::escape_like))
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(ContentItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(ContentListData { items, total })
}

/// 创建内容。
///
/// 相似提示**先查后插**：同类型、`similarity > 0.4`、取前 5（D-24）。
/// 提示不阻断创建——ERD §3.1 明确允许重复，只负责让用户知情。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateContentRequest,
) -> AppResult<ContentCreateData> {
    rules::reject_write_protected("metadata", payload.metadata.is_some())?;

    let title = rules::ensure_title("title", &payload.title)?;
    let description = rules::normalize_description(payload.description)?;
    let body = rules::normalize_body(payload.body)?;
    let body_format = rules::resolve_body_format(payload.body_format);
    let source_url = rules::normalize_source_url(payload.source_url)?;

    if let Some(parent_id) = payload.parent_id {
        ensure_parent_exists(pool, owner_id, parent_id).await?;
        let depth = parent_depth(pool, parent_id).await? + 1;
        rules::ensure_depth_within_limit(depth)?;
    }

    let hints = similar_hints(pool, owner_id, payload.content_type, &title).await?;

    let sql = format!(
        r#"
        INSERT INTO learning_contents (
            owner_id, content_type, title, description, parent_id,
            body, body_format, source_url, created_source, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'user', 'active')
        RETURNING {DETAIL_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, ContentDetailRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(payload.content_type.as_str())
        .bind(&title)
        .bind(description)
        .bind(payload.parent_id)
        .bind(body)
        .bind(body_format.as_str())
        .bind(source_url)
        .fetch_one(pool)
        .await?;

    let depth = node_depth(pool, row.id).await?;
    Ok(ContentCreateData {
        item: ContentDetailItem::new(row, depth)?,
        similar_hints: hints,
    })
}

/// 相似内容提示；`pg_trgm` 的 `similarity()` 对中文按三元组切分，短标题也能给出信号
async fn similar_hints(
    pool: &PgPool,
    owner_id: Uuid,
    content_type: ContentType,
    title: &str,
) -> AppResult<Vec<ContentHint>> {
    let sql = r#"
        SELECT t.id, t.title, t.similarity
        FROM (
            SELECT id, title, similarity(title, $3) AS similarity
            FROM learning_contents
            WHERE owner_id = $1
              AND content_type = $2
              AND status = 'active'
              AND similarity(title, $3) > 0.4
        ) t
        ORDER BY t.similarity DESC, t.id
        LIMIT 5
    "#;

    Ok(sqlx::query_as::<_, ContentHint>(sql)
        .bind(owner_id)
        .bind(content_type.as_str())
        .bind(title)
        .fetch_all(pool)
        .await?)
}

/// 内容详情：唯一返回 `body` 的读取路径，同时算出树深度
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
) -> AppResult<ContentDetailItem> {
    let row = fetch_detail_row(pool, owner_id, content_id).await?;
    let depth = node_depth(pool, content_id).await?;
    ContentDetailItem::new(row, depth)
}

/// 取详情行（含正文），归属校验写进 `WHERE`
async fn fetch_detail_row(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
) -> AppResult<ContentDetailRow> {
    let sql = format!(
        r#"
        SELECT {DETAIL_COLUMNS}
        FROM learning_contents
        WHERE id = $1 AND owner_id = $2
        "#
    );

    sqlx::query_as::<_, ContentDetailRow>(sqlx::AssertSqlSafe(sql))
        .bind(content_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("内容不存在".to_string()))
}

/// 部分更新（ADR-002 三态）。
///
/// 改 `parent_id` 走单事务：同归属、不成环、整棵子树深度 ≤ 10 三项校验与写入之间
/// 不允许插入并发写。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
    payload: UpdateContentRequest,
) -> AppResult<ContentDetailItem> {
    rules::reject_write_protected("metadata", payload.metadata.is_some())?;
    rules::reject_write_protected("status", payload.status.is_some())?;
    rules::reject_write_protected("created_source", payload.created_source.is_some())?;

    let current = fetch_detail_row(pool, owner_id, content_id).await?;

    let mut tx = pool.begin().await?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE learning_contents SET ");
    let mut separated = builder.separated(", ");
    let mut touched = false;

    if let Some(title) = payload.title {
        let title = rules::ensure_title("title", &title)?;
        separated.push("title = ").push_bind_unseparated(title);
        touched = true;
    }

    if let Some(description) = payload.description {
        let description = rules::normalize_description(description)?;
        separated
            .push("description = ")
            .push_bind_unseparated(description);
        touched = true;
    }

    if let Some(body) = payload.body {
        let body = rules::normalize_body(body)?;
        separated.push("body = ").push_bind_unseparated(body);
        touched = true;
    }

    if let Some(body_format) = payload.body_format {
        separated
            .push("body_format = ")
            .push_bind_unseparated(body_format.as_str().to_string());
        touched = true;
    }

    if let Some(source_url) = payload.source_url {
        let source_url = rules::normalize_source_url(source_url)?;
        separated
            .push("source_url = ")
            .push_bind_unseparated(source_url);
        touched = true;
    }

    if let Some(content_type) = payload.content_type {
        separated
            .push("content_type = ")
            .push_bind_unseparated(content_type.as_str().to_string());
        touched = true;
    }

    if let Some(parent) = payload.parent_id {
        // 与当前值相同视为未变，避免无谓的环检测查询
        if parent != current.parent_id {
            if let Some(parent_id) = parent {
                ensure_parent_attachable(&mut tx, owner_id, content_id, parent_id).await?;
            }
            separated.push("parent_id = ").push_bind_unseparated(parent);
            touched = true;
        }
    }

    if !touched {
        return detail(pool, owner_id, content_id).await;
    }

    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(content_id)
        .push(" AND owner_id = ")
        .push_bind(owner_id)
        .push(format!(" RETURNING {DETAIL_COLUMNS}"));

    let row = builder
        .build_query_as::<ContentDetailRow>()
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("内容不存在".to_string()))?;

    tx.commit().await?;

    let depth = node_depth(pool, content_id).await?;
    ContentDetailItem::new(row, depth)
}

/// 删除：默认归档（幂等），`purge=true` 物理删除（D-26）。
///
/// 归档**不触碰** `project_contents` 与 `content_relations`——已关联的项目上下文保留，
/// 只是不能再新建关联。
pub async fn delete(pool: &PgPool, owner_id: Uuid, content_id: Uuid, purge: bool) -> AppResult<()> {
    if !purge {
        sqlx::query(
            r#"
            UPDATE learning_contents
            SET status = 'archived', updated_at = now()
            WHERE id = $1 AND owner_id = $2 AND status <> 'archived'
            "#,
        )
        .bind(content_id)
        .bind(owner_id)
        .execute(pool)
        .await?;

        // 已归档时上面的 UPDATE 命中 0 行，仍要确认内容存在（否则越权被当成幂等成功）
        fetch_detail_row(pool, owner_id, content_id).await?;
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM learning_contents WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(content_id)
    .bind(owner_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("内容不存在".to_string()))?;

    if status != ContentStatus::Archived.as_str() {
        return Err(AppError::Conflict(
            "彻底删除前必须先归档内容（DELETE 不带 purge 参数）".to_string(),
        ));
    }

    let project_refs =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM project_contents WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&mut *tx)
            .await?;

    let children =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM learning_contents WHERE parent_id = $1")
            .bind(content_id)
            .fetch_one(&mut *tx)
            .await?;

    if project_refs > 0 || children > 0 {
        return Err(AppError::DeletionConflict {
            impact: serde_json::json!({
                "project_contents": project_refs,
                "children": children,
            }),
            hint: "内容仍被项目引用或存在子内容；请先从项目解除关联并上移子内容".to_string(),
        });
    }

    // content_relations 两端与 learning_content_tags 由外键 ON DELETE CASCADE 清理
    sqlx::query("DELETE FROM learning_contents WHERE id = $1 AND owner_id = $2")
        .bind(content_id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 父节点必须存在且属于当前用户；越权与不存在同为 404
///
/// 接受任意 sqlx executor：创建路径用连接池，改父路径在事务内复用同一实现。
async fn ensure_parent_exists<'c, E>(executor: E, owner_id: Uuid, parent_id: Uuid) -> AppResult<()>
where
    E: sqlx::Executor<'c, Database = Postgres>,
{
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_contents WHERE id = $1 AND owner_id = $2",
    )
    .bind(parent_id)
    .bind(owner_id)
    .fetch_optional(executor)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::NotFound("父内容不存在".to_string()));
    }
    Ok(())
}

/// 改父前的三项校验：同归属、不成环、整棵子树深度 ≤ 10
async fn ensure_parent_attachable(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    owner_id: Uuid,
    content_id: Uuid,
    parent_id: Uuid,
) -> AppResult<()> {
    if parent_id == content_id {
        return Err(AppError::Validation("内容不能以自身为父节点".to_string()));
    }

    ensure_parent_exists(&mut **tx, owner_id, parent_id).await?;
    // 父节点必须是自身的祖先链之外：链上出现自己的 id 即成环
    let chain_sql = r#"
        WITH RECURSIVE chain AS (
            SELECT id, parent_id, 0 AS depth FROM learning_contents WHERE id = $1
            UNION ALL
            SELECT c.id, c.parent_id, chain.depth + 1
            FROM learning_contents c
            JOIN chain ON c.id = chain.parent_id
            WHERE chain.depth < $3
        )
        SELECT COUNT(*) FILTER (WHERE id = $2) AS hits, COALESCE(MAX(depth), 0) AS depth
        FROM chain
    "#;

    let (hits, parent_depth) = sqlx::query_as::<_, (i64, i32)>(chain_sql)
        .bind(parent_id)
        .bind(content_id)
        .bind(rules::MAX_TREE_DEPTH)
        .fetch_one(&mut **tx)
        .await?;

    if hits > 0 {
        return Err(AppError::Validation("parent_id 会形成内容树环".to_string()));
    }

    // 子树整体下移，最深的那个后代才是约束点
    let height_sql = r#"
        WITH RECURSIVE subtree AS (
            SELECT id, 0 AS height FROM learning_contents WHERE id = $1
            UNION ALL
            SELECT c.id, s.height + 1
            FROM learning_contents c
            JOIN subtree s ON c.parent_id = s.id
            WHERE s.height < $2
        )
        SELECT COALESCE(MAX(height), 0) FROM subtree
    "#;

    let height = sqlx::query_scalar::<_, i32>(height_sql)
        .bind(content_id)
        .bind(rules::MAX_TREE_DEPTH)
        .fetch_one(&mut **tx)
        .await?;

    rules::ensure_depth_within_limit(parent_depth + 1 + height)
}

/// 节点到根的距离（根为 0）
async fn node_depth(pool: &PgPool, content_id: Uuid) -> AppResult<i32> {
    let sql = r#"
        WITH RECURSIVE chain AS (
            SELECT id, parent_id, 0 AS depth FROM learning_contents WHERE id = $1
            UNION ALL
            SELECT c.id, c.parent_id, chain.depth + 1
            FROM learning_contents c
            JOIN chain ON c.id = chain.parent_id
            WHERE chain.depth < $2
        )
        SELECT COALESCE(MAX(depth), 0) FROM chain
    "#;

    Ok(sqlx::query_scalar::<_, i32>(sql)
        .bind(content_id)
        .bind(rules::MAX_TREE_DEPTH)
        .fetch_one(pool)
        .await?)
}

/// 父节点的深度；用于创建时估算新节点层级
async fn parent_depth(pool: &PgPool, parent_id: Uuid) -> AppResult<i32> {
    node_depth(pool, parent_id).await
}
