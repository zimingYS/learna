//! 学习证据持久化：owner 级独立资源，物理删除（ADR-010 §4）。
//!
//! 证据是快照（当时的笔记 / 链接 / 测试结果），不提供 PATCH——改就删了重建。
//! session 软删不解挂证据：证据有独立价值。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::evidence::dto::{
    CreateEvidenceRequest, EvidenceItem, EvidenceListData, EvidenceQuery, EvidenceRow,
};
use crate::modules::evidence::rules;
use crate::shared::{AppError, AppResult};

const COLUMNS: &str = "id, session_id, evidence_type, title, description, external_url, created_at";

/// 证据列表：排序固定 `created_at DESC, id DESC`
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    query: EvidenceQuery,
) -> AppResult<EvidenceListData> {
    let (limit, offset) = query.window();
    let evidence_type = match query.evidence_type.as_deref() {
        Some(value) => Some(rules::parse_evidence_type(value)?.as_str().to_string()),
        None => None,
    };

    let filter = r#"
        FROM learning_evidence
        WHERE owner_id = $1
          AND ($2::uuid IS NULL OR session_id = $2)
          AND ($3::text IS NULL OR evidence_type = $3)
          AND ($4::timestamptz IS NULL OR created_at >= $4)
          AND ($5::timestamptz IS NULL OR created_at < $5)
    "#;

    let sql =
        format!("SELECT {COLUMNS} {filter} ORDER BY created_at DESC, id DESC LIMIT $6 OFFSET $7");

    let rows = sqlx::query_as::<_, EvidenceRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(query.session_id)
        .bind(evidence_type.as_deref())
        .bind(query.from)
        .bind(query.to)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let count_sql = format!("SELECT COUNT(*) {filter}");
    let total = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql))
        .bind(owner_id)
        .bind(query.session_id)
        .bind(evidence_type.as_deref())
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(EvidenceItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(EvidenceListData { items, total })
}

/// 新增证据：v1 无文件上传，单语句即可（ADR-010 §4）
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateEvidenceRequest,
) -> AppResult<EvidenceItem> {
    let evidence_type = rules::parse_evidence_type(&payload.evidence_type)?;
    let title = rules::normalize_title(payload.title)?;
    let description = rules::normalize_text(payload.description);
    let external_url = rules::normalize_url(payload.external_url)?;

    rules::ensure_payload(evidence_type, &description, &external_url)?;

    if let Some(session_id) = payload.session_id {
        ensure_session(pool, owner_id, session_id).await?;
    }

    let sql = format!(
        r#"
        INSERT INTO learning_evidence (
            owner_id, session_id, evidence_type, title, description, external_url
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING {COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, EvidenceRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(payload.session_id)
        .bind(evidence_type.as_str())
        .bind(title)
        .bind(description)
        .bind(external_url)
        .fetch_one(pool)
        .await?;

    EvidenceItem::try_from(row)
}

/// 证据详情
pub async fn detail(pool: &PgPool, owner_id: Uuid, evidence_id: Uuid) -> AppResult<EvidenceItem> {
    let sql = format!("SELECT {COLUMNS} FROM learning_evidence WHERE id = $1 AND owner_id = $2");

    let row = sqlx::query_as::<_, EvidenceRow>(sqlx::AssertSqlSafe(sql))
        .bind(evidence_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("证据不存在".to_string()))?;

    EvidenceItem::try_from(row)
}

/// 物理删除（无下游引用）；物理删后无法区分「删过」与「从未存在」，故重复删返回 404
pub async fn delete(pool: &PgPool, owner_id: Uuid, evidence_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query("DELETE FROM learning_evidence WHERE id = $1 AND owner_id = $2")
        .bind(evidence_id)
        .bind(owner_id)
        .execute(pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("证据不存在".to_string()));
    }
    Ok(())
}

/// 证据只能挂在存在、属于本人且未软删的活动上（软删对客户端表现为不存在）
async fn ensure_session(pool: &PgPool, owner_id: Uuid, session_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_sessions WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL",
    )
    .bind(session_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::NotFound("学习活动不存在".to_string()));
    }
    Ok(())
}
