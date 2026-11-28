import { request } from "./http";

// 与 openapi ContentType / ContentStatus / BodyFormat / RelationType 一致
export type ContentType = "knowledge_point" | "skill_point" | "language_item";

export type ContentStatus = "active" | "archived";

export type BodyFormat = "markdown" | "plain_text";

export type ContentCreatedSource = "user" | "ai_confirmed" | "template";

export type RelationType =
  | "prerequisite"
  | "related"
  | "contains"
  | "extends"
  | "alternative";

export type ProjectContentStatus =
  | "not_started"
  | "learning"
  | "reviewing"
  | "achieved"
  | "waived";

export type RelationDirection = "outbound" | "inbound";

// 列表项：不含 body（ADR-009 §2）
export interface ContentItem {
  id: string;
  title: string;
  description: string | null;
  content_type: ContentType;
  parent_id: string | null;
  source_url: string | null;
  status: ContentStatus;
  body_format: BodyFormat;
  created_source: ContentCreatedSource;
  created_at: string;
  updated_at: string;
}

// 详情：唯一带 body 的视图，depth 为树深度（根为 0）
export interface ContentDetailItem extends ContentItem {
  body: string | null;
  depth: number;
}

export interface ContentHint {
  id: string;
  title: string;
  similarity: number;
}

export interface ContentListData {
  items: ContentItem[];
  total: number;
}

export interface ContentCreateData {
  item: ContentDetailItem;
  similar_hints: ContentHint[];
}

export interface ContentListQuery {
  page?: number;
  page_size?: number;
  content_type?: ContentType;
  status?: ContentStatus | "all";
  parent_id?: string;
  root_only?: boolean;
  q?: string;
}

export interface CreateContentInput {
  title: string;
  content_type: ContentType;
  description?: string | null;
  body?: string | null;
  body_format?: BodyFormat;
  source_url?: string | null;
  parent_id?: string | null;
}

// 三态：字段缺失＝不变，null＝置空，值＝更新（ADR-002）
export interface UpdateContentInput {
  title?: string;
  description?: string | null;
  body?: string | null;
  body_format?: BodyFormat;
  source_url?: string | null;
  parent_id?: string | null;
  content_type?: ContentType;
}

export interface RelationItem {
  id: string;
  source_content_id: string;
  target_content_id: string;
  relation_type: RelationType;
  direction: RelationDirection;
}

export interface RelationListData {
  items: RelationItem[];
  total: number;
}

export interface RelationListQuery {
  relation_type?: RelationType;
  direction?: RelationDirection | "all";
}

export interface ContentTagItem {
  id: string;
  name: string;
  color: string | null;
  created_at: string;
}

export interface ContentTagListData {
  items: ContentTagItem[];
  total: number;
}

export interface ContentTagQuery {
  page?: number;
  page_size?: number;
  q?: string;
}

export interface ProjectContentItem {
  id: string;
  project_id: string;
  content_id: string;
  current_status: ProjectContentStatus;
  target_mastery_level: number;
  priority: number;
  auto_review_enabled: boolean;
  linked_at: string;
  achieved_at: string | null;
}

export interface ProjectContentListData {
  items: ProjectContentItem[];
  total: number;
}

export interface ProjectContentQuery {
  page?: number;
  page_size?: number;
  project_id?: string;
  content_id?: string;
  current_status?: ProjectContentStatus;
}

export interface CreateProjectContentInput {
  project_id: string;
  content_id: string;
  target_mastery_level?: number;
  priority?: number;
  auto_review_enabled?: boolean;
}

// achieved 不在可写集合内：由 M06 评价流程写入（D-27）
export interface UpdateProjectContentInput {
  current_status?: Exclude<ProjectContentStatus, "achieved">;
  target_mastery_level?: number;
  priority?: number;
  auto_review_enabled?: boolean;
}

// ---------------------------------------------------------------------------
// 内容本体
// ---------------------------------------------------------------------------

export function fetchContents(query: ContentListQuery): Promise<ContentListData> {
  return request<ContentListData>({
    url: "/learning-contents",
    method: "GET",
    params: query,
  });
}

