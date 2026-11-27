import {request} from "./http";

// 枚举值与 openapi.yaml 的 GoalType / GoalStatus / RoadmapStatus / StageStatus /
// MilestoneStatus / TaskStatus / Contribution / DependencyType 一一对应
export type GoalType = "outcome" | "performance" | "completion";

export type GoalStatus = "active" | "achieved" | "cancelled";

export type Contribution = "primary" | "supporting";

export type RoadmapStatus = "draft" | "active" | "superseded";

export type RoadmapSource = "user" | "ai" | "template";

export type StageStatus = "pending" | "active" | "completed" | "skipped";

export type CompletionMethod = "user" | "system_confirmed" | "ai_suggested";

export type MilestoneStatus = "pending" | "in_progress" | "completed" | "skipped";

export type TaskStatus = "todo" | "in_progress" | "blocked" | "completed" | "cancelled";

export type TaskAction = "start" | "complete" | "skip" | "reactivate";

export type DependencyType = "finish_to_start";

// ---------------------------------------------------------------------------
// 目标
// ---------------------------------------------------------------------------

export interface GoalItem {
  id: string;
  project_id: string;
  parent_goal_id: string | null;
  title: string;
  description: string | null;
  goal_type: GoalType;
  status: GoalStatus;
  target_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface GoalListData {
  items: GoalItem[];
  total: number;
}

export interface CreateGoalInput {
  title: string;
  description?: string;
  parent_goal_id?: string | null;
  goal_type?: GoalType;
  target_at?: string;
}

// 三态：字段缺失＝不变，null＝置空，值＝更新
export interface UpdateGoalInput {
  title?: string;
  description?: string | null;
  parent_goal_id?: string | null;
  goal_type?: GoalType;
  target_at?: string | null;
  status?: GoalStatus;
}

export interface GoalStageItem {
  goal_id: string;
  stage_id: string;
  contribution: Contribution;
}

export interface GoalStageLink {
  stage_id: string;
  contribution: Contribution;
}

// ---------------------------------------------------------------------------
// 路线
// ---------------------------------------------------------------------------

export interface RoadmapItem {
  id: string;
  project_id: string;
  version: number;
  status: RoadmapStatus;
  source: RoadmapSource;
  change_summary: string | null;
  confirmed_at: string | null;
  created_by: string | null;
  created_at: string;
  updated_at: string;
}

export interface RoadmapListData {
  items: RoadmapItem[];
  total: number;
}

export interface CreateRoadmapInput {
  source?: "user" | "template";
  change_summary?: string;
}

export interface UpdateRoadmapInput {
  change_summary?: string | null;
}

// ---------------------------------------------------------------------------
// 阶段
// ---------------------------------------------------------------------------

export interface StageItem {
  id: string;
  roadmap_id: string;
  name: string;
  description: string | null;
  sequence_no: number;
  status: StageStatus;
  completion_method: CompletionMethod;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateStageInput {
  name: string;
  description?: string;
  sequence_no?: number;
}

export interface UpdateStageInput {
  name?: string;
  description?: string | null;
  sequence_no?: number;
}

// ---------------------------------------------------------------------------
// 里程碑
// ---------------------------------------------------------------------------

export interface MilestoneItem {
  id: string;
  stage_id: string;
  title: string;
  description: string | null;
  status: MilestoneStatus;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface MilestoneListData {
  items: MilestoneItem[];
  total: number;
}

export interface CreateMilestoneInput {
  stage_id: string;
  title: string;
  description?: string;
}

export interface UpdateMilestoneInput {
  title?: string;
  description?: string | null;
  status?: MilestoneStatus;
}

// ---------------------------------------------------------------------------
// 任务与依赖
// ---------------------------------------------------------------------------

export interface TaskItem {
  id: string;
  project_id: string;
  milestone_id: string | null;
  stage_id: string | null;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: number;
  due_at: string | null;
  estimated_minutes: number | null;
  completed_at: string | null;
  created_source: "user" | "ai" | "template";
  created_at: string;
  updated_at: string;
}

export interface TaskListData {
  items: TaskItem[];
  total: number;
}

export interface TaskListQuery {
  page?: number;
  page_size?: number;
  status?: TaskStatus;
  milestone_id?: string;
  stage_id?: string;
}

export interface CreateTaskInput {
  title: string;
  description?: string;
  milestone_id?: string | null;
  stage_id?: string | null;
  priority?: number;
  due_at?: string;
  estimated_minutes?: number;
}

export interface UpdateTaskInput {
  title?: string;
  description?: string | null;
  milestone_id?: string | null;
  stage_id?: string | null;
  priority?: number;
  due_at?: string | null;
  estimated_minutes?: number | null;
  status?: TaskStatus;
}

export interface TaskDependencyItem {
  task_id: string;
  depends_on_task_id: string;
  dependency_type: DependencyType;
}

export interface CreateDependencyInput {
  depends_on_task_id: string;
  dependency_type?: DependencyType;
}

// ---------------------------------------------------------------------------
// 端点
// ---------------------------------------------------------------------------

const base = (projectId: string) => `/projects/${projectId}`;

export function fetchGoals(
  projectId: string,
  page: number,
  pageSize: number,
): Promise<GoalListData> {
  return request<GoalListData>({
    url: `${base(projectId)}/goals`,
    method: "GET",
    params: {page, page_size: pageSize},
  });
}

export function createGoal(projectId: string, input: CreateGoalInput): Promise<GoalItem> {
  return request<GoalItem>({
    url: `${base(projectId)}/goals`,
    method: "POST",
    data: input,
  });
}

export function updateGoal(
  projectId: string,
  goalId: string,
  input: UpdateGoalInput,
): Promise<GoalItem> {
  return request<GoalItem>({
    url: `${base(projectId)}/goals/${goalId}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteGoal(
  projectId: string,
  goalId: string,
  cascade: boolean,
): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/goals/${goalId}`,
    method: "DELETE",
    params: cascade ? {cascade: true} : undefined,
  });
}

export function fetchGoalStages(
  projectId: string,
  goalId: string,
): Promise<{items: GoalStageItem[]}> {
  return request<{items: GoalStageItem[]}>({
    url: `${base(projectId)}/goals/${goalId}/stages`,
    method: "GET",
  });
}

export function replaceGoalStages(
  projectId: string,
  goalId: string,
  links: GoalStageLink[],
): Promise<{items: GoalStageItem[]}> {
  return request<{items: GoalStageItem[]}>({
    url: `${base(projectId)}/goals/${goalId}/stages`,
    method: "PUT",
    data: {links},
  });
}

export function fetchRoadmaps(
  projectId: string,
  page: number,
  pageSize: number,
): Promise<RoadmapListData> {
  return request<RoadmapListData>({
    url: `${base(projectId)}/roadmaps`,
    method: "GET",
    params: {page, page_size: pageSize},
  });
}

export function createRoadmap(
  projectId: string,
  input: CreateRoadmapInput,
): Promise<RoadmapItem> {
  return request<RoadmapItem>({
    url: `${base(projectId)}/roadmaps`,
    method: "POST",
    data: input,
  });
}

export function updateRoadmap(
  projectId: string,
  roadmapId: string,
  input: UpdateRoadmapInput,
): Promise<RoadmapItem> {
  return request<RoadmapItem>({
    url: `${base(projectId)}/roadmaps/${roadmapId}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteRoadmap(projectId: string, roadmapId: string): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/roadmaps/${roadmapId}`,
    method: "DELETE",
  });
}

export function confirmRoadmap(projectId: string, roadmapId: string): Promise<RoadmapItem> {
  return request<RoadmapItem>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/confirm`,
    method: "POST",
  });
}

export function fetchStages(
  projectId: string,
  roadmapId: string,
): Promise<{items: StageItem[]}> {
  return request<{items: StageItem[]}>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/stages`,
    method: "GET",
  });
}

export function createStage(
  projectId: string,
  roadmapId: string,
  input: CreateStageInput,
): Promise<StageItem> {
  return request<StageItem>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/stages`,
    method: "POST",
    data: input,
  });
}

export function updateStage(
  projectId: string,
  roadmapId: string,
  stageId: string,
  input: UpdateStageInput,
): Promise<StageItem> {
  return request<StageItem>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/stages/${stageId}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteStage(
  projectId: string,
  roadmapId: string,
  stageId: string,
): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/stages/${stageId}`,
    method: "DELETE",
  });
}

