import {beforeEach, describe, expect, it, vi} from "vitest";
import {
  createEvidence,
  createSession,
  deleteEvidence,
  deleteSession,
  fetchEvidence,
  fetchEvidenceItem,
  fetchSession,
  fetchSessions,
  setSessionContents,
  setSessionProjects,
  setSessionTasks,
  updateSession,
} from "../src/api/sessions.ts";

const {requestMock} = vi.hoisted(() => ({requestMock: vi.fn()}));

vi.mock("../src/api/http.ts", () => ({
  describeError: vi.fn(),
  request: requestMock,
}));

beforeEach(() => {
  requestMock.mockReset();
  requestMock.mockResolvedValue({items: [], total: 0});
});

describe("学习活动端点契约", () => {
  it("列表把筛选条件原样带给后端", async () => {
    await fetchSessions({
      page: 2,
      page_size: 10,
      primary_project_id: "p-1",
      content_id: "c-1",
      activity_type: "review",
      from: "2026-11-01T00:00:00.000Z",
      to: "2026-11-30T23:59:59.000Z",
    });

    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions",
      method: "GET",
      params: {
        page: 2,
        page_size: 10,
        primary_project_id: "p-1",
        content_id: "c-1",
        activity_type: "review",
        from: "2026-11-01T00:00:00.000Z",
        to: "2026-11-30T23:59:59.000Z",
      },
    });
  });

  it("创建走 POST、更新走 PATCH、详情走 GET、删除走 DELETE", async () => {
    await createSession({
      started_at: "2026-11-28T09:00:00.000Z",
      ended_at: "2026-11-28T10:00:00.000Z",
      duration_minutes: 45,
    });
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions",
      method: "POST",
      data: {
        started_at: "2026-11-28T09:00:00.000Z",
        ended_at: "2026-11-28T10:00:00.000Z",
        duration_minutes: 45,
      },
    });

    await fetchSession("s-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1",
      method: "GET",
    });

    await updateSession("s-1", {summary: null});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1",
      method: "PATCH",
      data: {summary: null},
    });

    await deleteSession("s-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1",
      method: "DELETE",
    });
  });

  it("三类关联都走 PUT 且只带替换后的集合", async () => {
    await setSessionContents("s-1", [
      {content_id: "c-1", contribution_type: "practiced", note: null},
    ]);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1/contents",
      method: "PUT",
      data: {items: [{content_id: "c-1", contribution_type: "practiced", note: null}]},
    });

    await setSessionProjects("s-1", ["p-1", "p-2"]);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1/projects",
      method: "PUT",
      data: {project_ids: ["p-1", "p-2"]},
    });

    await setSessionTasks("s-1", [{task_id: "t-1", contribution_minutes: 30, note: null}]);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-sessions/s-1/tasks",
      method: "PUT",
      data: {items: [{task_id: "t-1", contribution_minutes: 30, note: null}]},
    });
  });

  it("没有服务端计时端点（ADR-010 §2）", async () => {
    const api = await import("../src/api/sessions.ts");

    expect(Object.keys(api).some((name) => /start|pause|resume|stop/i.test(name))).toBe(false);
  });
});

describe("证据端点契约", () => {
  it("列表带 session_id 与类型过滤", async () => {
    await fetchEvidence({page: 1, page_size: 50, session_id: "s-1", evidence_type: "note"});

    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/evidence",
      method: "GET",
      params: {page: 1, page_size: 50, session_id: "s-1", evidence_type: "note"},
    });
  });

  it("创建、详情、删除对应 POST / GET / DELETE", async () => {
    await createEvidence({evidence_type: "note", description: "错题摘录"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/evidence",
      method: "POST",
      data: {evidence_type: "note", description: "错题摘录"},
    });

    await fetchEvidenceItem("e-1");
    expect(requestMock).toHaveBeenLastCalledWith({url: "/evidence/e-1", method: "GET"});

    await deleteEvidence("e-1");
    expect(requestMock).toHaveBeenLastCalledWith({url: "/evidence/e-1", method: "DELETE"});
  });
});
