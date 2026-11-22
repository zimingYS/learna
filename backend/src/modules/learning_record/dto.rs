use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// 创建记录请求
#[derive(Debug, Deserialize)]
pub struct CreateRecordRequest {
    pub title: String,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub subject: Option<String>,
}

/// 更新记录请求
#[derive(Debug, Deserialize)]
pub struct UpdateRecordRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub subject: Option<String>,
}

/// 记录响应
#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub subject: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 记录列表响应
#[derive(Debug, Serialize)]
pub struct RecordListResponse {
    pub items: Vec<RecordResponse>,
    pub total: i64,
}

/// 分页参数
#[derive(Debug, Deserialize)]
pub struct RecordQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
