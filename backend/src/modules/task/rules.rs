//! 任务的字段校验与状态迁移。
//!
//! 迁移规则的唯一实现在 `modules::roadmap::rules::evaluate_task`：任务与里程碑同构
//! （ADR-007 §1），在这里复制一份必然漂移。

pub use crate::modules::roadmap::rules::{TransitionOutcome, evaluate_task};

use crate::shared::{AppError, AppResult};

/// 优先级：1–5（DB CHECK 兜底，提前转 422）
pub fn ensure_priority(priority: i16) -> AppResult<()> {
    if !(1..=5).contains(&priority) {
        return Err(AppError::Validation(
            "priority 必须在 1 到 5 之间".to_string(),
        ));
    }
    Ok(())
}

/// 预估时长：非负
pub fn ensure_estimated_minutes(minutes: i32) -> AppResult<()> {
    if minutes < 0 {
        return Err(AppError::Validation(
            "estimated_minutes 不能为负数".to_string(),
        ));
    }
    Ok(())
}
