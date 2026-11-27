//! M04 任务（LearningTask）与依赖（TaskDependency）的 DTO。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{
    AppError, AppResult, CreatedSource, DependencyType, TaskStatus, double_option,
};

/// 任务行
#[derive(Debug, sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub milestone_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i16,
    pub due_at: Option<DateTime<Utc>>,
    pub estimated_minutes: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 任务响应（`TaskItem`）
#[derive(Debug, Serialize)]
pub struct TaskItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub milestone_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: i16,
    pub due_at: Option<DateTime<Utc>>,
    pub estimated_minutes: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_source: CreatedSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TaskRow> for TaskItem {
    type Error = AppError;

    fn try_from(row: TaskRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            project_id: row.project_id,
            milestone_id: row.milestone_id,
            stage_id: row.stage_id,
            title: row.title,
            description: row.description,
            status: TaskStatus::parse(&row.status)?,
            priority: row.priority,
            due_at: row.due_at,
            estimated_minutes: row.estimated_minutes,
            completed_at: row.completed_at,
            created_source: CreatedSource::parse(&row.created_source)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 任务列表数据
#[derive(Debug, Serialize)]
pub struct TaskListData {
    pub items: Vec<TaskItem>,
    pub total: i64,
}

/// 任务列表查询。`stage_id` 虽未在 openapi 参数表登记，但端点描述明确要求支持。
#[derive(Debug, Deserialize)]
pub struct TaskQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<TaskStatus>,
    pub milestone_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
}

impl TaskQuery {
    /// 返回 (limit, offset)
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}

/// 创建任务请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub milestone_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub priority: Option<i16>,
    pub due_at: Option<DateTime<Utc>>,
    pub estimated_minutes: Option<i32>,
}

/// 更新任务请求（含状态）
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub milestone_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "double_option")]
    pub stage_id: Option<Option<Uuid>>,
    pub priority: Option<i16>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, deserialize_with = "double_option")]
    pub estimated_minutes: Option<Option<i32>>,
    pub status: Option<TaskStatus>,
}

/// 依赖行
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TaskDependencyRow {
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
    pub dependency_type: String,
}

/// 依赖响应
#[derive(Debug, Serialize)]
pub struct TaskDependencyItem {
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
    pub dependency_type: DependencyType,
}

impl TryFrom<TaskDependencyRow> for TaskDependencyItem {
    type Error = AppError;

    fn try_from(row: TaskDependencyRow) -> AppResult<Self> {
        Ok(Self {
            task_id: row.task_id,
            depends_on_task_id: row.depends_on_task_id,
            dependency_type: DependencyType::parse(&row.dependency_type)?,
        })
    }
}

/// 依赖列表数据
#[derive(Debug, Serialize)]
pub struct TaskDependencyListData {
    pub items: Vec<TaskDependencyItem>,
}

/// 创建依赖请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskDependencyRequest {
    pub depends_on_task_id: Uuid,
    pub dependency_type: Option<DependencyType>,
}
