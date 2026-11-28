import {beforeEach, describe, expect, it, vi} from "vitest";
import {
  attachContent,
  createContent,
  createRelation,
  createTag,
  deleteContent,
  deleteRelation,
  deleteTag,
  detachContent,
  fetchContent,
  fetchContentTags,
  fetchContents,
  fetchProjectContents,
  fetchRelations,
  fetchTags,
  setContentTags,
  updateContent,
  updateProjectContent,
  updateTag,
} from "../src/api/contents.ts";

const {requestMock} = vi.hoisted(() => ({requestMock: vi.fn()}));

vi.mock("../src/api/http.ts", () => ({
  describeError: vi.fn(),
  request: requestMock,
}));

beforeEach(() => {
  requestMock.mockReset();
  requestMock.mockResolvedValue({items: [], total: 0});
});

describe("内容本体端点契约", () => {
  it("列表把筛选条件原样带给后端", async () => {
    await fetchContents({page: 2, page_size: 12, status: "active", root_only: true, q: "极限"});

    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents",
      method: "GET",
      params: {page: 2, page_size: 12, status: "active", root_only: true, q: "极限"},
    });
  });

  it("创建走 POST，更新走 PATCH，详情走 GET", async () => {
    await createContent({title: "极限的定义", content_type: "knowledge_point"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents",
      method: "POST",
      data: {title: "极限的定义", content_type: "knowledge_point"},
    });

    await updateContent("c-1", {title: "改名"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1",
      method: "PATCH",
      data: {title: "改名"},
    });

    await fetchContent("c-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1",
      method: "GET",
    });
  });

  it("删除默认归档，purge=true 才是物理删除", async () => {
    await deleteContent("c-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1",
      method: "DELETE",
      params: undefined,
    });

    await deleteContent("c-1", true);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1",
      method: "DELETE",
      params: {purge: true},
    });
  });
});

describe("内容关系端点契约", () => {
  it("列表可带方向与类型，创建只提交目标与类型", async () => {
    await fetchRelations("c-1", {direction: "all", relation_type: "prerequisite"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1/relations",
      method: "GET",
      params: {direction: "all", relation_type: "prerequisite"},
    });

    await createRelation("c-1", {target_content_id: "c-2", relation_type: "related"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1/relations",
      method: "POST",
      data: {target_content_id: "c-2", relation_type: "related"},
    });

    await deleteRelation("c-1", "r-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1/relations/r-1",
      method: "DELETE",
    });
  });
});

describe("标签端点契约", () => {
  it("内容标签是整体替换语义", async () => {
    await fetchContentTags("c-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1/tags",
      method: "GET",
    });

    await setContentTags("c-1", ["t-1", "t-2"]);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/learning-contents/c-1/tags",
      method: "PUT",
      data: {tag_ids: ["t-1", "t-2"]},
    });
  });

  it("词表 CRUD 各自命中独立前缀", async () => {
    await fetchTags({page: 1, page_size: 100});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/content-tags",
      method: "GET",
      params: {page: 1, page_size: 100},
    });

    await createTag({name: "高频考点", color: "#7d7cf8"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/content-tags",
      method: "POST",
      data: {name: "高频考点", color: "#7d7cf8"},
    });

    await updateTag("t-1", {name: "低频考点"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/content-tags/t-1",
      method: "PATCH",
      data: {name: "低频考点"},
    });

    await deleteTag("t-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/content-tags/t-1",
      method: "DELETE",
    });
  });
});

describe("项目内容端点契约", () => {
  it("按内容查跨项目复用，关联用 POST 幂等提交", async () => {
    await fetchProjectContents({content_id: "c-1", page_size: 100});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/project-contents",
      method: "GET",
      params: {content_id: "c-1", page_size: 100},
    });

    await attachContent({project_id: "p-1", content_id: "c-1"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/project-contents",
      method: "POST",
      data: {project_id: "p-1", content_id: "c-1"},
    });
  });

  it("更新项目内状态走 PATCH，解除关联走 DELETE", async () => {
    await updateProjectContent("pc-1", {current_status: "learning", priority: 4});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/project-contents/pc-1",
      method: "PATCH",
      data: {current_status: "learning", priority: 4},
    });

    await detachContent("pc-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/project-contents/pc-1",
      method: "DELETE",
    });
  });
});
