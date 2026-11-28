//! 项目内容上下文的准入规则：等级 / 优先级范围与结论型状态边界。

use crate::shared::{AppError, AppResult, ProjectContentStatus};

/// 目标掌握等级 0–4（ERD §5.2）
pub fn ensure_mastery_level(value: i16) -> AppResult<i16> {
    if !(0..=4).contains(&value) {
        return Err(AppError::Validation(
            "target_mastery_level 必须在 0–4 之间".to_string(),
        ));
    }
    Ok(value)
}

/// 优先级 1–5（与项目 `importance_level` 同口径）
pub fn ensure_priority(value: i16) -> AppResult<i16> {
    if !(1..=5).contains(&value) {
        return Err(AppError::Validation("priority 必须在 1–5 之间".to_string()));
    }
    Ok(value)
}

/// 项目内容状态的可写集合（D-27）。
///
/// `achieved` 是掌握结论而非动作结果，只能由 M06 评价流程写入；用户提交即 409。
pub fn ensure_user_writable_status(target: ProjectContentStatus) -> AppResult<()> {
    if target == ProjectContentStatus::Achieved {
        return Err(AppError::Conflict(
            "achieved 由掌握度评价流程写入，不能手工置位".to_string(),
        ));
    }
    Ok(())
}
