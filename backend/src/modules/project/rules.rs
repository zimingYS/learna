//! 项目生命周期的准入判定与字段校验。
//!
//! 全部为纯函数：状态机在这里做单元测试，service 只负责读写数据库，
//! 避免「迁移矩阵」散落在 SQL 的 `WHERE status = ...` 里无法单测。

use sqlx::types::chrono::{DateTime, Utc};

use crate::shared::{AppError, AppResult, DeadlineType, ProjectStatus};

/// 项目生命周期动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAction {
    Activate,
    Pause,
    Resume,
    Complete,
    Reopen,
    Archive,
}

impl ProjectAction {
    /// 线上端点名，用于错误信息
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Activate => "activate",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Complete => "complete",
            Self::Reopen => "reopen",
            Self::Archive => "archive",
        }
    }
}

/// 计算迁移后的目标状态。
///
/// `None` 表示**未定义迁移**（含 `deleting` 上的任何动作），调用方一律转 409。
/// 三条已决策的边缘迁移（D-01）：暂停→已完成开放、草稿→已归档开放、已完成→暂停拒绝。
pub fn transition(current: ProjectStatus, action: ProjectAction) -> Option<ProjectStatus> {
    use ProjectAction as A;
    use ProjectStatus as S;

    match (current, action) {
        (S::Draft, A::Activate) => Some(S::Active),
        (S::Active, A::Pause) => Some(S::Paused),
        (S::Paused, A::Resume) => Some(S::Active),
        (S::Active | S::Paused, A::Complete) => Some(S::Completed),
        (S::Completed | S::Archived, A::Reopen) => Some(S::Active),
        (S::Draft | S::Active | S::Paused | S::Completed, A::Archive) => Some(S::Archived),
        _ => None,
    }
}

/// 该动作是否写入 `archived_at`
pub const fn writes_archived_at(action: ProjectAction) -> bool {
    matches!(action, ProjectAction::Archive)
}

/// 该动作是否写 `reopened_at` 并清 `archived_at`
pub const fn writes_reopened_at(action: ProjectAction) -> bool {
    matches!(action, ProjectAction::Reopen)
}

/// 项目名：去空白后 1–160 字符（与 `ck_learning_projects_name` 对齐）
pub fn ensure_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("name 不能为空".to_string()));
    }
    if trimmed.chars().count() > 160 {
        return Err(AppError::Validation(
            "name 长度不能超过 160 个字符".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// 重要程度：1–5（DB CHECK 兜底，此处提前转 422 而非 500）
pub fn ensure_importance(level: i16) -> AppResult<()> {
    if !(1..=5).contains(&level) {
        return Err(AppError::Validation(
            "importance_level 必须在 1 到 5 之间".to_string(),
        ));
    }
    Ok(())
}

/// 截止日期组合：`none` 必须无 `deadline_at`；`soft` / `fixed` 必须有时刻
/// （与 `ck_learning_projects_deadline_pair` 对齐，ERD §5.1）
pub fn ensure_deadline_pair(
    deadline_type: DeadlineType,
    deadline_at: Option<DateTime<Utc>>,
) -> AppResult<()> {
    match (deadline_type, deadline_at) {
        (DeadlineType::None, Some(_)) => Err(AppError::Validation(
            "deadline_type=none 时不能指定 deadline_at".to_string(),
        )),
        (DeadlineType::Soft | DeadlineType::Fixed, None) => Err(AppError::Validation(
            "deadline_type 为 soft / fixed 时必须提供 deadline_at".to_string(),
        )),
        _ => Ok(()),
    }
}
