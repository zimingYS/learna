import type {
  GoalStatus,
  GoalType,
  MilestoneStatus,
  RoadmapStatus,
  RoadmapSource,
  StageStatus,
  TaskAction,
  TaskStatus,
} from "../../api/planning";
import {daysUntil} from "../../lib/datetime";

// 文案取自 ERD §5.1 枚举注释与设计记录 §2.4 / §2.5
export const GOAL_TYPE_LABELS: Record<GoalType, string> = {
  outcome: "成果型",
  performance: "表现型",
  completion: "完成型",
};

export const GOAL_STATUS_LABELS: Record<GoalStatus, string> = {
  active: "进行中",
  achieved: "已达成",
  cancelled: "已取消",
};

export const ROADMAP_STATUS_LABELS: Record<RoadmapStatus, string> = {
  draft: "草稿",
  active: "生效中",
  superseded: "已废弃",
};

export const ROADMAP_SOURCE_LABELS: Record<RoadmapSource, string> = {
  user: "手动规划",
  ai: "AI 生成",
  template: "模板",
};

export const STAGE_STATUS_LABELS: Record<StageStatus, string> = {
  pending: "未开始",
  active: "进行中",
  completed: "已完成",
  skipped: "已跳过",
};

export const MILESTONE_STATUS_LABELS: Record<MilestoneStatus, string> = {
  pending: "未开始",
  in_progress: "进行中",
  completed: "已完成",
  skipped: "已跳过",
};

export const TASK_STATUS_LABELS: Record<TaskStatus, string> = {
  todo: "待办",
  in_progress: "进行中",
  blocked: "受阻",
  completed: "已完成",
  cancelled: "已取消",
};

export const TASK_ACTION_LABELS: Record<TaskAction, string> = {
  start: "开始阶段",
  complete: "标记完成",
  skip: "跳过",
  reactivate: "重新激活",
};

export const PRIORITY_LABELS: Record<number, string> = {
  1: "P1 最低",
  2: "P2 较低",
  3: "P3 普通",
  4: "P4 较高",
  5: "P5 最高",
};

// 下拉与筛选的选项源：显式声明值类型，避免模板里对 Object.entries 做断言
export const GOAL_TYPE_OPTIONS: { value: GoalType; label: string }[] = [
  { value: "outcome", label: GOAL_TYPE_LABELS.outcome },
  { value: "performance", label: GOAL_TYPE_LABELS.performance },
  { value: "completion", label: GOAL_TYPE_LABELS.completion },
];

export const TASK_STATUS_OPTIONS: { value: TaskStatus | "all"; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "todo", label: TASK_STATUS_LABELS.todo },
  { value: "in_progress", label: TASK_STATUS_LABELS.in_progress },
  { value: "blocked", label: TASK_STATUS_LABELS.blocked },
  { value: "completed", label: TASK_STATUS_LABELS.completed },
  { value: "cancelled", label: TASK_STATUS_LABELS.cancelled },
];

export const MILESTONE_STATUS_OPTIONS: { value: MilestoneStatus; label: string }[] = [
  { value: "pending", label: MILESTONE_STATUS_LABELS.pending },
  { value: "in_progress", label: MILESTONE_STATUS_LABELS.in_progress },
  { value: "completed", label: MILESTONE_STATUS_LABELS.completed },
  { value: "skipped", label: MILESTONE_STATUS_LABELS.skipped },
];

export const PRIORITY_OPTIONS: { value: string; label: string }[] = [5, 4, 3, 2, 1].map(
  (level) => ({ value: String(level), label: PRIORITY_LABELS[level] }),
);

// ---------------------------------------------------------------------------
// 状态动作：与 backend modules/roadmap/rules.rs 逐条对齐
// ---------------------------------------------------------------------------

/** 阶段四动作的合法边；同态动作后端幂等返回 200，前端不重复展示 */
const STAGE_ACTIONS: Record<StageStatus, TaskAction[]> = {
  pending: ["start", "skip"],
  active: ["complete", "skip"],
  completed: ["reactivate"],
  skipped: ["reactivate"],
};

export function stageActions(status: StageStatus): TaskAction[] {
  return STAGE_ACTIONS[status];
}

/** 目标状态边（契约 D-07b）：active ⇄ achieved / cancelled，achieved 不可直接转 cancelled */
const GOAL_TARGETS: Record<GoalStatus, GoalStatus[]> = {
  active: ["achieved", "cancelled"],
  achieved: ["active"],
  cancelled: ["active"],
};

