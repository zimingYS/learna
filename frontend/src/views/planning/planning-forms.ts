import type {
  CreateGoalInput,
  CreateMilestoneInput,
  CreateStageInput,
  CreateTaskInput,
  GoalItem,
  GoalType,
  MilestoneItem,
  StageItem,
  TaskItem,
  UpdateGoalInput,
  UpdateMilestoneInput,
  UpdateStageInput,
  UpdateTaskInput,
} from "../../api/planning";
import {sameInstant, toIso, toLocalInput} from "../../lib/datetime";

export type BuildPayloadResult<T> =
  | { ok: true; payload: T }
  | { ok: false; error: string };

function trimmedOrNull(value: string): string | null {
  const trimmed = value.trim();

  return trimmed ? trimmed : null;
}

function ensureTitle(value: string, max: number): string | null {
  const trimmed = value.trim();

  if (!trimmed) {
    return null;
  }

  return trimmed.length > max ? null : trimmed;
}

function errorText(field: string, max: number): string {
  return `请输入${field}（不超过 ${max} 个字符）`;
}// ---------------------------------------------------------------------------
// 目标
// ---------------------------------------------------------------------------

export interface GoalFormFields {
  title: string;
  description: string;
  parentGoalId: string;
  goalType: GoalType;
  targetAt: string;
}

export function emptyGoalForm(): GoalFormFields {
  return {
    title: "",
    description: "",
    parentGoalId: "",
    goalType: "outcome",
    targetAt: "",
  };
}

export function goalToForm(goal: GoalItem): GoalFormFields {
  return {
    title: goal.title,
    description: goal.description ?? "",
    parentGoalId: goal.parent_goal_id ?? "",
    goalType: goal.goal_type,
    targetAt: toLocalInput(goal.target_at),
  };
}

export function buildGoalCreatePayload(fields: GoalFormFields): BuildPayloadResult<CreateGoalInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("目标标题", 200) };
  }

  const payload: CreateGoalInput = {title, goal_type: fields.goalType};

  const description = trimmedOrNull(fields.description);

  if (description) {
    payload.description = description;
  }

  if (fields.parentGoalId) {
    payload.parent_goal_id = fields.parentGoalId;
  }

  const targetAt = toIso(fields.targetAt);

  if (targetAt) {
    payload.target_at = targetAt;
  }

  return { ok: true, payload };
}

/// 只提交真正变化的字段：缺失＝不变、null＝置空（ADR-002 三态）
export function buildGoalUpdatePayload(
  goal: GoalItem,
  fields: GoalFormFields,
): BuildPayloadResult<UpdateGoalInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("目标标题", 200) };
  }

  const payload: UpdateGoalInput = {};

  if (title !== goal.title) {
    payload.title = title;
  }

  const description = trimmedOrNull(fields.description);

  if (description !== goal.description) {
    payload.description = description;
  }

  // 主目标不能被「选成」某个子目标：空值即提升为主目标，后端会再校验唯一性
  const parentGoalId = fields.parentGoalId ? fields.parentGoalId : null;

  if (parentGoalId !== goal.parent_goal_id) {
    payload.parent_goal_id = parentGoalId;
  }

  if (fields.goalType !== goal.goal_type) {
    payload.goal_type = fields.goalType;
  }

  const targetAt = toIso(fields.targetAt);

  if (!sameInstant(targetAt, goal.target_at)) {
    payload.target_at = targetAt;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, error: "没有需要保存的修改" };
  }

  return { ok: true, payload };
}

// ---------------------------------------------------------------------------
// 阶段
// ---------------------------------------------------------------------------

export interface StageFormFields {
  name: string;
  description: string;
  sequenceNo: string;
}

export function emptyStageForm(): StageFormFields {
  return {name: "", description: "", sequenceNo: ""};
}

export function stageToForm(stage: StageItem): StageFormFields {
  return {
    name: stage.name,
    description: stage.description ?? "",
    sequenceNo: String(stage.sequence_no),
  };
}

function parseSequenceNo(value: string): number | null {
  const trimmed = value.trim();

  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);

  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

export function buildStageCreatePayload(
  fields: StageFormFields,
): BuildPayloadResult<CreateStageInput> {
  const name = ensureTitle(fields.name, 160);

  if (name === null) {
    return { ok: false, error: errorText("阶段名称", 160) };
  }

  const payload: CreateStageInput = {name};

  const description = trimmedOrNull(fields.description);

  if (description) {
    payload.description = description;
  }

  // 留空即由服务端追加到末尾
  const sequenceNo = parseSequenceNo(fields.sequenceNo);

  if (fields.sequenceNo.trim() && sequenceNo === null) {
    return { ok: false, error: "顺序号必须是大于 0 的整数" };
  }

  if (sequenceNo !== null) {
    payload.sequence_no = sequenceNo;
  }

  return { ok: true, payload };
}

export function buildStageUpdatePayload(
  stage: StageItem,
  fields: StageFormFields,
): BuildPayloadResult<UpdateStageInput> {
  const name = ensureTitle(fields.name, 160);

  if (name === null) {
    return { ok: false, error: errorText("阶段名称", 160) };
  }

  const sequenceNo = parseSequenceNo(fields.sequenceNo);

  if (fields.sequenceNo.trim() && sequenceNo === null) {
    return { ok: false, error: "顺序号必须是大于 0 的整数" };
  }

  const payload: UpdateStageInput = {};

  if (name !== stage.name) {
    payload.name = name;
  }

  const description = trimmedOrNull(fields.description);

  if (description !== stage.description) {
    payload.description = description;
  }

  if (sequenceNo !== null && sequenceNo !== stage.sequence_no) {
    payload.sequence_no = sequenceNo;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, error: "没有需要保存的修改" };
  }

  return { ok: true, payload };
}

