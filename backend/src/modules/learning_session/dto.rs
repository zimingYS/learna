//! M05 学习活动 DTO。
//!
//! 契约来源：`openapi.yaml` 1.6.1 的 `SessionItem` / `SessionDetailItem` /
//! `CreateSessionRequest` / `UpdateSessionRequest` / `SessionContentItem` /
//! `SessionTaskItem` / `SetSession*Request`。
//!
//! 枚举字段在请求侧用 `String` 接、由 `rules` 解析：`created_source=imported` 与
//! `activity_type` 拼错都必须回统一信封的 422，而 serde 拒绝会走 axum 的纯文本
//! `JsonRejection`（M03 踩过的同一个坑）。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{
    ActivityType, AppError, AppResult, SessionContribution, SessionSource, double_option,
};

/// 列表行（`SessionItem` 的物理形状）；四段式描述只在详情查询里取
#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub title: Option<String>,
    pub activity_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub primary_project_id: Option<Uuid>,
    pub created_source: String,
    pub created_at: DateTime<Utc>,
}

/// 详情行：列表行 + 四段式描述
#[derive(Debug, sqlx::FromRow)]
pub struct SessionDetailRow {
    pub id: Uuid,
    pub title: Option<String>,
    pub activity_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub primary_project_id: Option<Uuid>,
    pub created_source: String,
    pub created_at: DateTime<Utc>,
    pub summary: Option<String>,
    pub what_done: Option<String>,
    pub problems: Option<String>,
    pub next_steps: Option<String>,
}

/// 列表视图（`SessionItem`）：不含四段式描述与关联明细
#[derive(Debug, Serialize)]
pub struct SessionItem {
    pub id: Uuid,
    pub title: Option<String>,
    pub activity_type: ActivityType,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub primary_project_id: Option<Uuid>,
    pub created_source: SessionSource,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<SessionRow> for SessionItem {
    type Error = AppError;

    fn try_from(row: SessionRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            title: row.title,
            activity_type: ActivityType::parse(&row.activity_type)?,
            started_at: row.started_at,
            ended_at: row.ended_at,
            duration_minutes: row.duration_minutes,
            primary_project_id: row.primary_project_id,
            created_source: SessionSource::parse(&row.created_source)?,
            created_at: row.created_at,
        })
    }
}

/// 详情（`SessionDetailItem`）：内联三组关联，一次往返拿全
#[derive(Debug, Serialize)]
pub struct SessionDetailItem {
    #[serde(flatten)]
    pub item: SessionItem,
    pub summary: Option<String>,
    pub what_done: Option<String>,
    pub problems: Option<String>,
    pub next_steps: Option<String>,
    /// 只含 related；主要项目见 `item.primary_project_id`
    pub related_project_ids: Vec<Uuid>,
    pub contents: Vec<SessionContentItem>,
    pub tasks: Vec<SessionTaskItem>,
}

impl SessionDetailItem {
    pub fn new(
        row: SessionDetailRow,
        related_project_ids: Vec<Uuid>,
        contents: Vec<SessionContentItem>,
        tasks: Vec<SessionTaskItem>,
    ) -> AppResult<Self> {
        Ok(Self {
            item: SessionItem {
                id: row.id,
                title: row.title,
                activity_type: ActivityType::parse(&row.activity_type)?,
                started_at: row.started_at,
                ended_at: row.ended_at,
                duration_minutes: row.duration_minutes,
                primary_project_id: row.primary_project_id,
                created_source: SessionSource::parse(&row.created_source)?,
                created_at: row.created_at,
            },
            summary: row.summary,
            what_done: row.what_done,
            problems: row.problems,
            next_steps: row.next_steps,
            related_project_ids,
            contents,
            tasks,
        })
    }
}

/// 活动列表数据（`SessionListData`）
#[derive(Debug, Serialize)]
pub struct SessionListData {
    pub items: Vec<SessionItem>,
    pub total: i64,
}

/// 关联内容行
#[derive(Debug, sqlx::FromRow)]
pub struct SessionContentRow {
    pub content_id: Uuid,
    pub contribution_type: String,
    pub note: Option<String>,
}

/// 关联内容（`SessionContentItem`，请求与响应同形）
#[derive(Debug, Serialize)]
pub struct SessionContentItem {
    pub content_id: Uuid,
    pub contribution_type: SessionContribution,
    pub note: Option<String>,
}