// 四个状态动作共用一条路径模板，差异只在动作名
export function applyStageAction(
  projectId: string,
  roadmapId: string,
  stageId: string,
  action: TaskAction,
): Promise<StageItem> {
  return request<StageItem>({
    url: `${base(projectId)}/roadmaps/${roadmapId}/stages/${stageId}/${action}`,
    method: "POST",
  });
}

export function fetchMilestones(
  projectId: string,
  query: {page?: number; page_size?: number; stage_id?: string},
): Promise<MilestoneListData> {
  return request<MilestoneListData>({
    url: `${base(projectId)}/milestones`,
    method: "GET",
    params: query,
  });
}

export function createMilestone(
  projectId: string,
  input: CreateMilestoneInput,
): Promise<MilestoneItem> {
  return request<MilestoneItem>({
    url: `${base(projectId)}/milestones`,
    method: "POST",
    data: input,
  });
}

export function updateMilestone(
  projectId: string,
  milestoneId: string,
  input: UpdateMilestoneInput,
): Promise<MilestoneItem> {
  return request<MilestoneItem>({
    url: `${base(projectId)}/milestones/${milestoneId}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteMilestone(
  projectId: string,
  milestoneId: string,
): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/milestones/${milestoneId}`,
    method: "DELETE",
  });
}

export function fetchTasks(projectId: string, query: TaskListQuery): Promise<TaskListData> {
  return request<TaskListData>({
    url: `${base(projectId)}/tasks`,
    method: "GET",
    params: query,
  });
}

export function createTask(projectId: string, input: CreateTaskInput): Promise<TaskItem> {
  return request<TaskItem>({
    url: `${base(projectId)}/tasks`,
    method: "POST",
    data: input,
  });
}

export function updateTask(
  projectId: string,
  taskId: string,
  input: UpdateTaskInput,
): Promise<TaskItem> {
  return request<TaskItem>({
    url: `${base(projectId)}/tasks/${taskId}`,
    method: "PATCH",
    data: input,
  });
}

export async function deleteTask(projectId: string, taskId: string): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/tasks/${taskId}`,
    method: "DELETE",
  });
}

export function fetchDependencies(
  projectId: string,
  taskId: string,
): Promise<{items: TaskDependencyItem[]}> {
  return request<{items: TaskDependencyItem[]}>({
    url: `${base(projectId)}/tasks/${taskId}/dependencies`,
    method: "GET",
  });
}

export function createDependency(
  projectId: string,
  taskId: string,
  input: CreateDependencyInput,
): Promise<TaskDependencyItem> {
  return request<TaskDependencyItem>({
    url: `${base(projectId)}/tasks/${taskId}/dependencies`,
    method: "POST",
    data: input,
  });
}

export async function deleteDependency(
  projectId: string,
  taskId: string,
  dependsOnTaskId: string,
): Promise<void> {
  await request<void>({
    url: `${base(projectId)}/tasks/${taskId}/dependencies/${dependsOnTaskId}`,
    method: "DELETE",
  });
}
