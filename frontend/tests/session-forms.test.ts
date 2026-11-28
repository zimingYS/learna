import {describe, expect, it} from "vitest";
import type {SessionDetailItem} from "../src/api/sessions.ts";
import {
  buildContentItems,
  buildEvidencePayload,
  buildProjectIds,
  buildSessionCreatePayload,
  buildSessionUpdatePayload,
  buildTaskItems,
  emptySessionForm,
  sessionToForm,
} from "../src/views/sessions/session-forms.ts";

function localInput(hour: number, minute = 0, day = 28): string {
  const pad = (value: number) => String(value).padStart(2, "0");

  return `2026-11-${pad(day)}T${pad(hour)}:${pad(minute)}`;
}

function detail(overrides: Partial<SessionDetailItem> = {}): SessionDetailItem {
  return {
    id: "s-1",
    title: "高数第一轮",
    activity_type: "mixed",
    started_at: new Date(2026, 10, 28, 9, 0).toISOString(),
    ended_at: new Date(2026, 10, 28, 11, 0).toISOString(),
    duration_minutes: 90,
    primary_project_id: "p-1",
    created_source: "user",
    created_at: new Date(2026, 10, 28, 11, 5).toISOString(),
    summary: "函数与极限",
    what_done: null,
    problems: null,
    next_steps: null,
    related_project_ids: ["p-2"],
    contents: [],
    tasks: [],
    ...overrides,
  };
}

describe("创建活动请求体", () => {
  it("时长留空就不发该字段，交给后端按区间生成", () => {
    const built = buildSessionCreatePayload({
      ...emptySessionForm(),
      startedAt: localInput(9),
      endedAt: localInput(11),
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value.duration_minutes).toBeUndefined();
      expect(built.value.started_at).toBe(new Date(2026, 10, 28, 9, 0).toISOString());
      expect(built.value.activity_type).toBe("mixed");
    }
  });

  it("上报时长超过区间时直接拦住，不白跑一次请求", () => {
    const built = buildSessionCreatePayload({
      ...emptySessionForm(),
      startedAt: localInput(9),
      endedAt: localInput(10),
      durationMinutes: "90",
    });

    expect(built.ok).toBe(false);
  });

  it("结束早于开始时报错", () => {
    const built = buildSessionCreatePayload({
      ...emptySessionForm(),
      startedAt: localInput(11),
      endedAt: localInput(9),
    });

    expect(built.ok).toBe(false);
  });

  it("标题与四段式都做 trim，空串不进请求体", () => {
    const built = buildSessionCreatePayload({
      ...emptySessionForm(),
      startedAt: localInput(9),
      endedAt: localInput(10),
      title: "  高数  ",
      summary: "   ",
      whatDone: " 做了 12 道题 ",
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value.title).toBe("高数");
      expect(built.value.summary).toBeUndefined();
      expect(built.value.what_done).toBe("做了 12 道题");
    }
  });

  it("时长必须是非负整数", () => {
    const built = buildSessionCreatePayload({
      ...emptySessionForm(),
      startedAt: localInput(9),
      endedAt: localInput(11),
      durationMinutes: "-5",
    });

    expect(built.ok).toBe(false);
  });
});

