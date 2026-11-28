import type {
  ActivityType,
  CreateEvidenceInput,
  CreateSessionInput,
  EvidenceType,
  SessionContentInput,
  SessionContributionType,
  SessionDetailItem,
  SessionTaskInput,
  UpdateSessionInput,
} from "../../api/sessions";
import {sameInstant, toIso, toLocalInput} from "../../lib/datetime";
import {intervalMinutes} from "./session-rules";

/** 表单原始字段（输入框均为字符串） */
export interface SessionFormFields {
  title: string;
  activityType: ActivityType;
  startedAt: string;
  endedAt: string;
  durationMinutes: string;
  primaryProjectId: string;
  summary: string;
  whatDone: string;
  problems: string;
  nextSteps: string;
}

export type BuildResult<T> = {ok: true; value: T} | {ok: false; error: string};

const TITLE_MAX = 200;

/** 四段式描述共用同一套三态处理 */
type TextKey = "summary" | "what_done" | "problems" | "next_steps";

/** 默认填「刚过去的一小时」：补录是最常见的入口，计时提交会覆盖这三个字段 */
export function emptySessionForm(preset?: {
  startedAt?: string;
  endedAt?: string;
  durationMinutes?: number;
}): SessionFormFields {
  const end = preset?.endedAt ? new Date(preset.endedAt) : new Date();
  const start = preset?.startedAt
    ? new Date(preset.startedAt)
    : new Date(end.getTime() - 60 * 60_000);

  return {
    title: "",
    activityType: "mixed",
    startedAt: toLocalInput(start.toISOString()),
    endedAt: toLocalInput(end.toISOString()),
    durationMinutes:
      preset?.durationMinutes === undefined ? "" : String(preset.durationMinutes),
    primaryProjectId: "",
    summary: "",
    whatDone: "",
    problems: "",
    nextSteps: "",
  };
}

export function sessionToForm(item: SessionDetailItem): SessionFormFields {
  return {
    title: item.title ?? "",
    activityType: item.activity_type,
    startedAt: toLocalInput(item.started_at),
    endedAt: toLocalInput(item.ended_at),
    durationMinutes: String(item.duration_minutes),
    primaryProjectId: item.primary_project_id ?? "",
    summary: item.summary ?? "",
    whatDone: item.what_done ?? "",
    problems: item.problems ?? "",
    nextSteps: item.next_steps ?? "",
  };
}

/** 一次性取两个时刻；任一非法或倒序都直接给可读错误 */
function resolveRange(fields: SessionFormFields): BuildResult<{
  startedAt: string;
  endedAt: string;
}> {
  const startedAt = toIso(fields.startedAt);
  const endedAt = toIso(fields.endedAt);

  if (!startedAt) {
    return {ok: false, error: "请填写开始时间"};
  }

  if (!endedAt) {
    return {ok: false, error: "请填写结束时间"};
  }

  if (new Date(endedAt).getTime() < new Date(startedAt).getTime()) {
    return {ok: false, error: "结束时间不能早于开始时间"};
  }

  return {ok: true, value: {startedAt, endedAt}};
}

/** 时长留空表示「按区间计」；填了就必须 ≤ 区间（ADR-010 §1） */
function resolveDuration(
  raw: string,
  startedAt: string,
  endedAt: string,
): BuildResult<number | undefined> {
  const trimmed = raw.trim();

  if (trimmed === "") {
    return {ok: true, value: undefined};
  }

  const minutes = Number(trimmed);

  if (!Number.isSafeInteger(minutes) || minutes < 0) {
    return {ok: false, error: "有效时长必须是非负整数分钟"};
  }

  const span = intervalMinutes(startedAt, endedAt);

  if (span !== null && minutes > span) {
    return {
      ok: false,
      error: `有效时长 ${minutes} 分钟超过区间 ${span} 分钟，请调整时间或时长`,
    };
  }

  return {ok: true, value: minutes};
}

export function buildSessionCreatePayload(
  fields: SessionFormFields,
): BuildResult<CreateSessionInput> {
  const range = resolveRange(fields);

  if (!range.ok) {
    return range;
  }

  const title = fields.title.trim();

  if (title.length > TITLE_MAX) {
    return {ok: false, error: `标题不能超过 ${TITLE_MAX} 个字符`};
  }

  const duration = resolveDuration(
    fields.durationMinutes,
    range.value.startedAt,
    range.value.endedAt,
  );

  if (!duration.ok) {
    return duration;
  }

  const payload: CreateSessionInput = {
    activity_type: fields.activityType,
    started_at: range.value.startedAt,
    ended_at: range.value.endedAt,
  };

  if (title) {
    payload.title = title;
  }

  if (duration.value !== undefined) {
    payload.duration_minutes = duration.value;
  }

  if (fields.primaryProjectId) {
    payload.primary_project_id = fields.primaryProjectId;
  }

  const texts: [TextKey, string][] = [
    ["summary", fields.summary],
    ["what_done", fields.whatDone],
    ["problems", fields.problems],
    ["next_steps", fields.nextSteps],
  ];

  for (const [key, raw] of texts) {
    const value = raw.trim();

    if (value) {
      payload[key] = value;
    }
  }

  return {ok: true, value: payload};
}

