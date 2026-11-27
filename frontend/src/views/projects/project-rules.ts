import type {
  CreateProjectInput,
  DeadlineType,
  ProjectAction,
  ProjectItem,
  ProjectStatus,
  ProjectType,
  UpdateProjectInput,
} from "../../api/projects";

// 文案取自 PRD §4.3（五态）、§5.1（I1~I5）与 ERD §5.1（project_type / deadline_type）
export const PROJECT_STATUS_LABELS: Record<ProjectStatus, string> = {
  draft: "草稿",
  active: "进行中",
  paused: "已暂停",
  completed: "已完成",
  archived: "已归档",
};

export const PROJECT_TYPE_LABELS: Record<ProjectType, string> = {
  study: "学习",
  skill_development: "技能发展",
  challenge: "挑战",
  deliverable: "交付成果",
};

export const DEADLINE_TYPE_LABELS: Record<DeadlineType, string> = {
  none: "无截止",
  soft: "软截止",
  fixed: "固定截止",
};

export const IMPORTANCE_LABELS: Record<number, string> = {
  1: "I1 低",
  2: "I2 较低",
  3: "I3 普通",
  4: "I4 高",
  5: "I5 极高",
};

export const PROJECT_ACTION_LABELS: Record<ProjectAction, string> = {
  activate: "开始项目",
  pause: "暂停",
  resume: "继续推进",
  complete: "标记完成",
  reopen: "重新开启",
  archive: "归档",
};

// 下拉/筛选的选项源：显式声明值类型，避免模板里对 Object.entries 做断言
export const PROJECT_TYPE_OPTIONS: { value: ProjectType; label: string }[] = [
  { value: "study", label: PROJECT_TYPE_LABELS.study },
  { value: "skill_development", label: PROJECT_TYPE_LABELS.skill_development },
  { value: "challenge", label: PROJECT_TYPE_LABELS.challenge },
  { value: "deliverable", label: PROJECT_TYPE_LABELS.deliverable },
];

export const IMPORTANCE_OPTIONS: { value: string; label: string }[] = [1, 2, 3, 4, 5].map(
  (level) => ({ value: String(level), label: IMPORTANCE_LABELS[level] }),
);

export const DEADLINE_TYPE_OPTIONS: { value: DeadlineType; label: string }[] = [
  { value: "none", label: DEADLINE_TYPE_LABELS.none },
  { value: "soft", label: DEADLINE_TYPE_LABELS.soft },
  { value: "fixed", label: DEADLINE_TYPE_LABELS.fixed },
];

export const PROJECT_STATUS_OPTIONS: { value: ProjectStatus | "all"; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "active", label: PROJECT_STATUS_LABELS.active },
  { value: "draft", label: PROJECT_STATUS_LABELS.draft },
  { value: "paused", label: PROJECT_STATUS_LABELS.paused },
  { value: "completed", label: PROJECT_STATUS_LABELS.completed },
  { value: "archived", label: PROJECT_STATUS_LABELS.archived },
];

// 状态含义取自 PRD §4.3
export const PROJECT_STATUS_HINTS: Record<ProjectStatus, string> = {
  draft: "已创建，尚未正式开始",
  active: "正常推进中",
  paused: "暂时不参与常规学习安排",
  completed: "已确认目标完成",
  archived: "保留历史，不进入活跃项目列表",
};

// 线性图标路径，卡片与详情页共用
export const PROJECT_TYPE_ICONS: Record<ProjectType, string[]> = {
  study: [
    "M4 19.5A2.5 2.5 0 0 1 6.5 17H20",
    "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z",
  ],
  skill_development: ["m9 8-4 4 4 4", "m15 8 4 4-4 4"],
  challenge: ["M4 21V4", "M4 4h11l-1.5 4L15 12H4"],
  deliverable: ["M21 8 12 3 3 8l9 5 9-5Z", "M3 8v8l9 5 9-5V8", "M12 13v8"],
};

// 与 backend rules::transition 的迁移矩阵一致；未列出的组合一律拒绝
const TRANSITIONS: Record<ProjectStatus, ProjectAction[]> = {
  draft: ["activate", "archive"],
  active: ["pause", "complete", "archive"],
  paused: ["resume", "complete", "archive"],
  completed: ["reopen", "archive"],
  archived: ["reopen"],
};

export function availableActions(status: ProjectStatus): ProjectAction[] {
  return TRANSITIONS[status];
}

// 仅草稿可物理删除；其余状态只能归档或 purge
export function canDeleteDraft(status: ProjectStatus): boolean {
  return status === "draft";
}

// fixed 截止日期属外部约束，改期必须走 extend-deadline（D-04）
export function isDeadlineEditable(project: ProjectItem): boolean {
  return project.deadline_type !== "fixed";
}

/// 表单原始字段（输入框均为字符串，datetime-local 为本地时间原值）
export interface ProjectFormFields {
  name: string;
  description: string;
  projectType: ProjectType;
  importanceLevel: string;
  deadlineType: DeadlineType;
  deadlineAt: string;
  overrideUntil: string;
}

export type BuildPayloadResult<T> =
  | { ok: true; payload: T }
  | { ok: false; error: string };

export function emptyProjectForm(): ProjectFormFields {
  return {
    name: "",
    description: "",
    projectType: "study",
    importanceLevel: "3",
    deadlineType: "none",
    deadlineAt: "",
    overrideUntil: "",
  };
}

export function projectToForm(project: ProjectItem): ProjectFormFields {
  return {
    name: project.name,
    description: project.description ?? "",
    projectType: project.project_type,
    importanceLevel: String(project.importance_level),
    deadlineType: project.deadline_type,
    deadlineAt: toLocalInput(project.deadline_at),
    overrideUntil: toLocalInput(project.importance_override_until),
  };
}

