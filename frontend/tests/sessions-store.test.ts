import {createPinia, setActivePinia} from "pinia";
import {afterEach, beforeEach, describe, expect, it, vi} from "vitest";
import {ApiError} from "../src/api/http.ts";
import type {EvidenceItem, SessionDetailItem, SessionItem} from "../src/api/sessions.ts";
import {useSessionsStore} from "../src/stores/sessions.ts";

const {
  createEvidenceMock,
  createSessionMock,
  deleteEvidenceMock,
  deleteSessionMock,
  fetchContentsMock,
  fetchEvidenceMock,
  fetchProjectsMock,
  fetchSessionMock,
  fetchSessionsMock,
  fetchTasksMock,
  setSessionContentsMock,
  setSessionProjectsMock,
  setSessionTasksMock,
  updateSessionMock,
} = vi.hoisted(() => ({
  createEvidenceMock: vi.fn(),
  createSessionMock: vi.fn(),
  deleteEvidenceMock: vi.fn(),
  deleteSessionMock: vi.fn(),
  fetchContentsMock: vi.fn(),
  fetchEvidenceMock: vi.fn(),
  fetchProjectsMock: vi.fn(),
  fetchSessionMock: vi.fn(),
  fetchSessionsMock: vi.fn(),
  fetchTasksMock: vi.fn(),
  setSessionContentsMock: vi.fn(),
  setSessionProjectsMock: vi.fn(),
  setSessionTasksMock: vi.fn(),
  updateSessionMock: vi.fn(),
}));

vi.mock("../src/api/sessions.ts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/api/sessions.ts")>();

  return {
    ...actual,
    createEvidence: createEvidenceMock,
    createSession: createSessionMock,
    deleteEvidence: deleteEvidenceMock,
    deleteSession: deleteSessionMock,
    fetchEvidence: fetchEvidenceMock,
    fetchSession: fetchSessionMock,
    fetchSessions: fetchSessionsMock,
    setSessionContents: setSessionContentsMock,
    setSessionProjects: setSessionProjectsMock,
    setSessionTasks: setSessionTasksMock,
    updateSession: updateSessionMock,
  };
});

vi.mock("../src/api/projects.ts", () => ({fetchProjects: fetchProjectsMock}));
vi.mock("../src/api/contents.ts", () => ({fetchContents: fetchContentsMock}));
vi.mock("../src/api/planning.ts", () => ({fetchTasks: fetchTasksMock}));

const item: SessionItem = {
  id: "s-1",
  title: "高数第一轮",
  activity_type: "practice",
  started_at: "2026-11-28T01:00:00Z",
  ended_at: "2026-11-28T02:30:00Z",
  duration_minutes: 90,
  primary_project_id: "p-1",
  created_source: "user",
  created_at: "2026-11-28T02:35:00Z",
};

const detail: SessionDetailItem = {
  ...item,
  summary: "函数与极限",
  what_done: null,
  problems: null,
  next_steps: null,
  related_project_ids: ["p-2"],
  contents: [],
  tasks: [],
};

const evidence: EvidenceItem = {
  id: "e-1",
  session_id: "s-1",
  evidence_type: "note",
  title: null,
  description: "错题摘录",
  external_url: null,
  created_at: "2026-11-28T02:40:00Z",
};

