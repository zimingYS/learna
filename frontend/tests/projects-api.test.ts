import {beforeEach, describe, expect, it, vi} from "vitest";
import {
  applyProjectAction,
  createProject,
  deleteProject,
  extendProjectDeadline,
  fetchProject,
  fetchProjects,
  purgeProject,
  updateProject,
} from "../src/api/projects.ts";

const {requestMock} = vi.hoisted(() => ({requestMock: vi.fn()}));

vi.mock("../src/api/http.ts", () => ({
  describeError: vi.fn(),
  request: requestMock,
}));

beforeEach(() => {
  requestMock.mockReset();
});

describe("projects API 请求契约", () => {
  it("列表把筛选条件映射为 snake_case 查询参数", async () => {
    requestMock.mockResolvedValue({items: [], total: 0});

    await fetchProjects({page: 2, page_size: 12, status: "active", include_archived: true});

    expect(requestMock).toHaveBeenCalledWith({
      url: "/projects",
      method: "GET",
      params: {page: 2, page_size: 12, status: "active", include_archived: true},
    });
  });

  it("详情与更新都把项目 ID 插入路径", async () => {
    requestMock.mockResolvedValue({id: "p-1"});

    await fetchProject("p-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1",
      method: "GET",
    });

    await updateProject("p-1", {name: "改名"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1",
      method: "PATCH",
      data: {name: "改名"},
    });
  });

  it("创建项目 POST 到集合端点", async () => {
    requestMock.mockResolvedValue({id: "p-2"});

    const input = {name: "考研数学", importance_level: 4};
    await createProject(input);

    expect(requestMock).toHaveBeenCalledWith({
      url: "/projects",
      method: "POST",
      data: input,
    });
  });

  it("六个生命周期动作共用动作路径", async () => {
    requestMock.mockResolvedValue({id: "p-3"});

    for (const action of ["activate", "pause", "resume", "complete", "reopen", "archive"] as const) {
      await applyProjectAction("p-3", action);

      expect(requestMock).toHaveBeenLastCalledWith({
        url: `/projects/p-3/${action}`,
        method: "POST",
      });
    }
  });

  it("改期请求带 new_deadline_at 与 reason", async () => {
    requestMock.mockResolvedValue({id: "p-4"});

    const input = {new_deadline_at: "2027-06-01T00:00:00.000Z", reason: "考试延期"};
    await extendProjectDeadline("p-4", input);

    expect(requestMock).toHaveBeenCalledWith({
      url: "/projects/p-4/extend-deadline",
      method: "POST",
      data: input,
    });
  });

  it("草稿删除走 DELETE，purge 走 POST 并提交 confirm_name", async () => {
    requestMock.mockResolvedValue(undefined);

    await deleteProject("p-5");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-5",
      method: "DELETE",
    });

    await purgeProject("p-5", "考研数学");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-5/purge",
      method: "POST",
      data: {confirm_name: "考研数学"},
    });
  });
});
