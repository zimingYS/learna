import type {
  BodyFormat,
  ContentDetailItem,
  ContentType,
  CreateContentInput,
  UpdateContentInput,
} from "../../api/contents";

// 上限与 backend modules/content/rules.rs 逐条对齐
export const TITLE_MAX = 200;
export const DESCRIPTION_MAX = 500;
export const BODY_MAX = 100_000;
export const SOURCE_URL_MAX = 2048;

export type BuildPayloadResult<T> =
  | { ok: true; payload: T }
  | { ok: false; error: string };

export interface ContentFormFields {
  title: string;
  contentType: ContentType;
  description: string;
  body: string;
  bodyFormat: BodyFormat;
  sourceUrl: string;
  parentId: string;
}

export function emptyContentForm(parentId = ""): ContentFormFields {
  return {
    title: "",
    contentType: "knowledge_point",
    description: "",
    body: "",
    bodyFormat: "markdown",
    sourceUrl: "",
    parentId,
  };
}

export function contentToForm(item: ContentDetailItem): ContentFormFields {
  return {
    title: item.title,
    contentType: item.content_type,
    description: item.description ?? "",
    body: item.body ?? "",
    bodyFormat: item.body_format,
    sourceUrl: item.source_url ?? "",
    parentId: item.parent_id ?? "",
  };
}

function trimmedOrNull(value: string): string | null {
  const trimmed = value.trim();

  return trimmed ? trimmed : null;
}

/** 与后端一致：空串归一为 null，避免 `''` 与 `NULL` 两种空并存（ADR-009 §1） */
function normalizeBody(value: string): string | null {
  return value.trim() ? value : null;
}

function isHttpUrl(value: string): boolean {
  const lower = value.toLowerCase();

  return (
    (lower.startsWith("http://") && value.length > "http://".length) ||
    (lower.startsWith("https://") && value.length > "https://".length)
  );
}

interface NormalizedFields {
  title: string;
  contentType: ContentType;
  description: string | null;
  body: string | null;
  bodyFormat: BodyFormat;
  sourceUrl: string | null;
  parentId: string | null;
}

function normalizeFields(
  fields: ContentFormFields,
): BuildPayloadResult<NormalizedFields> {
  const title = fields.title.trim();

  if (!title) {
    return { ok: false, error: "请输入内容标题" };
  }

  if (title.length > TITLE_MAX) {
    return { ok: false, error: `标题不能超过 ${TITLE_MAX} 个字符` };
  }

  const description = trimmedOrNull(fields.description);

  if (description !== null && description.length > DESCRIPTION_MAX) {
    return { ok: false, error: `摘要不能超过 ${DESCRIPTION_MAX} 个字符` };
  }

  const body = normalizeBody(fields.body);

  if (body !== null && body.length > BODY_MAX) {
    return { ok: false, error: `正文不能超过 ${BODY_MAX} 个字符` };
  }

  const sourceUrl = trimmedOrNull(fields.sourceUrl);

  if (sourceUrl !== null) {
    if (sourceUrl.length > SOURCE_URL_MAX) {
      return { ok: false, error: `出处链接不能超过 ${SOURCE_URL_MAX} 个字符` };
    }

    if (!isHttpUrl(sourceUrl)) {
      return { ok: false, error: "出处链接仅支持 http / https 地址" };
    }
  }

  return {
    ok: true,
    payload: {
      title,
      contentType: fields.contentType,
      description,
      body,
      bodyFormat: fields.bodyFormat,
      sourceUrl,
      parentId: fields.parentId || null,
    },
  };
}

export function buildContentCreatePayload(
  fields: ContentFormFields,
): BuildPayloadResult<CreateContentInput> {
  const normalized = normalizeFields(fields);

  if (!normalized.ok) {
    return normalized;
  }

  const value = normalized.payload;

  return {
    ok: true,
    payload: {
      title: value.title,
      content_type: value.contentType,
      description: value.description,
      body: value.body,
      body_format: value.bodyFormat,
      source_url: value.sourceUrl,
      parent_id: value.parentId,
    },
  };
}

/**
 * 只提交真正变化的字段（ADR-002 三态）：未变化的键不出现，
 * 否则每次保存都会把 `updated_at` 往前推。
 */
export function buildContentUpdatePayload(
  item: ContentDetailItem,
  fields: ContentFormFields,
): BuildPayloadResult<UpdateContentInput> {
  const normalized = normalizeFields(fields);

  if (!normalized.ok) {
    return normalized;
  }

  const value = normalized.payload;
  const payload: UpdateContentInput = {};

  if (value.title !== item.title) {
    payload.title = value.title;
  }

  if (value.contentType !== item.content_type) {
    payload.content_type = value.contentType;
  }

  if (value.description !== (item.description ?? null)) {
    payload.description = value.description;
  }

  if (value.body !== (item.body ?? null)) {
    payload.body = value.body;
  }

  if (value.bodyFormat !== item.body_format) {
    payload.body_format = value.bodyFormat;
  }

  if (value.sourceUrl !== (item.source_url ?? null)) {
    payload.source_url = value.sourceUrl;
  }

  if (value.parentId !== (item.parent_id ?? null)) {
    payload.parent_id = value.parentId;
  }

  return { ok: true, payload };
}
