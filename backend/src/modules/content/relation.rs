//! 内容关系：单向存储、双向查询（D-25），`prerequisite` 成环检测。
//!
//! 存一行而不是两行：对称关系（`related` / `alternative`）的双向性在查询层展开，
//! 否则「删一条关系」要删两行，且两行可能不同步。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::content::dto::{
    CreateRelationRequest, RelationDirectionFilter, RelationItem, RelationListData, RelationQuery,
    RelationRow,
};
use crate::modules::content::rules;
use crate::shared::{AppError, AppResult, RelationType};

/// 环检测的递归深度上限：单内容关系数上限即图规模上限，超过必然已存在环
const RING_SCAN_LIMIT: i32 = 200;

/// 关系列表：以内容为锚点，`direction=all` 时同时返回出边与入边
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
    query: RelationQuery,
) -> AppResult<RelationListData> {
    ensure_content(pool, owner_id, content_id).await?;

    let direction = query.direction.unwrap_or(RelationDirectionFilter::All);
    let sql = r#"
        SELECT id, source_content_id, target_content_id, relation_type, created_at
        FROM content_relations
        WHERE owner_id = $1
          AND ($3::text IS NULL OR relation_type = $3)
          AND (
                ($4 = 'all' AND (source_content_id = $2 OR target_content_id = $2))
             OR ($4 = 'outbound' AND source_content_id = $2)
             OR ($4 = 'inbound' AND target_content_id = $2)
          )
        ORDER BY created_at DESC, id
    "#;

    let direction_value = match direction {
        RelationDirectionFilter::Outbound => "outbound",
        RelationDirectionFilter::Inbound => "inbound",
        RelationDirectionFilter::All => "all",
    };

    let rows = sqlx::query_as::<_, RelationRow>(sql)
        .bind(owner_id)
        .bind(content_id)
        .bind(query.relation_type.map(|value| value.as_str().to_string()))
        .bind(direction_value)
        .fetch_all(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| RelationItem::from_row(row, content_id))
        .collect::<AppResult<Vec<_>>>()?;

    let total = items.len() as i64;
    Ok(RelationListData { items, total })
}

/// 建立关系。
///
/// 单事务：按 id 排序锁两端内容行 → `prerequisite` 查环 → 插入。与 `task_dependencies`
/// 同构，防并发下各自通过检测却共同成环。
/// 重复边幂等返回既有行（`UNIQUE` + `ON CONFLICT DO NOTHING`）。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    source_id: Uuid,
    payload: CreateRelationRequest,
) -> AppResult<RelationItem> {
    let target_id = payload.target_content_id;
    rules::ensure_not_self_relation(source_id, target_id)?;

    let mut tx = pool.begin().await?;

    // 固定加锁顺序（小 id 在前），两个并发反方向请求不会互相等待
    let (first, second) = if source_id < target_id {
        (source_id, target_id)
    } else {
        (target_id, source_id)
    };

    let locked = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM learning_contents WHERE id = ANY($1) AND owner_id = $2 ORDER BY id FOR UPDATE",
    )
    .bind(&[first, second][..])
    .bind(owner_id)
    .fetch_all(&mut *tx)
    .await?;

    if locked.len() != 2 {
        return Err(AppError::NotFound("内容不存在".to_string()));
    }

    let existing = sqlx::query_as::<_, RelationRow>(
        r#"
        SELECT id, source_content_id, target_content_id, relation_type, created_at
        FROM content_relations
        WHERE source_content_id = $1 AND target_content_id = $2 AND relation_type = $3
        "#,
    )
    .bind(source_id)
    .bind(target_id)
    .bind(payload.relation_type.as_str())
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(row) = existing {
        tx.commit().await?;
        return RelationItem::from_row(row, source_id);
    }

    if payload.relation_type == RelationType::Prerequisite {
        ensure_no_cycle(&mut tx, owner_id, source_id, target_id).await?;
    }

    let outgoing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM content_relations WHERE source_content_id = $1 OR target_content_id = $1",
    )
    .bind(source_id)
    .fetch_one(&mut *tx)
    .await?;

    if outgoing >= rules::MAX_RELATIONS as i64 {
        return Err(AppError::Validation(format!(
            "单个内容的关系数不能超过 {} 条",
            rules::MAX_RELATIONS
        )));
    }

    let row = sqlx::query_as::<_, RelationRow>(
        r#"
        INSERT INTO content_relations (owner_id, source_content_id, target_content_id, relation_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, source_content_id, target_content_id, relation_type, created_at
        "#,
    )
    .bind(owner_id)
    .bind(source_id)
    .bind(target_id)
    .bind(payload.relation_type.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    RelationItem::from_row(row, source_id)
}

/// `prerequisite` 环检测：新边 `source → target` 成环，等价于 `target` 已能到达 `source`
async fn ensure_no_cycle(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner_id: Uuid,
    source_id: Uuid,
    target_id: Uuid,
) -> AppResult<()> {
    let sql = r#"
        WITH RECURSIVE reach AS (
            SELECT target_content_id AS id, 0 AS depth
            FROM content_relations
            WHERE source_content_id = $1 AND relation_type = 'prerequisite' AND owner_id = $3
            UNION ALL
            SELECT r.target_content_id, reach.depth + 1
            FROM content_relations r
            JOIN reach ON r.source_content_id = reach.id
            WHERE r.relation_type = 'prerequisite'
              AND r.owner_id = $3
              AND reach.depth < $4
        )
        SELECT COUNT(*) FROM reach WHERE id = $2
    "#;

    let hits = sqlx::query_scalar::<_, i64>(sql)
        .bind(target_id)
        .bind(source_id)
        .bind(owner_id)
        .bind(RING_SCAN_LIMIT)
        .fetch_one(&mut **tx)
        .await?;

    if hits > 0 {
        return Err(AppError::Validation(
            "prerequisite 关系会形成环".to_string(),
        ));
    }
    Ok(())
}

/// 删除关系：关系必须属于当前用户的该内容（任一端）
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    content_id: Uuid,
    relation_id: Uuid,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        DELETE FROM content_relations
        WHERE id = $1
          AND owner_id = $2
          AND (source_content_id = $3 OR target_content_id = $3)
        "#,
    )
    .bind(relation_id)
    .bind(owner_id)
    .bind(content_id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("关系不存在".to_string()));
    }
    Ok(())
}

/// 锚点内容必须属于当前用户
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
