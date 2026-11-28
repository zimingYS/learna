//! M05 学习证据 DTO。
//!
//! 契约来源：`openapi.yaml` 1.6.1 的 `EvidenceItem` / `CreateEvidenceRequest`。
//! `storage_key` / `mime_type` / `size_bytes` 无生产者，不在契约内（ADR-010 §4）。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{AppError, AppResult, EvidenceType};

/// 证据行（`learning_evidence` 的物理形状）
#[derive(Debug, sqlx::FromRow)]
pub struct EvidenceRow {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub evidence_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 证据响应（`EvidenceItem`）
#[derive(Debug, Serialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub evidence_type: EvidenceType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<EvidenceRow> for EvidenceItem {
    type Error = AppError;

    fn try_from(row: EvidenceRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            session_id: row.session_id,
            evidence_type: EvidenceType::parse(&row.evidence_type)?,
            title: row.title,
            description: row.description,
            external_url: row.external_url,
            created_at: row.created_at,
        })
    }
}

/// 证据列表数据（`EvidenceListData`）
#[derive(Debug, Serialize)]
pub struct EvidenceListData {
    pub items: Vec<EvidenceItem>,
    pub total: i64,
}

/// 新增证据请求（`CreateEvidenceRequest`）
#[derive(Debug, Deserialize)]
pub struct CreateEvidenceRequest {
    /// 可空：证据能独立存在（随手记的错题链接不必挂在某次活动上）
    pub session_id: Option<Uuid>,
    /// 用 `String` 接：`code` / `image` / `file` 要回统一信封的 422（ADR-010 §4）
    pub evidence_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
}

/// 证据列表查询：`session_id` 为空时返回全部（含独立证据）
#[derive(Debug, Deserialize)]
pub struct EvidenceQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub session_id: Option<Uuid>,
    pub evidence_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl EvidenceQuery {
    /// 返回 (limit, offset)，与全局分页约定一致（1 ≤ page_size ≤ 100）
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}
