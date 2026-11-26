import {beforeEach, describe, expect, it, vi} from "vitest";
import {
  createRecord,
  deleteRecord,
  fetchRecord,
  fetchRecords,
  updateRecord,
} from "../src/api/records.ts";

const {requestMock} = vi.hoisted(() => ({requestMock: vi.fn()}));

vi.mock("../src/api/http.ts", () => ({
  request: requestMock,
}));

beforeEach(() => {
  requestMock.mockReset();
});

describe("records API 请求契约", () => {
  it("列表使用 snake_case 分页参数", async () => {
    requestMock.mockResolvedValue({items: [], total: 0});

    await fetchRecords(2, 20);

    expect(requestMock).toHaveBeenCalledWith({
      url: "/records",
      method: "GET",
      params: {page: 2, page_size: 20},
    });
  });

  it("详情路径正确插值记录 ID", async () => {
    requestMock.mockResolvedValue({id: "abc-123", title: "x"});

    await fetchRecord("abc-123");

    expect(requestMock).toHaveBeenCalledWith({
      url: "/records/abc-123",
      method: "GET",
    });
  });

  it("创建记录 POST 到列表端点", async () => {
    requestMock.mockResolvedValue({id: "r-1", title: "复习"});

    const input = {title: "复习", duration_minutes: 30};
    await createRecord(input);

    expect(requestMock).toHaveBeenCalledWith({
      url: "/records",
      method: "POST",
      data: input,
    });
  });

  it("更新记录 PUT 到具体资源", async () => {
    requestMock.mockResolvedValue({id: "r-1", title: "强化"});

    await updateRecord("r-1", {title: "强化"});

    expect(requestMock).toHaveBeenCalledWith({
      url: "/records/r-1",
      method: "PUT",
      data: {title: "强化"},
    });
  });

  it("删除记录 DELETE 后不抛出", async () => {
    requestMock.mockResolvedValue(undefined);

    await expect(deleteRecord("r-9")).resolves.toBeUndefined();
    expect(requestMock).toHaveBeenCalledWith({
      url: "/records/r-9",
      method: "DELETE",
    });
  });
});