/** 只提交真正变化的字段：三态语义下「没变」必须省略，而不是回传旧值 */
export function buildSessionUpdatePayload(
  current: SessionDetailItem,
  fields: SessionFormFields,
): BuildResult<UpdateSessionInput> {
  const range = resolveRange(fields);

  if (!range.ok) {
    return range;
  }

  const payload: UpdateSessionInput = {};

  const title = fields.title.trim();

  if (title.length > TITLE_MAX) {
    return {ok: false, error: `标题不能超过 ${TITLE_MAX} 个字符`};
  }

  if (title !== (current.title ?? "")) {
    payload.title = title === "" ? null : title;
  }

  if (fields.activityType !== current.activity_type) {
    payload.activity_type = fields.activityType;
  }

  if (!sameInstant(range.value.startedAt, current.started_at)) {
    payload.started_at = range.value.startedAt;
  }

  if (!sameInstant(range.value.endedAt, current.ended_at)) {
    payload.ended_at = range.value.endedAt;
  }

  const duration = resolveDuration(
    fields.durationMinutes,
    range.value.startedAt,
    range.value.endedAt,
  );

  if (!duration.ok) {
    return duration;
  }

  if (duration.value !== undefined && duration.value !== current.duration_minutes) {
    payload.duration_minutes = duration.value;
  }

  if (fields.primaryProjectId !== (current.primary_project_id ?? "")) {
    payload.primary_project_id = fields.primaryProjectId || null;
  }

  const texts: [TextKey, string | null, string][] = [
    ["summary", current.summary, fields.summary],
    ["what_done", current.what_done, fields.whatDone],
    ["problems", current.problems, fields.problems],
    ["next_steps", current.next_steps, fields.nextSteps],
  ];

  for (const [key, previous, raw] of texts) {
    const next = raw.trim();

    if (next === (previous ?? "")) {
      continue;
    }

    payload[key] = next === "" ? null : next;
  }

  return {ok: true, value: payload};
}

// ---------------------------------------------------------------------------
// 关联行（内容 / 任务）：整体替换前先在本地拦住明显的错
// ---------------------------------------------------------------------------

export interface ContentDraftRow {
  contentId: string;
  contributionType: SessionContributionType;
  note: string;
}

export interface TaskDraftRow {
  taskId: string;
  minutes: string;
  note: string;
}

export function buildContentItems(rows: ContentDraftRow[]): BuildResult<SessionContentInput[]> {
  const items: SessionContentInput[] = [];
  const seen = new Set<string>();

  for (const row of rows) {
    if (!row.contentId) {
      return {ok: false, error: "每条关联都要选择内容"};
    }

    if (seen.has(row.contentId)) {
      return {ok: false, error: "同一份内容不能重复关联"};
    }

    seen.add(row.contentId);

    const note = row.note.trim();
    items.push({
      content_id: row.contentId,
      contribution_type: row.contributionType,
      note: note === "" ? null : note,
    });
  }

  return {ok: true, value: items};
}

/** 分摊合计不得超过有效时长；这里与后端同规则，避免白跑一次请求 */
export function buildTaskItems(
  rows: TaskDraftRow[],
  durationMinutes: number,
): BuildResult<SessionTaskInput[]> {
  const items: SessionTaskInput[] = [];
  const seen = new Set<string>();
  let total = 0;

  for (const row of rows) {
    if (!row.taskId) {
      return {ok: false, error: "每条分摊都要选择任务"};
    }

    if (seen.has(row.taskId)) {
      return {ok: false, error: "同一个任务不能重复分摊"};
    }

    seen.add(row.taskId);

    const minutes = Number(row.minutes.trim() === "" ? "0" : row.minutes);

    if (!Number.isSafeInteger(minutes) || minutes < 0) {
      return {ok: false, error: "分摊时长必须是非负整数分钟"};
    }

    total += minutes;
    const note = row.note.trim();
    items.push({
      task_id: row.taskId,
      contribution_minutes: minutes,
      note: note === "" ? null : note,
    });
  }

  if (total > durationMinutes) {
    return {
      ok: false,
      error: `分摊合计 ${total} 分钟，超过本次有效学习时长 ${durationMinutes} 分钟`,
    };
  }

  return {ok: true, value: items};
}

/** 主要项目不能再出现在关联项目里（后端 422，这里先拦住） */
export function buildProjectIds(
  selected: string[],
  primaryProjectId: string,
): BuildResult<string[]> {
  if (primaryProjectId && selected.includes(primaryProjectId)) {
    return {ok: false, error: "主要项目不能同时出现在关联项目里"};
  }

  return {ok: true, value: [...selected]};
}

// ---------------------------------------------------------------------------
// 证据
// ---------------------------------------------------------------------------

export interface EvidenceFormFields {
  sessionId: string;
  type: EvidenceType;
  title: string;
  description: string;
  externalUrl: string;
}

export function emptyEvidenceForm(sessionId = ""): EvidenceFormFields {
  return {sessionId, type: "note", title: "", description: "", externalUrl: ""};
}

/** 按类型校验必填字段（ADR-010 §4）：note / test_result 用描述，link 用 http(s) 链接 */
export function buildEvidencePayload(
  fields: EvidenceFormFields,
): BuildResult<CreateEvidenceInput> {
  const description = fields.description.trim();
  const externalUrl = fields.externalUrl.trim();
  const title = fields.title.trim();

  if (title.length > TITLE_MAX) {
    return {ok: false, error: `标题不能超过 ${TITLE_MAX} 个字符`};
  }

  if (fields.type === "link") {
    if (!externalUrl) {
      return {ok: false, error: "链接类证据必须填写外部链接"};
    }

    if (!/^https?:\/\//i.test(externalUrl)) {
      return {ok: false, error: "外部链接只接受 http(s) 地址"};
    }
  } else if (!description) {
    return {ok: false, error: "笔记 / 测试结果类证据必须填写描述"};
  }

  const payload: CreateEvidenceInput = {evidence_type: fields.type};

  if (fields.sessionId) {
    payload.session_id = fields.sessionId;
  }

  if (title) {
    payload.title = title;
  }

  if (description) {
    payload.description = description;
  }

  if (externalUrl) {
    payload.external_url = externalUrl;
  }

  return {ok: true, value: payload};
}
