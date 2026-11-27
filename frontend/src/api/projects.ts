import { request } from "./http";

// 与 openapi ProjectStatus / ProjectType / DeadlineType 一致
// deleting 是 purge 事务内的中间态，前端不产生也不展示
export type ProjectStatus = "draft" | "active" | "paused" | "completed" | "archived";

export type ProjectType = "study" | "skill_development" | "challenge" | "deliverable";

export type DeadlineType = "none" | "soft" | "fixed";

export type ProjectAction = "activate" | "pause" | "resume" | "complete" | "reopen" | "archive";

export interface ProjectItem {
  id: string;
  name: string;
  description: string | null;
  project_type: ProjectType;
  status: ProjectStatus;
  importance_level: number;
  importance_override_until: string | null;
  deadline_type: DeadlineType;
  deadline_at: string | null;
  archived_at: string | null;
  reopened_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectListData {
  items: ProjectItem[];
  total: number;
}

export interface ProjectListQuery {
  page?: number;
  page_size?: number;
  status?: ProjectStatus;
  include_archived?: boolean;
}

export interface CreateProjectInput {
  name: string;
  description?: string;
  project_type?: ProjectType;
  importance_level?: number;
  importance_override_until?: string;
  deadline_type?: DeadlineType;
  deadline_at?: string;
}

// 三态：字段缺失＝不变，null＝置空，值＝更新（ADR-002）
export interface UpdateProjectInput {
  name?: string;
  description?: string | null;
  project_type?: ProjectType;
  importance_level?: number;
  importance_override_until?: string | null;
  deadline_type?: DeadlineType;
  deadline_at?: string | null;
}

export interface ExtendDeadlineInput {
  new_deadline_at: string;
  reason: string;
}

export function fetchProjects(query: ProjectListQuery): Promise<ProjectListData> {
  return request<ProjectListData>({
    url: "/projects",
    method: "GET",
    params: query,
  });
}

export function fetchProject(id: string): Promise<ProjectItem> {
  return request<ProjectItem>({
    url: `/projects/${id}`,
    method: "GET",
  });
}

export function createProject(input: CreateProjectInput): Promise<ProjectItem> {
  return request<ProjectItem>({
    url: "/projects",
    method: "POST",
    data: input,
  });
}

export function updateProject(id: string, input: UpdateProjectInput): Promise<ProjectItem> {
  return request<ProjectItem>({
    url: `/projects/${id}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteProject(id: string): Promise<void> {
  await request<void>({
    url: `/projects/${id}`,
    method: "DELETE",
  });
}

// 六个生命周期动作共用一条路径模板，差异只在动作名
export function applyProjectAction(id: string, action: ProjectAction): Promise<ProjectItem> {
  return request<ProjectItem>({
    url: `/projects/${id}/${action}`,
    method: "POST",
  });
}

export function extendProjectDeadline(
  id: string,
  input: ExtendDeadlineInput,
): Promise<ProjectItem> {
  return request<ProjectItem>({
    url: `/projects/${id}/extend-deadline`,
    method: "POST",
    data: input,
  });
}

export async function purgeProject(id: string, confirmName: string): Promise<void> {
  await request<void>({
    url: `/projects/${id}/purge`,
    method: "POST",
    data: { confirm_name: confirmName },
  });
}
