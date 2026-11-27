//! 规划域的准入判定与状态迁移（纯函数）。
//!
//! 这里集中三件容易被分散实现而互相矛盾的事：
//! 1. **路线准入**——draft 可编辑、active 可执行、superseded 全程只读；
//! 2. **阶段四动作**——固定四条边，`pending` 不可直达 `completed`（ADR-007 §1 的两条例外）；
//! 3. **里程碑 / 任务的统一迁移规则**——一条规则替代 5×5 矩阵（ADR-007 §1）。

use crate::shared::{AppError, AppResult, MilestoneStatus, RoadmapStatus, StageStatus, TaskStatus};

// ---------------------------------------------------------------------------
// 路线准入
// ---------------------------------------------------------------------------

/// 编辑类写操作：`superseded` 路线全程只读（ADR-006 §6）
pub fn ensure_roadmap_writable(status: RoadmapStatus) -> AppResult<()> {
    if status == RoadmapStatus::Superseded {
        return Err(AppError::Conflict(
            "已废弃（superseded）的路线为只读历史，不能修改".to_string(),
        ));
    }
    Ok(())
}

/// 状态类动作：只有 `active` 路线的阶段可以被推进（设计记录 §2.4）
pub fn ensure_roadmap_active(status: RoadmapStatus) -> AppResult<()> {
    if status != RoadmapStatus::Active {
        return Err(AppError::Conflict(format!(
            "路线当前状态为 {}，仅 active 路线可执行阶段状态动作",
            status.as_str()
        )));
    }
    Ok(())
}

/// 删除类操作：只有草稿路线的结构可以物理删除（ADR-006 L1）
pub fn ensure_roadmap_draft(status: RoadmapStatus) -> AppResult<()> {
    if status != RoadmapStatus::Draft {
        return Err(AppError::Conflict(
            "仅草稿（draft）路线可删除结构；已确认路线请用状态动作表达调整".to_string(),
        ));
    }
    Ok(())
}

/// 路线本体与阶段的编辑：仅 `draft` 可改结构（`active` 路线的结构是承诺）。
///
/// 与 `ensure_roadmap_writable` 的区别：后者放行 `active`，用于里程碑 / 任务的状态类字段。
pub fn ensure_draft_editable(status: RoadmapStatus) -> AppResult<()> {
    if status != RoadmapStatus::Draft {
        return Err(AppError::Conflict(format!(
            "路线当前状态为 {}，只有 draft 路线可以编辑结构",
            status.as_str()
        )));
    }
    Ok(())
}

