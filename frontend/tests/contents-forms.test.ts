import {describe, expect, it} from "vitest";
import {
  buildContentCreatePayload,
  buildContentUpdatePayload,
  type ContentFormFields,
  contentToForm,
  DESCRIPTION_MAX,
  emptyContentForm,
} from "../src/views/contents/contents-forms.ts";

function detail(overrides: Partial<Parameters<typeof buildContentUpdatePayload>[0]> = {}) {
  return {
    id: "c-1",
    title: "极限的定义",
    description: "ε-δ 语言",
    content_type: "knowledge_point" as const,
    parent_id: null,
    source_url: "https://example.com/a",
    status: "active" as const,
    body_format: "markdown" as const,
    created_source: "user" as const,
    created_at: "2026-11-01T10:00:00Z",
    updated_at: "2026-11-01T10:00:00Z",
    body: "# 定义",
    depth: 0,
    ...overrides,
  };
}

describe("内容表单创建载荷", () => {
  it("空摘要与空正文归一为 null，不产生空串", () => {
    const fields: ContentFormFields = {...emptyContentForm(), title: "  函数极限  ", body: "   "};
    const built = buildContentCreatePayload(fields);

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.title).toBe("函数极限");
      expect(built.payload.description).toBeNull();
      expect(built.payload.body).toBeNull();
      expect(built.payload.parent_id).toBeNull();
      expect(built.payload.body_format).toBe("markdown");
    }
  });

  it("标题为空或超长时拒绝提交", () => {
    expect(buildContentCreatePayload(emptyContentForm()).ok).toBe(false);

    const tooLong = {...emptyContentForm(), title: "字".repeat(201)};
    expect(buildContentCreatePayload(tooLong).ok).toBe(false);
  });

  it("摘要超长时拒绝，边界值放行", () => {
    const boundary = {...emptyContentForm(), title: "t", description: "字".repeat(DESCRIPTION_MAX)};
    expect(buildContentCreatePayload(boundary).ok).toBe(true);

    const over = {...emptyContentForm(), title: "t", description: "字".repeat(DESCRIPTION_MAX + 1)};
    expect(buildContentCreatePayload(over).ok).toBe(false);
  });

  it("出处链接只接受 http/https", () => {
    const invalid = {...emptyContentForm(), title: "t", sourceUrl: "ftp://example.com/x"};
    const built = buildContentCreatePayload(invalid);

    expect(built.ok).toBe(false);

    if (!built.ok) {
      expect(built.error).toContain("http");
    }

    const valid = {...emptyContentForm(), title: "t", sourceUrl: "https://example.com/x"};
    expect(buildContentCreatePayload(valid).ok).toBe(true);
  });
});

describe("内容表单更新载荷（ADR-002 三态）", () => {
  it("原样打开不产生任何字段，避免刷新 updated_at", () => {
    const item = detail();
    const built = buildContentUpdatePayload(item, contentToForm(item));

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({});
    }
  });

  it("清空摘要提交 null，而不是空串", () => {
    const item = detail();
    const fields = {...contentToForm(item), description: ""};
    const built = buildContentUpdatePayload(item, fields);

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({description: null});
    }
  });

  it("只提交变化的字段，正文与格式互不牵连", () => {
    const item = detail();
    const fields = {...contentToForm(item), bodyFormat: "plain_text" as const};
    const built = buildContentUpdatePayload(item, fields);

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({body_format: "plain_text"});
    }
  });

  it("上级内容清空表示移到根节点", () => {
    const item = detail({parent_id: "c-0"});
    const built = buildContentUpdatePayload(item, {...contentToForm(item), parentId: ""});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({parent_id: null});
    }
  });
});
