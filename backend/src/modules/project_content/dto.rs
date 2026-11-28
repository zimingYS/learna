//! M03 项目内容上下文 DTO：内容在某个项目里的状态与目标。
//!
//! 契约来源：`openapi.yaml` 1.6.1 的 `ProjectContentItem` /
//! `CreateProjectContentRequest` / `UpdateProjectContentRequest`。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{AppError, AppResult, ProjectContentStatus};

/// 关联行
#[derive(Debug, sqlx::FromRow)]
pub struct ProjectContentRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub content_id: Uuid,
    pub current_status: String,
    pub target_mastery_level: i16,
    pub priority: i16,
    pub auto_review_enabled: bool,
    pub linked_at: DateTime<Utc>,
    pub achieved_at: Option<DateTime<Utc>>,
}

/// 关联响应（`ProjectContentItem`）
#[derive(Debug, Serialize)]
pub struct ProjectContentItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub content_id: Uuid,
    pub current_status: ProjectContentStatus,
    pub target_mastery_level: i16,
    pub priority: i16,
    pub auto_review_enabled: bool,
    pub linked_at: DateTime<Utc>,
    pub achieved_at: Option<DateTime<Utc>>,
}

impl TryFrom<ProjectContentRow> for ProjectContentItem {
    type Error = AppError;

    fn try_from(row: ProjectContentRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            project_id: row.project_id,
            content_id: row.content_id,
            current_status: ProjectContentStatus::parse(&row.current_status)?,
            target_mastery_level: row.target_mastery_level,
            priority: row.priority,
            auto_review_enabled: row.auto_review_enabled,
            linked_at: row.linked_at,
            achieved_at: row.achieved_at,
        })
    }
}

/// 关联列表数据（`ProjectContentListData`）
#[derive(Debug, Serialize)]
pub struct ProjectContentListData {
    pub items: Vec<ProjectContentItem>,
    pub total: i64,
}

/// 内容加入项目请求
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectContentRequest {
    pub project_id: Uuid,
    pub content_id: Uuid,
    pub target_mastery_level: Option<i16>,
    pub priority: Option<i16>,
    pub auto_review_enabled: Option<bool>,
}

/// 更新项目内状态与目标；`current_status` 不含 `achieved`（D-27）
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProjectContentRequest {
    pub current_status: Option<ProjectContentStatus>,
    pub target_mastery_level: Option<i16>,
    pub priority: Option<i16>,
    pub auto_review_enabled: Option<bool>,
}

/// 关联列表查询：`project_id` 与 `content_id` 二选一必填
#[derive(Debug, Deserialize)]
pub struct ProjectContentQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub project_id: Option<Uuid>,
    pub content_id: Option<Uuid>,
    pub current_status: Option<ProjectContentStatus>,
}

impl ProjectContentQuery {
    /// 返回 (limit, offset)，与全局分页约定一致（1 ≤ page_size ≤ 100）
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}
