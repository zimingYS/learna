//! M02 学习项目的请求 / 响应 DTO。
//!
//! 契约来源：`openapi.yaml` 1.4.1 的 `ProjectItem` / `CreateProjectRequest` /
//! `UpdateProjectRequest` / `ExtendDeadlineRequest` / `PurgeProjectRequest`。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{AppError, AppResult, DeadlineType, ProjectStatus, ProjectType, double_option};

/// 项目行（`learning_projects` 的物理形状）。
///
/// 枚举列以 `String` 承载，转 DTO 时才解析成枚举：DB 的 `CHECK` 保证取值合法，
/// 解析失败属服务端缺陷，不应污染请求校验的错误通道。
#[derive(Debug, sqlx::FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: String,
    pub importance_level: i16,
    pub importance_override_until: Option<DateTime<Utc>>,
    pub deadline_type: String,
    pub deadline_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub reopened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 项目响应（`ProjectItem`）
#[derive(Debug, Serialize)]
pub struct ProjectItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub status: ProjectStatus,
    pub importance_level: i16,
    pub importance_override_until: Option<DateTime<Utc>>,
    pub deadline_type: DeadlineType,
    pub deadline_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub reopened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ProjectRow> for ProjectItem {
    type Error = AppError;

    fn try_from(row: ProjectRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            name: row.name,
            description: row.description,
            project_type: ProjectType::parse(&row.project_type)?,
            status: ProjectStatus::parse(&row.status)?,
            importance_level: row.importance_level,
            importance_override_until: row.importance_override_until,
            deadline_type: DeadlineType::parse(&row.deadline_type)?,
            deadline_at: row.deadline_at,
            archived_at: row.archived_at,
            reopened_at: row.reopened_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 项目列表数据（`ProjectListData`）
#[derive(Debug, Serialize)]
pub struct ProjectListData {
    pub items: Vec<ProjectItem>,
    pub total: i64,
}

/// 创建项目请求。省略字段取 D-19 默认值。
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<ProjectType>,
    pub importance_level: Option<i16>,
    pub importance_override_until: Option<DateTime<Utc>>,
    pub deadline_type: Option<DeadlineType>,
    pub deadline_at: Option<DateTime<Utc>>,
}

/// 部分更新请求。可空字段用 `Option<Option<T>>` 表达三态（ADR-002）。
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    pub project_type: Option<ProjectType>,
    pub importance_level: Option<i16>,
    #[serde(default, deserialize_with = "double_option")]
    pub importance_override_until: Option<Option<DateTime<Utc>>>,
    pub deadline_type: Option<DeadlineType>,
    #[serde(default, deserialize_with = "double_option")]
    pub deadline_at: Option<Option<DateTime<Utc>>>,
}

/// 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ProjectQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<ProjectStatus>,
    pub include_archived: Option<bool>,
}

/// fixed 截止日期改期请求（D-04）
#[derive(Debug, Deserialize)]
pub struct ExtendDeadlineRequest {
    pub new_deadline_at: DateTime<Utc>,
    pub reason: String,
}

/// 强制删除请求（D-02b）
#[derive(Debug, Deserialize)]
pub struct PurgeProjectRequest {
    pub confirm_name: String,
}
