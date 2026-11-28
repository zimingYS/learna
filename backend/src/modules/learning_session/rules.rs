//! 学习活动的时间、来源与分摊规则（ADR-010 §1 / §2）。
//!
//! 全部是纯函数：时长推导的边界（零长度、跨午夜、上报超区间）与分摊总和不经过
//! 数据库即可验证，单测直接覆盖。

use std::collections::HashSet;

use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::learning_session::dto::{SessionContentInput, SessionTaskInput};
use crate::shared::{ActivityType, AppError, AppResult, SessionContribution, SessionSource};

/// 标题长度上限，与 DB 的 `VARCHAR(200)` 同值
pub const MAX_TITLE_LEN: usize = 200;

/// 区间分钟数（向下取整）；跨午夜不特殊处理，比时间先后即可
pub fn interval_minutes(started_at: DateTime<Utc>, ended_at: DateTime<Utc>) -> i32 {
    let minutes = (ended_at - started_at).num_minutes();
    i32::try_from(minutes).unwrap_or(i32::MAX)
}

/// `ended_at` 不得早于 `started_at`
pub fn ensure_interval(started_at: DateTime<Utc>, ended_at: DateTime<Utc>) -> AppResult<()> {
    if ended_at < started_at {
        return Err(AppError::Validation("结束时间不能早于开始时间".to_string()));
    }
    Ok(())
}

/// 有效时长 = `min(用户上报, 区间分钟数)`，用户上报可选（ADR-010 §1）
///
/// 允许小于区间（区间内有暂停），禁止大于——上界由服务端算。
pub fn derive_duration(
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    reported: Option<i32>,
) -> AppResult<i32> {
    let span = interval_minutes(started_at, ended_at);

    match reported {
        None => Ok(span),
        Some(value) if value < 0 => Err(AppError::Validation("有效学习时长不能为负数".to_string())),
        Some(value) if value > span => Err(AppError::Validation(format!(
            "上报 {value} 分钟，但区间只有 {span} 分钟"
        ))),
        Some(value) => Ok(value),
    }
}