// datetime-local 需要本地时间且不含时区后缀
export function toLocalInput(iso: string | null): string {
  if (!iso) {
    return "";
  }

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const pad = (value: number) => String(value).padStart(2, "0");

  return [
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
    `${pad(date.getHours())}:${pad(date.getMinutes())}`,
  ].join("T");
}

// 后端要求 RFC3339；空串表示未填
function toIso(input: string): string | null {
  const trimmed = input.trim();

  if (!trimmed) {
    return null;
  }

  const date = new Date(trimmed);

  if (Number.isNaN(date.getTime())) {
    return null;
  }

  return date.toISOString();
}

function parseImportance(value: string): number | null {
  const level = Number(value);

  return Number.isInteger(level) && level >= 1 && level <= 5 ? level : null;
}

// datetime-local 只精确到分钟，后端返回值带毫秒：比较前统一截断，
// 否则「原样打开再保存」会被判成改动过截止日期
function sameInstant(a: string | null, b: string | null): boolean {
  const toMinute = (iso: string | null): number | null => {
    if (!iso) {
      return null;
    }

    const time = new Date(iso).getTime();

    return Number.isNaN(time) ? null : Math.floor(time / 60_000);
  };

  return toMinute(a) === toMinute(b);
}

// 截止日期组合：none 必须无时刻，soft / fixed 必须有时刻（ERD §5.1）
function resolveDeadline(
  fields: ProjectFormFields,
): { ok: true; deadlineType: DeadlineType; deadlineAt: string | null } | { ok: false; error: string } {
  if (fields.deadlineType === "none") {
    return { ok: true, deadlineType: "none", deadlineAt: null };
  }

  const deadlineAt = toIso(fields.deadlineAt);

  if (!deadlineAt) {
    return { ok: false, error: "请选择截止日期时间" };
  }

  return { ok: true, deadlineType: fields.deadlineType, deadlineAt };
}

export function buildCreatePayload(
  fields: ProjectFormFields,
): BuildPayloadResult<CreateProjectInput> {
  const name = fields.name.trim();

  if (!name) {
    return { ok: false, error: "请输入项目名称" };
  }

  if (name.length > 160) {
    return { ok: false, error: "项目名称不能超过 160 个字符" };
  }

  const importance = parseImportance(fields.importanceLevel);

  if (importance === null) {
    return { ok: false, error: "重要程度必须是 1~5" };
  }

  const deadline = resolveDeadline(fields);

  if (!deadline.ok) {
    return { ok: false, error: deadline.error };
  }

  const payload: CreateProjectInput = {
    name,
    project_type: fields.projectType,
    importance_level: importance,
    deadline_type: deadline.deadlineType,
  };

  const description = fields.description.trim();

  if (description) {
    payload.description = description;
  }

  if (deadline.deadlineAt) {
    payload.deadline_at = deadline.deadlineAt;
  }

  const overrideUntil = toIso(fields.overrideUntil);

  if (overrideUntil) {
    payload.importance_override_until = overrideUntil;
  }

  return { ok: true, payload };
}

/// 只提交真正变化的字段：缺失＝不变、null＝置空（ADR-002 三态）
export function buildUpdatePayload(
  project: ProjectItem,
  fields: ProjectFormFields,
): BuildPayloadResult<UpdateProjectInput> {
  const name = fields.name.trim();

  if (!name) {
    return { ok: false, error: "请输入项目名称" };
  }

  if (name.length > 160) {
    return { ok: false, error: "项目名称不能超过 160 个字符" };
  }

  const importance = parseImportance(fields.importanceLevel);

  if (importance === null) {
    return { ok: false, error: "重要程度必须是 1~5" };
  }

  const deadlineTouched =
    fields.deadlineType !== project.deadline_type ||
    !sameInstant(toIso(fields.deadlineAt), project.deadline_at);

  if (deadlineTouched && !isDeadlineEditable(project)) {
    return { ok: false, error: "固定截止日期的项目请使用「修改截止日期」" };
  }

  const payload: UpdateProjectInput = {};

  if (name !== project.name) {
    payload.name = name;
  }

  // 清空描述要显式发 null，否则后端按「字段缺失」理解成不变
  const description = fields.description.trim() ? fields.description.trim() : null;

  if (description !== project.description) {
    payload.description = description;
  }

  if (fields.projectType !== project.project_type) {
    payload.project_type = fields.projectType;
  }

  if (importance !== project.importance_level) {
    payload.importance_level = importance;
  }

  const overrideUntil = toIso(fields.overrideUntil);

  if (!sameInstant(overrideUntil, project.importance_override_until)) {
    payload.importance_override_until = overrideUntil;
  }

  if (deadlineTouched) {
    const deadline = resolveDeadline(fields);

    if (!deadline.ok) {
      return { ok: false, error: deadline.error };
    }

    payload.deadline_type = deadline.deadlineType;
    payload.deadline_at = deadline.deadlineAt;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, error: "没有需要保存的修改" };
  }

  return { ok: true, payload };
}

export function formatDateTime(iso: string | null): string {
  if (!iso) {
    return "—";
  }

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "—";
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

/// 截止日期剩余天数：按本地自然日计算，已逾期为负
export function daysUntil(iso: string | null): number | null {
  if (!iso) {
    return null;
  }

  const target = new Date(iso);

  if (Number.isNaN(target.getTime())) {
    return null;
  }

  const startOfTarget = new Date(target.getFullYear(), target.getMonth(), target.getDate());
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  return Math.round((startOfTarget.getTime() - startOfToday.getTime()) / 86_400_000);
}
