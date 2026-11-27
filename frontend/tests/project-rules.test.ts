import {describe, expect, it} from "vitest";
import type {ProjectItem} from "../src/api/projects.ts";
import {
  availableActions,
  buildCreatePayload,
  buildUpdatePayload,
  daysUntil,
  emptyProjectForm,
  formatDateTime,
  isDeadlineEditable,
  projectToForm,
  toLocalInput,
} from "../src/views/projects/project-rules.ts";

function makeProject(overrides: Partial<ProjectItem> = {}): ProjectItem {
  return {
    id: "p-1",
    name: "考研数学",
    description: "强化阶段",
    project_type: "study",
    status: "active",
    importance_level: 3,
    importance_override_until: null,
    deadline_type: "soft",
    deadline_at: "2027-06-01T12:00:00.000Z",
    archived_at: null,
    reopened_at: null,
    created_at: "2026-11-01T08:00:00.000Z",
    updated_at: "2026-11-20T08:00:00.000Z",
    ...overrides,
  };
}

describe("项目生命周期动作可用性", () => {
  it("与后端迁移矩阵一致", () => {
    expect(availableActions("draft")).toEqual(["activate", "archive"]);
    expect(availableActions("active")).toEqual(["pause", "complete", "archive"]);
    expect(availableActions("paused")).toEqual(["resume", "complete", "archive"]);
    expect(availableActions("completed")).toEqual(["reopen", "archive"]);
    expect(availableActions("archived")).toEqual(["reopen"]);
  });

  it("固定截止日期不可经编辑表单修改", () => {
    expect(isDeadlineEditable(makeProject({deadline_type: "fixed"}))).toBe(false);
    expect(isDeadlineEditable(makeProject({deadline_type: "soft"}))).toBe(true);
  });
});

describe("创建项目载荷", () => {
  it("默认值走契约默认，无截止时不发 deadline_at", () => {
    const result = buildCreatePayload({...emptyProjectForm(), name: "  考研 408  "});

    expect(result).toEqual({
      ok: true,
      payload: {name: "考研 408", project_type: "study", importance_level: 3, deadline_type: "none"},
    });
  });

  it("名称为空或超长时拒绝", () => {
    expect(buildCreatePayload(emptyProjectForm())).toEqual({ok: false, error: "请输入项目名称"});
    expect(buildCreatePayload({...emptyProjectForm(), name: "x".repeat(161)}).ok).toBe(false);
  });

  it("soft / fixed 必须带截止时间", () => {
    const result = buildCreatePayload({
      ...emptyProjectForm(),
      name: "软截止",
      deadlineType: "soft",
    });

    expect(result).toEqual({ok: false, error: "请选择截止日期时间"});
  });

  it("把 datetime-local 转成 RFC3339", () => {
    const result = buildCreatePayload({
      ...emptyProjectForm(),
      name: "固定截止",
      deadlineType: "fixed",
      deadlineAt: "2027-12-26T08:30",
    });

    expect(result.ok).toBe(true);

    if (result.ok) {
      expect(result.payload.deadline_at).toBe(new Date("2027-12-26T08:30").toISOString());
    }
  });

  it("重要程度越界被拦下", () => {
    expect(buildCreatePayload({...emptyProjectForm(), name: "越界", importanceLevel: "7"})).toEqual({
      ok: false,
      error: "重要程度必须是 1~5",
    });
  });
});

describe("更新项目载荷", () => {
  it("原样提交时报告没有修改", () => {
    const project = makeProject();

    expect(buildUpdatePayload(project, projectToForm(project))).toEqual({
      ok: false,
      error: "没有需要保存的修改",
    });
  });

  it("清空描述用 null 表达置空", () => {
    const project = makeProject({description: "旧描述"});
    const result = buildUpdatePayload(project, {...projectToForm(project), description: ""});

    expect(result).toEqual({ok: true, payload: {description: null}});
  });

  it("切换为无截止时同时下发 none 与 null", () => {
    const project = makeProject();
    const fields = {...projectToForm(project), deadlineType: "none" as const};

    expect(buildUpdatePayload(project, fields)).toEqual({
      ok: true,
      payload: {deadline_type: "none", deadline_at: null},
    });
  });

  it("固定截止日期的项目禁止在表单里改期", () => {
    const project = makeProject({deadline_type: "fixed"});
    const fields = {...projectToForm(project), deadlineAt: "2028-01-01T00:00"};

    expect(buildUpdatePayload(project, fields)).toEqual({
      ok: false,
      error: "固定截止日期的项目请使用「修改截止日期」",
    });
  });

  it("只改重要程度时仅提交该字段", () => {
    const project = makeProject();
    const result = buildUpdatePayload(project, {...projectToForm(project), importanceLevel: "5"});

    expect(result).toEqual({ok: true, payload: {importance_level: 5}});
  });
});

describe("时间展示", () => {
  it("datetime-local 往返不丢分钟", () => {
    const project = makeProject({deadline_at: "2027-06-01T12:34:00.000Z"});
    const local = toLocalInput(project.deadline_at);

    // 断言往返等价而非字面值：本地时区不同，格式化的钟点也不同
    expect(new Date(local).getTime()).toBe(new Date(project.deadline_at).getTime());
    expect(local).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
  });

  it("非法时间显示占位符", () => {
    expect(formatDateTime("not-a-date")).toBe("—");
    expect(formatDateTime(null)).toBe("—");
  });

  it("剩余天数按自然日计算", () => {
    const now = new Date();
    const atNoon = (offset: number): string =>
      new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset, 12).toISOString();

    expect(daysUntil(atNoon(0))).toBe(0);
    expect(daysUntil(atNoon(3))).toBe(3);
    expect(daysUntil(atNoon(-2))).toBe(-2);
    expect(daysUntil(null)).toBeNull();
  });
});