/// PATCH 的时长校验：只改区间也必须让既有时长仍在范围内
pub fn ensure_duration_fits(
    duration: i32,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> AppResult<()> {
    if duration < 0 {
        return Err(AppError::Validation("有效学习时长不能为负数".to_string()));
    }

    let span = interval_minutes(started_at, ended_at);
    if duration > span {
        return Err(AppError::Validation(format!(
            "有效学习时长 {duration} 分钟超过区间 {span} 分钟"
        )));
    }
    Ok(())
}

/// `activity_type` 拼错回 422 而不是让 serde 抛纯文本
pub fn parse_activity_type(value: &str) -> AppResult<ActivityType> {
    ActivityType::parse(value).map_err(|_| {
        AppError::Validation(format!(
            "activity_type 只接受 theory / practice / review / mixed / other，收到 {value}"
        ))
    })
}

/// 内容贡献方式同上
pub fn parse_contribution(value: &str) -> AppResult<SessionContribution> {
    SessionContribution::parse(value).map_err(|_| {
        AppError::Validation(format!(
            "contribution_type 只接受 primary / covered / practiced，收到 {value}"
        ))
    })
}

/// API 只允许 `user` / `timer`；`imported` 是迁移脚本专用（ADR-010 §2）
pub fn parse_api_source(value: Option<&str>) -> AppResult<SessionSource> {
    match value {
        None => Ok(SessionSource::User),
        Some("imported") => Err(AppError::Validation(
            "created_source 不接受 imported：该值仅供迁移脚本使用".to_string(),
        )),
        Some(value) => match SessionSource::parse(value) {
            Ok(source) => Ok(source),
            Err(_) => Err(AppError::Validation(format!(
                "created_source 只接受 user / timer，收到 {value}"
            ))),
        },
    }
}

/// 空白归一为 NULL：避免存下「看起来是空」的文本
pub fn normalize_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// 标题规范化 + 长度校验（DB 侧同值 CHECK 兜底）
pub fn normalize_title(value: Option<String>) -> AppResult<Option<String>> {
    let title = normalize_text(value);

    if let Some(title) = &title
        && title.chars().count() > MAX_TITLE_LEN
    {
        return Err(AppError::Validation(format!(
            "标题不能超过 {MAX_TITLE_LEN} 个字符"
        )));
    }
    Ok(title)
}

/// 关联项目去重（保序）。
///
/// 契约把 `contents` / `tasks` 的重复 id 定为 422，对 `project_ids` 未作规定——
/// 重复的项目只是同一段归因写了两遍，去重比报错更贴近用户意图。
pub fn dedupe_ids(ids: &[Uuid]) -> Vec<Uuid> {
    let mut unique = Vec::with_capacity(ids.len());

    for id in ids {
        if !unique.contains(id) {
            unique.push(*id);
        }
    }
    unique
}

/// 主要项目不得重复出现在关联项目里（ERD §5.3：双重事实会分叉）
pub fn ensure_related_excludes_primary(primary: Option<Uuid>, related: &[Uuid]) -> AppResult<()> {
    if let Some(primary) = primary
        && related.contains(&primary)
    {
        return Err(AppError::Validation(
            "related_project_ids 不能包含主要项目；主要项目由 primary_project_id 唯一表达"
                .to_string(),
        ));
    }
    Ok(())
}

/// 重复 `content_id` → 422（契约 §4.2）
pub fn ensure_unique_contents(items: &[SessionContentInput]) -> AppResult<()> {
    let mut seen = HashSet::with_capacity(items.len());

    for item in items {
        if !seen.insert(item.content_id) {
            return Err(AppError::Validation(format!(
                "关联内容中存在重复的 content_id：{}",
                item.content_id
            )));
        }
    }
    Ok(())
}

/// 重复 `task_id` → 422
pub fn ensure_unique_tasks(items: &[SessionTaskInput]) -> AppResult<()> {
    let mut seen = HashSet::with_capacity(items.len());

    for item in items {
        if !seen.insert(item.task_id) {
            return Err(AppError::Validation(format!(
                "任务分摊中存在重复的 task_id：{}",
                item.task_id
            )));
        }
    }
    Ok(())
}

/// 分摊只是归因，不构成第二个时长：合计不得超过本次有效时长（跨行聚合，DB 表达不了）
pub fn ensure_allocation_within_duration(
    items: &[SessionTaskInput],
    duration: i32,
) -> AppResult<()> {
    let mut total: i64 = 0;

    for item in items {
        if item.contribution_minutes < 0 {
            return Err(AppError::Validation("任务分摊时长不能为负数".to_string()));
        }
        total += i64::from(item.contribution_minutes);
    }

    if total > i64::from(duration) {
        return Err(AppError::Validation(format!(
            "任务分摊合计 {total} 分钟，超过本次有效学习时长 {duration} 分钟"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 28, hour, minute, 0)
            .single()
            .expect("固定时刻应可构造")
    }

    #[test]
    fn zero_length_interval_yields_zero_duration() {
        assert_eq!(derive_duration(at(9, 0), at(9, 0), None).unwrap(), 0);
    }

    #[test]
    fn interval_across_midnight_is_measured_by_elapsed_time_only() {
        // 23:30 → 次日 00:30 是 60 分钟，不按日期分段
        let start = at(23, 30);
        let end = start + Duration::minutes(60);

        assert_eq!(interval_minutes(start, end), 60);
        assert_eq!(derive_duration(start, end, None).unwrap(), 60);
    }

    #[test]
    fn reported_duration_smaller_than_interval_is_accepted() {
        // 区间 120 分钟，用户上报 45（中间接了个电话）→ 采信上报值
        let start = at(9, 0);
        let end = start + Duration::minutes(120);

        assert_eq!(derive_duration(start, end, Some(45)).unwrap(), 45);
    }

    #[test]
    fn reported_duration_beyond_interval_is_rejected() {
        let start = at(9, 0);
        let end = start + Duration::minutes(60);

        assert!(derive_duration(start, end, Some(90)).is_err());
        // 恰好等于区间不算超
        assert_eq!(derive_duration(start, end, Some(60)).unwrap(), 60);
    }

    #[test]
    fn negative_reported_duration_is_rejected() {
        assert!(derive_duration(at(9, 0), at(10, 0), Some(-1)).is_err());
    }

    #[test]
    fn reversed_interval_is_rejected() {
        assert!(ensure_interval(at(10, 0), at(9, 0)).is_err());
        assert!(ensure_interval(at(9, 0), at(9, 0)).is_ok());
    }

    #[test]
    fn shrinking_interval_below_existing_duration_is_rejected() {
        // 既有 90 分钟，把区间改成 60 分钟 → 必须同时改时长
        assert!(ensure_duration_fits(90, at(9, 0), at(10, 0)).is_err());
        assert!(ensure_duration_fits(60, at(9, 0), at(10, 0)).is_ok());
    }

    #[test]
    fn dedupe_keeps_first_occurrence_order() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);

        assert_eq!(dedupe_ids(&[a, b, a]), vec![a, b]);
    }

    #[test]
    fn primary_project_cannot_be_repeated_in_related_list() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);

        assert!(ensure_related_excludes_primary(Some(a), &[a, b]).is_err());
        assert!(ensure_related_excludes_primary(Some(a), &[b]).is_ok());
        assert!(ensure_related_excludes_primary(None, &[a]).is_ok());
    }

    #[test]
    fn allocation_must_not_exceed_duration() {
        let items = |minutes: i32| SessionTaskInput {
            task_id: Uuid::from_u128(1),
            contribution_minutes: minutes,
            note: None,
        };

        assert!(ensure_allocation_within_duration(&[items(30), items(30)], 60).is_ok());
        assert!(ensure_allocation_within_duration(&[items(30), items(31)], 60).is_err());
        assert!(ensure_allocation_within_duration(&[items(-1)], 60).is_err());
    }
}