describe("更新活动请求体（三态）", () => {
  it("没改任何字段就提交空对象，避免无谓的 updated_at 变更", () => {
    const current = detail();
    const built = buildSessionUpdatePayload(current, sessionToForm(current));

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value).toEqual({});
    }
  });

  it("清空描述发显式 null，而不是省略字段", () => {
    const current = detail();
    const built = buildSessionUpdatePayload(current, {...sessionToForm(current), summary: ""});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value).toEqual({summary: null});
    }
  });

  it("只改标题时只带标题", () => {
    const current = detail();
    const built = buildSessionUpdatePayload(current, {...sessionToForm(current), title: "改名"});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value).toEqual({title: "改名"});
    }
  });

  it("主要项目清空发 null，改成别的项目发新 id", () => {
    const current = detail();
    const cleared = buildSessionUpdatePayload(current, {
      ...sessionToForm(current),
      primaryProjectId: "",
    });
    const moved = buildSessionUpdatePayload(current, {
      ...sessionToForm(current),
      primaryProjectId: "p-9",
    });

    expect(cleared.ok && cleared.value).toEqual({primary_project_id: null});
    expect(moved.ok && moved.value).toEqual({primary_project_id: "p-9"});
  });

  it("改区间到比原时长更窄而不改时长时报错", () => {
    const current = detail();
    const built = buildSessionUpdatePayload(current, {
      ...sessionToForm(current),
      endedAt: localInput(10),
    });

    expect(built.ok).toBe(false);
  });
});

describe("关联行校验", () => {
  it("重复内容直接拦住", () => {
    const built = buildContentItems([
      {contentId: "c-1", contributionType: "covered", note: ""},
      {contentId: "c-1", contributionType: "practiced", note: ""},
    ]);

    expect(built.ok).toBe(false);
  });

  it("空备注归一为 null", () => {
    const built = buildContentItems([{contentId: "c-1", contributionType: "covered", note: "  "}]);

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value[0].note).toBeNull();
    }
  });

  it("分摊合计超过有效时长时报错，等于时长则通过", () => {
    const rows = [
      {taskId: "t-1", minutes: "30", note: ""},
      {taskId: "t-2", minutes: "30", note: ""},
    ];

    expect(buildTaskItems(rows, 60).ok).toBe(true);
    expect(buildTaskItems(rows, 59).ok).toBe(false);
  });

  it("重复任务与负数分摊都拦住", () => {
    expect(
      buildTaskItems(
        [
          {taskId: "t-1", minutes: "10", note: ""},
          {taskId: "t-1", minutes: "10", note: ""},
        ],
        60,
      ).ok,
    ).toBe(false);

    expect(buildTaskItems([{taskId: "t-1", minutes: "-1", note: ""}], 60).ok).toBe(false);
  });

  it("主要项目不能同时出现在关联项目里", () => {
    expect(buildProjectIds(["p-1"], "p-1").ok).toBe(false);
    expect(buildProjectIds(["p-2"], "p-1").ok).toBe(true);
    expect(buildProjectIds([], "").ok).toBe(true);
  });
});

describe("证据请求体", () => {
  it("笔记缺描述被拦住", () => {
    const built = buildEvidencePayload({
      sessionId: "",
      type: "note",
      title: "",
      description: "  ",
      externalUrl: "",
    });

    expect(built.ok).toBe(false);
  });

  it("链接缺地址或被非 http(s) 协议拦住", () => {
    const missing = buildEvidencePayload({
      sessionId: "",
      type: "link",
      title: "题解",
      description: "",
      externalUrl: "",
    });
    const unsafe = buildEvidencePayload({
      sessionId: "",
      type: "link",
      title: "题解",
      description: "",
      externalUrl: "javascript:alert(1)",
    });

    expect(missing.ok).toBe(false);
    expect(unsafe.ok).toBe(false);
  });

  it("合法链接只带必要字段，未选活动就不发 session_id", () => {
    const built = buildEvidencePayload({
      sessionId: "",
      type: "link",
      title: "题解",
      description: "",
      externalUrl: "https://example.com/a",
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.value).toEqual({
        evidence_type: "link",
        title: "题解",
        external_url: "https://example.com/a",
      });
    }
  });

  it("选中活动时带上 session_id", () => {
    const built = buildEvidencePayload({
      sessionId: "s-1",
      type: "test_result",
      title: "",
      description: "10 题错 2 题",
      externalUrl: "",
    });

    expect(built.ok && built.value).toEqual({
      evidence_type: "test_result",
      session_id: "s-1",
      description: "10 题错 2 题",
    });
  });
});
