import { request } from "./http";

// 与 openapi ActivityType / SessionSource / SessionContributionType / EvidenceType 一致
export type ActivityType = "theory" | "practice" | "review" | "mixed" | "other";

export type SessionSource = "user" | "timer" | "imported";

export type SessionContributionType = "primary" | "covered" | "practiced";

export type EvidenceType = "note" | "link" | "test_result";

// 列表项：不含四段式描述与关联明细（详情才内联）
export interface SessionItem {
  id: string;
  title: string | null;
  activity_type: ActivityType;
  started_at: string;
  ended_at: string;
  duration_minutes: number;
  primary_project_id: string | null;
  created_source: SessionSource;
  created_at: string;
}

export interface SessionContentItem {
  content_id: string;
  contribution_type: SessionContributionType;
  note: string | null;
}

export interface SessionTaskItem {
  task_id: string;
  contribution_minutes: number;
  note: string | null;
}

export interface SessionDetailItem extends SessionItem {
  summary: string | null;
  what_done: string | null;
  problems: string | null;
  next_steps: string | null;
  related_project_ids: string[];
  contents: SessionContentItem[];
  tasks: SessionTaskItem[];
}

export interface SessionListData {
  items: SessionItem[];
  total: number;
}

export interface SessionListQuery {
  page?: number;
  page_size?: number;
  primary_project_id?: string;
  content_id?: string;
  activity_type?: ActivityType;
  from?: string;
  to?: string;
}

export interface SessionContentInput {
  content_id: string;
  contribution_type: SessionContributionType;
  note?: string | null;
}

export interface SessionTaskInput {
  task_id: string;
  contribution_minutes: number;
  note?: string | null;
}

export interface SessionContentListData {
  items: SessionContentItem[];
  total: number;
}

export interface SessionTaskListData {
  items: SessionTaskItem[];
  total: number;
}

export interface SessionProjectsData {
  project_ids: string[];
}

export interface CreateSessionInput {
  title?: string | null;
  activity_type?: ActivityType;
  started_at: string;
  ended_at: string;
  duration_minutes?: number;
  primary_project_id?: string | null;
  related_project_ids?: string[];
  contents?: SessionContentInput[];
  tasks?: SessionTaskInput[];
  summary?: string | null;
  what_done?: string | null;
  problems?: string | null;
  next_steps?: string | null;
  // imported 仅供迁移脚本使用，API 提交会被后端 422
  created_source?: Exclude<SessionSource, "imported">;
}

// 三态 PATCH：字段缺失＝不变、显式 null＝置空、值＝更新（ADR-002）
export interface UpdateSessionInput {
  title?: string | null;
  activity_type?: ActivityType;
  started_at?: string;
  ended_at?: string;
  duration_minutes?: number;
  primary_project_id?: string | null;
  summary?: string | null;
  what_done?: string | null;
  problems?: string | null;
  next_steps?: string | null;
}

export interface EvidenceItem {
  id: string;
  session_id: string | null;
  evidence_type: EvidenceType;
  title: string | null;
  description: string | null;
  external_url: string | null;
  created_at: string;
}

export interface EvidenceListData {
  items: EvidenceItem[];
  total: number;
}

export interface EvidenceListQuery {
  page?: number;
  page_size?: number;
  session_id?: string;
  evidence_type?: EvidenceType;
  from?: string;
  to?: string;
}

export interface CreateEvidenceInput {
  session_id?: string | null;
  evidence_type: EvidenceType;
  title?: string | null;
  description?: string | null;
  external_url?: string | null;
}

// ---------------------------------------------------------------------------
// 学习活动
// ---------------------------------------------------------------------------

export function fetchSessions(query: SessionListQuery): Promise<SessionListData> {
  return request<SessionListData>({
    url: "/learning-sessions",
    method: "GET",
    params: query,
  });
}

export function fetchSession(id: string): Promise<SessionDetailItem> {
  return request<SessionDetailItem>({
    url: `/learning-sessions/${id}`,
    method: "GET",
  });
}

export function createSession(input: CreateSessionInput): Promise<SessionDetailItem> {
  return request<SessionDetailItem>({
    url: "/learning-sessions",
    method: "POST",
    data: input,
  });
}

export function updateSession(
  id: string,
  input: UpdateSessionInput,
): Promise<SessionDetailItem> {
  return request<SessionDetailItem>({
    url: `/learning-sessions/${id}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteSession(id: string): Promise<void> {
  await request<void>({
    url: `/learning-sessions/${id}`,
    method: "DELETE",
  });
}

// ---------------------------------------------------------------------------
// 三类关联的整体替换（PUT，空数组即清空）
// ---------------------------------------------------------------------------

export function setSessionContents(
  id: string,
  items: SessionContentInput[],
): Promise<SessionContentListData> {
  return request<SessionContentListData>({
    url: `/learning-sessions/${id}/contents`,
    method: "PUT",
    data: { items },
  });
}

export function setSessionProjects(
  id: string,
  projectIds: string[],
): Promise<SessionProjectsData> {
  return request<SessionProjectsData>({
    url: `/learning-sessions/${id}/projects`,
    method: "PUT",
    data: { project_ids: projectIds },
  });
}

export function setSessionTasks(
  id: string,
  items: SessionTaskInput[],
): Promise<SessionTaskListData> {
  return request<SessionTaskListData>({
    url: `/learning-sessions/${id}/tasks`,
    method: "PUT",
    data: { items },
  });
}

// ---------------------------------------------------------------------------
// 学习证据
// ---------------------------------------------------------------------------

export function fetchEvidence(query: EvidenceListQuery): Promise<EvidenceListData> {
  return request<EvidenceListData>({
    url: "/evidence",
    method: "GET",
    params: query,
  });
}

export function fetchEvidenceItem(id: string): Promise<EvidenceItem> {
  return request<EvidenceItem>({
    url: `/evidence/${id}`,
    method: "GET",
  });
}

export function createEvidence(input: CreateEvidenceInput): Promise<EvidenceItem> {
  return request<EvidenceItem>({
    url: "/evidence",
    method: "POST",
    data: input,
  });
}

export async function deleteEvidence(id: string): Promise<void> {
  await request<void>({
    url: `/evidence/${id}`,
    method: "DELETE",
  });
}