beforeEach(() => {
  setActivePinia(createPinia());

  fetchSessionsMock.mockReset().mockResolvedValue({items: [item], total: 1});
  fetchSessionMock.mockReset().mockResolvedValue(detail);
  fetchEvidenceMock.mockReset().mockResolvedValue({items: [evidence], total: 1});
  fetchProjectsMock.mockReset().mockResolvedValue({items: [], total: 0});
  fetchContentsMock.mockReset().mockResolvedValue({items: [], total: 0});
  fetchTasksMock.mockReset().mockResolvedValue({items: [], total: 0});
  createSessionMock.mockReset().mockResolvedValue(detail);
  updateSessionMock.mockReset().mockResolvedValue(detail);
  deleteSessionMock.mockReset().mockResolvedValue(undefined);
  setSessionContentsMock.mockReset().mockResolvedValue({items: [], total: 0});
  setSessionProjectsMock.mockReset().mockResolvedValue({project_ids: []});
  setSessionTasksMock.mockReset().mockResolvedValue({items: [], total: 0});
  createEvidenceMock.mockReset().mockResolvedValue(evidence);
  deleteEvidenceMock.mockReset().mockResolvedValue(undefined);
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("列表筛选", () => {
  it("类型与项目筛选进请求，日期转成半开区间", async () => {
    const store = useSessionsStore();

    store.typeFilter = "practice";
    store.projectFilter = "p-1";
    store.fromDate = "2026-11-01";
    store.toDate = "2026-11-30";
    await store.load();

    const query = fetchSessionsMock.mock.calls[0][0];

    expect(query).toMatchObject({
      page: 1,
      page_size: 10,
      activity_type: "practice",
      primary_project_id: "p-1",
    });
    // 本地自然日边界换算成 UTC 后不一定是同一天，所以对照同一个换算
    expect(query.from).toBe(new Date("2026-11-01T00:00:00").toISOString());
    expect(query.to).toBe(new Date("2026-11-30T23:59:59").toISOString());
    expect(store.items).toHaveLength(1);
    expect(store.total).toBe(1);
  });

  it("分页被夹在有效范围内", async () => {
    const store = useSessionsStore();

    await store.load();
    await store.setPage(5);

    expect(store.page).toBe(1);
  });
});

describe("详情与证据", () => {
  it("选中活动后证据按活动过滤", async () => {
    const store = useSessionsStore();

    await store.select("s-1");

    expect(store.selected?.id).toBe("s-1");
    expect(fetchEvidenceMock).toHaveBeenLastCalledWith(
      expect.objectContaining({session_id: "s-1"}),
    );
  });

  it("取消选中后证据回到全部（含独立证据）", async () => {
    const store = useSessionsStore();

    await store.select("s-1");
    store.clearSelection();
    await Promise.resolve();

    expect(store.selected).toBeNull();
    expect(fetchEvidenceMock).toHaveBeenLastCalledWith(
      expect.objectContaining({session_id: undefined}),
    );
  });
});

describe("增删改", () => {
  it("创建成功后直接选中新记录并刷新列表", async () => {
    const store = useSessionsStore();

    const ok = await store.create({
      started_at: "2026-11-28T01:00:00Z",
      ended_at: "2026-11-28T02:00:00Z",
    });

    expect(ok).toBe(true);
    expect(store.selected?.id).toBe("s-1");
    expect(fetchSessionsMock).toHaveBeenCalled();
  });

  it("删除当前选中的活动后清空详情", async () => {
    const store = useSessionsStore();

    await store.select("s-1");
    const ok = await store.remove("s-1");

    expect(ok).toBe(true);
    expect(store.selected).toBeNull();
    expect(deleteSessionMock).toHaveBeenCalledWith("s-1");
  });

  it("关联替换后回读详情，前端不自行拼装服务端状态", async () => {
    const store = useSessionsStore();

    await store.select("s-1");
    fetchSessionMock.mockClear();

    await store.replaceProjects(["p-2"]);

    expect(setSessionProjectsMock).toHaveBeenCalledWith("s-1", ["p-2"]);
    expect(fetchSessionMock).toHaveBeenCalledWith("s-1");
  });

  it("接口报错时落到 error 而不是抛出", async () => {
    const store = useSessionsStore();

    await store.select("s-1");
    setSessionTasksMock.mockRejectedValueOnce(new ApiError("任务不属于本次活动的关联项目"));

    const ok = await store.replaceTasks([]);

    expect(ok).toBe(false);
    expect(store.error).toContain("任务不属于本次活动的关联项目");
  });
});

describe("客户端计时（ADR-010 §2）", () => {
  it("开始 → 暂停 → 继续 → 结束，只累计运行时段", () => {
    const store = useSessionsStore();
    const now = vi.spyOn(Date, "now");

    now.mockReturnValue(1_000);
    store.startTimer();

    now.mockReturnValue(61_000);
    expect(store.timerElapsedMs(61_000)).toBe(60_000);

    // 暂停后时间继续走，但累计值不再增长
    store.pauseTimer();
    now.mockReturnValue(121_000);
    expect(store.timerElapsedMs(121_000)).toBe(60_000);

    // 继续 30 秒后结束：共 90 秒
    store.startTimer();
    now.mockReturnValue(151_000);

    const snapshot = store.timerSnapshot(151_000);

    expect(snapshot).not.toBeNull();
    expect(snapshot?.durationMinutes).toBe(1);
    expect(snapshot?.startedAt).toBe(new Date(1_000).toISOString());
    expect(snapshot?.endedAt).toBe(new Date(151_000).toISOString());
    // 快照只暂停不重置：调用方若取消表单，计时还能继续
    expect(store.timerRunning).toBe(false);

    store.resetTimer();
    expect(store.timerStarted).toBe(false);
    expect(store.timerElapsedMs(200_000)).toBe(0);
  });

  it("不足一分钟按 1 分钟计，避免出现 0 分钟记录", () => {
    const store = useSessionsStore();
    const now = vi.spyOn(Date, "now");

    now.mockReturnValue(0);
    store.startTimer();
    now.mockReturnValue(5_000);

    expect(store.timerSnapshot(5_000)?.durationMinutes).toBe(1);
  });

  it("没开始计时就没有快照", () => {
    const store = useSessionsStore();

    expect(store.timerSnapshot(1_000)).toBeNull();
  });
});
