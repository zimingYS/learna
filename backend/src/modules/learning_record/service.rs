use crate::modules::learning_record::dto::{
    CreateRecordRequest, RecordListResponse, RecordResponse, UpdateRecordRequest,
};
use crate::shared::{AppError, AppResult};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct RecordRow {
    id: Uuid,
    title: String,
    content: Option<String>,
    duration_minutes: Option<i32>,
    subject: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RecordRow> for RecordResponse {
    fn from(row: RecordRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            content: row.content,
            duration_minutes: row.duration_minutes,
            subject: row.subject,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// 创建学习记录
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    payload: CreateRecordRequest,
) -> AppResult<RecordResponse> {
    let record = sqlx::query_as::<_, RecordRow>(
        r#"INSERT INTO learning_records (user_id,
            title,
            content,
            duration_minutes,
            subject) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, content, duration_minutes,
            subject, created_at, updated_at"#,
    )
    .bind(user_id)
    .bind(payload.title)
    .bind(payload.content)
    .bind(payload.duration_minutes.unwrap_or(0))
    .bind(payload.subject)
    .fetch_one(pool)
    .await?;

    Ok(record.into())
}

/// 获取学习记录列表
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<RecordListResponse, AppError> {
    let items = sqlx::query_as::<_, RecordRow>(
        r#"
            SELECT id, title, content, duration_minutes, subject, created_at, updated_at
            FROM learning_records WHERE user_id = $1 ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT COUNT(*) FROM learning_records WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(RecordListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
    })
}

/// 获取单条学习记录
pub async fn detail(
    pool: &PgPool,
    user_id: Uuid,
    record_id: Uuid,
) -> Result<RecordResponse, AppError> {
    let record = sqlx::query_as::<_, RecordRow>(
        r#"
            SELECT id, title, content, duration_minutes, subject, created_at, updated_at
            FROM learning_records WHERE id = $1 AND user_id = $2
"#,
    )
    .bind(record_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("未查询到此数据".to_string()))?;

    Ok(record.into())
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    record_id: Uuid,
    payload: UpdateRecordRequest,
) -> Result<RecordResponse, AppError> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE learning_records SET ");
    let mut separated = builder.separated(", ");
    let mut has_field = false;

    if let Some(title) = payload.title {
        separated.push("title = ").push_bind_unseparated(title);
        has_field = true;
    }

    if let Some(content) = payload.content {
        separated.push("content = ").push_bind_unseparated(content);
        has_field = true;
    }

    if let Some(duration) = payload.duration_minutes {
        separated
            .push("duration_minutes = ")
            .push_bind_unseparated(duration);
        has_field = true;
    }

    if let Some(subject) = payload.subject {
        separated.push("subject = ").push_bind_unseparated(subject);
        has_field = true;
    }

    if !has_field {
        return Err(AppError::Validation("no fields to update".to_string()));
    }
    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(record_id)
        .push(" AND user_id = ")
        .push_bind(user_id)
        .push(
            r#"
            RETURNING
                id, title, content, duration_minutes,
                subject, created_at, updated_at
            "#,
        );

    let record = builder
        .build_query_as::<RecordRow>()
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("未查询到此纪录".to_string()))?;

    Ok(record.into())
}

pub async fn delete(pool: &PgPool, user_id: Uuid, record_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM learning_records
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(record_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("未查询到此数据".to_string()));
    }

    Ok(())
}