/// 路线确认：仅 `draft` 可确认
pub fn ensure_confirmable(status: RoadmapStatus) -> AppResult<()> {
    if status != RoadmapStatus::Draft {
        return Err(AppError::Conflict(format!(
            "路线当前状态为 {}，只有 draft 路线可以确认",
            status.as_str()
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 阶段四动作
// ---------------------------------------------------------------------------

/// 阶段动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageAction {
    Start,
    Complete,
    Skip,
    Reactivate,
}

impl StageAction {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Complete => "complete",
            Self::Skip => "skip",
            Self::Reactivate => "reactivate",
        }
    }

    /// 该动作的目标状态
    const fn target(&self) -> StageStatus {
        match self {
            Self::Start | Self::Reactivate => StageStatus::Active,
            Self::Complete => StageStatus::Completed,
            Self::Skip => StageStatus::Skipped,
        }
    }

    /// 合法源状态集合
    const fn sources(&self) -> &'static [StageStatus] {
        match self {
            Self::Start => &[StageStatus::Pending],
            Self::Complete => &[StageStatus::Active],
            Self::Skip => &[StageStatus::Pending, StageStatus::Active],
            Self::Reactivate => &[StageStatus::Completed, StageStatus::Skipped],
        }
    }
}

/// 动作执行计划：服务层照此写库，不含判断
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagePlan {
    /// 目标状态
    pub target: StageStatus,
    /// 是否同态幂等（返回 200 且不改任何字段）
    pub noop: bool,
    /// 是否写 `started_at = now()`（仅首次 start；后续不覆盖）
    pub write_started_at: bool,
    /// 是否写 `completed_at = now()`
    pub write_completed_at: bool,
    /// 是否清空 `completed_at`
    pub clear_completed_at: bool,
}

/// 计算阶段动作的执行计划。
///
/// 同态判据是「**动作的目标态 == 当前态**」（ADR-007 §1 第 4 条），而不是「当前态在源集合里」——
/// 后者会把 `active` 上重复 `start` 判成 409，与幂等承诺矛盾。
pub fn plan_stage_action(current: StageStatus, action: StageAction) -> AppResult<StagePlan> {
    let noop_plan = StagePlan {
        target: current,
        noop: true,
        write_started_at: false,
        write_completed_at: false,
        clear_completed_at: false,
    };

    if action.target() == current {
        return Ok(noop_plan);
    }

    if !action.sources().contains(&current) {
        return Err(AppError::Conflict(format!(
            "阶段当前状态 {} 不允许 {} 动作",
            current.as_str(),
            action.as_str()
        )));
    }

    Ok(StagePlan {
        target: action.target(),
        noop: false,
        // started_at 只在 pending → active 的首次开始时写入
        write_started_at: matches!(action, StageAction::Start),
        write_completed_at: matches!(action, StageAction::Complete),
        // 离开 completed 才清空；skipped → active 本来就没有 completed_at
        clear_completed_at: matches!(action, StageAction::Reactivate)
            && current == StageStatus::Completed,
    })
}

// ---------------------------------------------------------------------------
// 里程碑 / 任务的统一迁移规则
// ---------------------------------------------------------------------------

/// 迁移的副作用分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOutcome {
    /// 同态幂等：返回 200 且不改任何字段（含不刷新时间戳）
    Noop,
    /// 迁移到 `completed`：写 `completed_at = now()`
    EnterCompleted,
    /// 离开 `completed`：清空 `completed_at`
    LeaveCompleted,
    /// 其余迁移：只改 `status`
    Plain,
}

const COMPLETED: &str = "completed";

fn evaluate(current: &str, target: &str, terminals: &[&str]) -> AppResult<TransitionOutcome> {
    if current == target {
        return Ok(TransitionOutcome::Noop);
    }

    let from_terminal = terminals.contains(&current);
    let to_terminal = terminals.contains(&target);

    // 终态之间不直转：须先回退再转，保留判定的两步痕迹（ADR-007 §1 第 3 条）
    if from_terminal && to_terminal {
        return Err(AppError::Conflict(format!(
            "不允许从终态 {current} 直接转到终态 {target}，请先回退到未完态"
        )));
    }

    if to_terminal && target == COMPLETED {
        return Ok(TransitionOutcome::EnterCompleted);
    }

    if from_terminal && current == COMPLETED {
        return Ok(TransitionOutcome::LeaveCompleted);
    }

    Ok(TransitionOutcome::Plain)
}

/// 里程碑迁移（终态：`completed` / `skipped`）
pub fn evaluate_milestone(
    current: MilestoneStatus,
    target: MilestoneStatus,
) -> AppResult<TransitionOutcome> {
    evaluate(
        current.as_str(),
        target.as_str(),
        &[
            MilestoneStatus::Completed.as_str(),
            MilestoneStatus::Skipped.as_str(),
        ],
    )
}

/// 任务迁移（终态：`completed` / `cancelled`）
pub fn evaluate_task(current: TaskStatus, target: TaskStatus) -> AppResult<TransitionOutcome> {
    evaluate(
        current.as_str(),
        target.as_str(),
        &[
            TaskStatus::Completed.as_str(),
            TaskStatus::Cancelled.as_str(),
        ],
    )
}

// ---------------------------------------------------------------------------
// 字段校验
// ---------------------------------------------------------------------------

/// 标题类字段：去空白后 1..=max
pub fn ensure_title(field: &str, value: &str, max: usize) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{field} 不能为空")));
    }
    if trimmed.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{field} 长度不能超过 {max} 个字符"
        )));
    }
    Ok(trimmed.to_string())
}

/// 顺序号必须为正
pub fn ensure_sequence_no(value: i32) -> AppResult<()> {
    if value < 1 {
        return Err(AppError::Validation("sequence_no 必须大于 0".to_string()));
    }
    Ok(())
}