export function fetchContent(id: string): Promise<ContentDetailItem> {
  return request<ContentDetailItem>({
    url: `/learning-contents/${id}`,
    method: "GET",
  });
}

export function createContent(
  payload: CreateContentInput,
): Promise<ContentCreateData> {
  return request<ContentCreateData>({
    url: "/learning-contents",
    method: "POST",
    data: payload,
  });
}

export function updateContent(
  id: string,
  payload: UpdateContentInput,
): Promise<ContentDetailItem> {
  return request<ContentDetailItem>({
    url: `/learning-contents/${id}`,
    method: "PATCH",
    data: payload,
  });
}

/** 默认归档；purge=true 为物理删除（需已归档、无项目关联、无子内容） */
export function deleteContent(id: string, purge = false): Promise<null> {
  return request<null>({
    url: `/learning-contents/${id}`,
    method: "DELETE",
    params: purge ? { purge: true } : undefined,
  });
}

// ---------------------------------------------------------------------------
// 内容关系
// ---------------------------------------------------------------------------

export function fetchRelations(
  contentId: string,
  query: RelationListQuery = {},
): Promise<RelationListData> {
  return request<RelationListData>({
    url: `/learning-contents/${contentId}/relations`,
    method: "GET",
    params: query,
  });
}

export function createRelation(
  contentId: string,
  payload: { target_content_id: string; relation_type: RelationType },
): Promise<RelationItem> {
  return request<RelationItem>({
    url: `/learning-contents/${contentId}/relations`,
    method: "POST",
    data: payload,
  });
}

export function deleteRelation(
  contentId: string,
  relationId: string,
): Promise<null> {
  return request<null>({
    url: `/learning-contents/${contentId}/relations/${relationId}`,
    method: "DELETE",
  });
}

// ---------------------------------------------------------------------------
// 标签
// ---------------------------------------------------------------------------

export function fetchContentTags(contentId: string): Promise<ContentTagListData> {
  return request<ContentTagListData>({
    url: `/learning-contents/${contentId}/tags`,
    method: "GET",
  });
}

/** 整体替换；空数组表示清空（D-28） */
export function setContentTags(
  contentId: string,
  tagIds: string[],
): Promise<ContentTagListData> {
  return request<ContentTagListData>({
    url: `/learning-contents/${contentId}/tags`,
    method: "PUT",
    data: { tag_ids: tagIds },
  });
}

export function fetchTags(query: ContentTagQuery = {}): Promise<ContentTagListData> {
  return request<ContentTagListData>({
    url: "/content-tags",
    method: "GET",
    params: query,
  });
}

export function createTag(payload: {
  name: string;
  color?: string | null;
}): Promise<ContentTagItem> {
  return request<ContentTagItem>({
    url: "/content-tags",
    method: "POST",
    data: payload,
  });
}

export function updateTag(
  id: string,
  payload: { name?: string; color?: string | null },
): Promise<ContentTagItem> {
  return request<ContentTagItem>({
    url: `/content-tags/${id}`,
    method: "PATCH",
    data: payload,
  });
}

export function deleteTag(id: string): Promise<null> {
  return request<null>({
    url: `/content-tags/${id}`,
    method: "DELETE",
  });
}

// ---------------------------------------------------------------------------
// 项目内容关联
// ---------------------------------------------------------------------------

export function fetchProjectContents(
  query: ProjectContentQuery,
): Promise<ProjectContentListData> {
  return request<ProjectContentListData>({
    url: "/project-contents",
    method: "GET",
    params: query,
  });
}

export function attachContent(
  payload: CreateProjectContentInput,
): Promise<ProjectContentItem> {
  return request<ProjectContentItem>({
    url: "/project-contents",
    method: "POST",
    data: payload,
  });
}

export function updateProjectContent(
  id: string,
  payload: UpdateProjectContentInput,
): Promise<ProjectContentItem> {
  return request<ProjectContentItem>({
    url: `/project-contents/${id}`,
    method: "PATCH",
    data: payload,
  });
}

export function detachContent(id: string): Promise<null> {
  return request<null>({
    url: `/project-contents/${id}`,
    method: "DELETE",
  });
}