export function goalStatusTargets(status: GoalStatus): GoalStatus[] {
  return GOAL_TARGETS[status];
}

/**
 * 里程碑 / 任务的统一迁移规则（ADR-007 §1）：允许跳过中间态、允许回退，
 * 但终态之间不直转，因此可选项要排除「当前终态 → 另一终态」。
 */
const MILESTONE_TERMINALS: MilestoneStatus[] = ["completed", "skipped"];
const TASK_TERMINALS: TaskStatus[] = ["completed", "cancelled"];

export function milestoneStatusTargets(status: MilestoneStatus): MilestoneStatus[] {
  return MILESTONE_STATUS_OPTIONS.map((option) => option.value).filter(
    (target) =>
      target !== status &&
      !(MILESTONE_TERMINALS.includes(status) && MILESTONE_TERMINALS.includes(target)),
  );
}

export function taskStatusTargets(status: TaskStatus): TaskStatus[] {
  return TASK_STATUS_OPTIONS.map((option) => option.value)
    .filter((value): value is TaskStatus => value !== "all")
    .filter(
      (target) =>
        target !== status &&
        !(TASK_TERMINALS.includes(status) && TASK_TERMINALS.includes(target)),
    );
}

// ---------------------------------------------------------------------------
// 路线准入（backend ensure_draft_editable / ensure_roadmap_draft / ensure_roadmap_writable）
// ---------------------------------------------------------------------------

/** 路线本体与阶段的增改：仅 draft（active 路线结构是承诺） */
export function canEditStructure(status: RoadmapStatus): boolean {
  return status === "draft";
}

/** 删除路线 / 阶段 / 里程碑：仅 draft */
export function canDeleteStructure(status: RoadmapStatus): boolean {
  return status === "draft";
}

export function canConfirm(status: RoadmapStatus): boolean {
  return status === "draft";
}

/** 阶段四动作：仅 active 路线 */
export function canAdvanceStage(status: RoadmapStatus): boolean {
  return status === "active";
}

/** 里程碑的增改：superseded 只读 */
export function canEditMilestone(status: RoadmapStatus): boolean {
  return status !== "superseded";
}

// ---------------------------------------------------------------------------
// 列表加工
// ---------------------------------------------------------------------------

/** 两层上限（D-06）：主目标唯一，其余都是它的子目标 */
export function groupGoals<T extends { id: string; title: string; parent_goal_id: string | null }>(
  goals: T[],
): { roots: T[]; childrenOf: Map<string, T[]> } {
  const roots = goals.filter((goal) => goal.parent_goal_id === null);
  const childrenOf = new Map<string, T[]>();

  for (const goal of goals) {
    if (goal.parent_goal_id === null) {
      continue;
    }

    const siblings = childrenOf.get(goal.parent_goal_id) ?? [];
    siblings.push(goal);
    childrenOf.set(goal.parent_goal_id, siblings);
  }

  return { roots, childrenOf };
}

/** 剩余天数文案：无截止返回 null，逾期返回「已逾期 N 天」 */
export function dueLabel(dueAt: string | null): { text: string; overdue: boolean } | null {
  const days = daysUntil(dueAt);

  if (days === null) {
    return null;
  }

  if (days < 0) {
    return { text: `已逾期 ${Math.abs(days)} 天`, overdue: true };
  }

  if (days === 0) {
    return { text: "今天到期", overdue: false };
  }

  return { text: `剩 ${days} 天`, overdue: false };
}

/** 任务排序权重：进行中优先、优先级高优先，用于阶段内任务列表 */
export function taskSortWeight(status: TaskStatus): number {
  const order: Record<TaskStatus, number> = {
    in_progress: 0,
    todo: 1,
    blocked: 2,
    completed: 3,
    cancelled: 4,
  };

  return order[status];
}

// ---------------------------------------------------------------------------
// 状态配色：不同资源同义状态用同一色板，避免每个面板各写一套
// ---------------------------------------------------------------------------

export type PillTone = "neutral" | "primary" | "success" | "warning" | "danger";

const STATUS_TONES: Record<string, PillTone> = {
  draft: "neutral",
  pending: "neutral",
  active: "primary",
  in_progress: "primary",
  achieved: "success",
  completed: "success",
  skipped: "warning",
  superseded: "neutral",
  cancelled: "danger",
  blocked: "danger",
  todo: "neutral",
};

export function statusTone(status: string): PillTone {
  return STATUS_TONES[status] ?? "neutral";
}