impl TryFrom<SessionContentRow> for SessionContentItem {
    type Error = AppError;

    fn try_from(row: SessionContentRow) -> AppResult<Self> {
        Ok(Self {
            content_id: row.content_id,
            contribution_type: SessionContribution::parse(&row.contribution_type)?,
            note: row.note,
        })
    }
}

/// 任务分摊行
#[derive(Debug, sqlx::FromRow)]
pub struct SessionTaskRow {
    pub task_id: Uuid,
    pub contribution_minutes: i32,
    pub note: Option<String>,
}

/// 任务分摊（`SessionTaskItem`，请求与响应同形）
#[derive(Debug, Serialize)]
pub struct SessionTaskItem {
    pub task_id: Uuid,
    pub contribution_minutes: i32,
    pub note: Option<String>,
}

impl From<SessionTaskRow> for SessionTaskItem {
    fn from(row: SessionTaskRow) -> Self {
        Self {
            task_id: row.task_id,
            contribution_minutes: row.contribution_minutes,
            note: row.note,
        }
    }
}

/// 关联内容列表数据（`SessionContentListData`）
#[derive(Debug, Serialize)]
pub struct SessionContentListData {
    pub items: Vec<SessionContentItem>,
    pub total: i64,
}

/// 关联项目（`SessionProjectsData`）
#[derive(Debug, Serialize)]
pub struct SessionProjectsData {
    pub project_ids: Vec<Uuid>,
}

/// 任务分摊列表数据（`SessionTaskListData`）
#[derive(Debug, Serialize)]
pub struct SessionTaskListData {
    pub items: Vec<SessionTaskItem>,
    pub total: i64,
}

/// 创建活动请求（`CreateSessionRequest`）：主表 + 三类关联一次提交，单事务
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub activity_type: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// 可选；缺省取区间分钟数，给出时不得超过区间（ADR-010 §1）
    pub duration_minutes: Option<i32>,
    pub primary_project_id: Option<Uuid>,
    #[serde(default)]
    pub related_project_ids: Vec<Uuid>,
    #[serde(default)]
    pub contents: Vec<SessionContentInput>,
    #[serde(default)]
    pub tasks: Vec<SessionTaskInput>,
    pub summary: Option<String>,
    pub what_done: Option<String>,
    pub problems: Option<String>,
    pub next_steps: Option<String>,
    /// 只接受 `user` / `timer`；`imported` 仅迁移脚本使用（ADR-010 §2）
    pub created_source: Option<String>,
}

/// 关联内容条目（`SessionContentItem` 的请求形态）
#[derive(Debug, Deserialize)]
pub struct SessionContentInput {
    pub content_id: Uuid,
    /// 拼错或缺失都在 `rules` 里转成统一信封的 422
    pub contribution_type: String,
    pub note: Option<String>,
}

/// 任务分摊条目（`SessionTaskItem` 的请求形态）
#[derive(Debug, Deserialize)]
pub struct SessionTaskInput {
    pub task_id: Uuid,
    pub contribution_minutes: i32,
    pub note: Option<String>,
}

/// 更新活动请求（`UpdateSessionRequest`）：三态，关联不在此端点修改
#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub title: Option<Option<String>>,
    pub activity_type: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    #[serde(default, deserialize_with = "double_option")]
    pub primary_project_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "double_option")]
    pub summary: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub what_done: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub problems: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub next_steps: Option<Option<String>>,
}

/// 整体替换关联内容（`SetSessionContentsRequest`）
#[derive(Debug, Deserialize)]
pub struct SetSessionContentsRequest {
    pub items: Vec<SessionContentInput>,
}

/// 整体替换关联项目（`SetSessionProjectsRequest`）
#[derive(Debug, Deserialize)]
pub struct SetSessionProjectsRequest {
    pub project_ids: Vec<Uuid>,
}

/// 整体替换任务分摊（`SetSessionTasksRequest`）
#[derive(Debug, Deserialize)]
pub struct SetSessionTasksRequest {
    pub items: Vec<SessionTaskInput>,
}

/// 活动列表查询：`from` / `to` 过滤 `started_at` 半开区间 `[from, to)`
#[derive(Debug, Deserialize)]
pub struct SessionQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub primary_project_id: Option<Uuid>,
    pub content_id: Option<Uuid>,
    pub activity_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl SessionQuery {
    /// 返回 (limit, offset)，与全局分页约定一致（1 ≤ page_size ≤ 100）
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}