// ---------------------------------------------------------------------------
// 里程碑
// ---------------------------------------------------------------------------

export interface MilestoneFormFields {
  title: string;
  description: string;
}

export function emptyMilestoneForm(): MilestoneFormFields {
  return {title: "", description: ""};
}

export function milestoneToForm(milestone: MilestoneItem): MilestoneFormFields {
  return {title: milestone.title, description: milestone.description ?? ""};
}

export function buildMilestoneCreatePayload(
  stageId: string,
  fields: MilestoneFormFields,
): BuildPayloadResult<CreateMilestoneInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("里程碑标题", 200) };
  }

  const payload: CreateMilestoneInput = {stage_id: stageId, title};

  const description = trimmedOrNull(fields.description);

  if (description) {
    payload.description = description;
  }

  return {ok: true, payload};
}

export function buildMilestoneUpdatePayload(
  milestone: MilestoneItem,
  fields: MilestoneFormFields,
): BuildPayloadResult<UpdateMilestoneInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("里程碑标题", 200) };
  }

  const payload: UpdateMilestoneInput = {};

  if (title !== milestone.title) {
    payload.title = title;
  }

  const description = trimmedOrNull(fields.description);

  if (description !== milestone.description) {
    payload.description = description;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, error: "没有需要保存的修改" };
  }

  return { ok: true, payload };
}

// ---------------------------------------------------------------------------
// 任务
// ---------------------------------------------------------------------------

export interface TaskFormFields {
  title: string;
  description: string;
  stageId: string;
  milestoneId: string;
  priority: string;
  dueAt: string;
  estimatedMinutes: string;
}

export function emptyTaskForm(): TaskFormFields {
  return {
    title: "",
    description: "",
    stageId: "",
    milestoneId: "",
    priority: "3",
    dueAt: "",
    estimatedMinutes: "",
  };
}

export function taskToForm(task: TaskItem): TaskFormFields {
  return {
    title: task.title,
    description: task.description ?? "",
    stageId: task.stage_id ?? "",
    milestoneId: task.milestone_id ?? "",
    priority: String(task.priority),
    dueAt: toLocalInput(task.due_at),
    estimatedMinutes: task.estimated_minutes === null ? "" : String(task.estimated_minutes),
  };
}

function parsePriority(value: string): number | null {
  const level = Number(value);

  return Number.isInteger(level) && level >= 1 && level <= 5 ? level : null;
}

function parseMinutes(value: string): number | null {
  const trimmed = value.trim();

  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);

  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}

export function buildTaskCreatePayload(
  fields: TaskFormFields,
): BuildPayloadResult<CreateTaskInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("任务标题", 200) };
  }

  const priority = parsePriority(fields.priority);

  if (priority === null) {
    return { ok: false, error: "优先级必须是 1~5" };
  }

  if (fields.estimatedMinutes.trim() && parseMinutes(fields.estimatedMinutes) === null) {
    return { ok: false, error: "预计时长必须是非负整数（分钟）" };
  }

  const payload: CreateTaskInput = {title, priority};

  const description = trimmedOrNull(fields.description);

  if (description) {
    payload.description = description;
  }

  // 两者都可空：只挂项目、只挂阶段、或挂到具体里程碑
  if (fields.stageId) {
    payload.stage_id = fields.stageId;
  }

  if (fields.milestoneId) {
    payload.milestone_id = fields.milestoneId;
  }

  const dueAt = toIso(fields.dueAt);

  if (dueAt) {
    payload.due_at = dueAt;
  }

  const minutes = parseMinutes(fields.estimatedMinutes);

  if (minutes !== null) {
    payload.estimated_minutes = minutes;
  }

  return { ok: true, payload };
}

export function buildTaskUpdatePayload(
  task: TaskItem,
  fields: TaskFormFields,
): BuildPayloadResult<UpdateTaskInput> {
  const title = ensureTitle(fields.title, 200);

  if (title === null) {
    return { ok: false, error: errorText("任务标题", 200) };
  }

  const priority = parsePriority(fields.priority);

  if (priority === null) {
    return { ok: false, error: "优先级必须是 1~5" };
  }

  if (fields.estimatedMinutes.trim() && parseMinutes(fields.estimatedMinutes) === null) {
    return { ok: false, error: "预计时长必须是非负整数（分钟）" };
  }

  const payload: UpdateTaskInput = {};

  if (title !== task.title) {
    payload.title = title;
  }

  const description = trimmedOrNull(fields.description);

  if (description !== task.description) {
    payload.description = description;
  }

  if (priority !== task.priority) {
    payload.priority = priority;
  }

  const stageId = fields.stageId ? fields.stageId : null;

  if (stageId !== task.stage_id) {
    payload.stage_id = stageId;
  }

  const milestoneId = fields.milestoneId ? fields.milestoneId : null;

  if (milestoneId !== task.milestone_id) {
    payload.milestone_id = milestoneId;
  }

  const dueAt = toIso(fields.dueAt);

  if (!sameInstant(dueAt, task.due_at)) {
    payload.due_at = dueAt;
  }

  const minutes = parseMinutes(fields.estimatedMinutes);

  if (minutes !== task.estimated_minutes) {
    payload.estimated_minutes = minutes;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, error: "没有需要保存的修改" };
  }

  return { ok: true, payload };
}
