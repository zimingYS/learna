import {createPinia, setActivePinia} from "pinia";
import {beforeEach, describe, expect, it, vi} from "vitest";
import type {ContentDetailItem, ContentItem} from "../src/api/contents.ts";
import {useContentsStore} from "../src/stores/contents.ts";

const {
  createContentMock,
  fetchContentMock,
  fetchContentTagsMock,
  fetchContentsMock,
  fetchProjectContentsMock,
  fetchRelationsMock,
  fetchTagsMock,
} = vi.hoisted(() => ({
  createContentMock: vi.fn(),
  fetchContentMock: vi.fn(),
  fetchContentTagsMock: vi.fn(),
  fetchContentsMock: vi.fn(),
  fetchProjectContentsMock: vi.fn(),
  fetchRelationsMock: vi.fn(),
  fetchTagsMock: vi.fn(),
}));

vi.mock("../src/api/contents.ts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/api/contents.ts")>();

  return {
    ...actual,
    createContent: createContentMock,
    fetchContent: fetchContentMock,
    fetchContentTags: fetchContentTagsMock,
    fetchContents: fetchContentsMock,
    fetchProjectContents: fetchProjectContentsMock,
    fetchRelations: fetchRelationsMock,
    fetchTags: fetchTagsMock,
  };
});

const item: ContentItem = {
  id: "c-1",
  title: "极限的定义",
  description: null,
  content_type: "knowledge_point",
  parent_id: null,
  source_url: null,
  status: "active",
  body_format: "markdown",
  created_source: "user",
  created_at: "2026-11-01T10:00:00Z",
  updated_at: "2026-11-01T10:00:00Z",
};

const detail: ContentDetailItem = {...item, body: "# 定义", depth: 0};

beforeEach(() => {
  setActivePinia(createPinia());

  fetchContentsMock.mockReset().mockResolvedValue({items: [item], total: 1});
  fetchContentMock.mockReset().mockResolvedValue(detail);
  fetchRelationsMock.mockReset().mockResolvedValue({items: [], total: 0});
  fetchContentTagsMock.mockReset().mockResolvedValue({items: [], total: 0});
  fetchProjectContentsMock.mockReset().mockResolvedValue({items: [], total: 0});
  fetchTagsMock.mockReset().mockResolvedValue({items: [], total: 0});
  createContentMock.mockReset();
});

describe("内容树导航", () => {
  it("根层级只发 root_only，进入子级后改发 parent_id（两者互斥）", async () => {
    const store = useContentsStore();

    await store.load();
    expect(fetchContentsMock).toHaveBeenLastCalledWith(
      expect.objectContaining({root_only: true, parent_id: undefined, page: 1}),
    );

    await store.enter(item);
    expect(store.parentId).toBe("c-1");
    expect(store.crumbs.map((crumb) => crumb.id)).toEqual(["c-1"]);
    expect(fetchContentsMock).toHaveBeenLastCalledWith(
      expect.objectContaining({parent_id: "c-1", root_only: undefined}),
    );

    await store.goTo(-1);
    expect(store.parentId).toBeNull();
    expect(store.crumbs).toEqual([]);
    expect(fetchContentsMock).toHaveBeenLastCalledWith(
      expect.objectContaining({root_only: true, parent_id: undefined}),
    );
  });

  it("goTo 截断中间层级时保留上层面包屑", async () => {
    const store = useContentsStore();

    await store.enter(item);
    await store.enter({...item, id: "c-2", title: "数列极限"});
    expect(store.crumbs).toHaveLength(2);

    await store.goTo(0);
    expect(store.crumbs.map((crumb) => crumb.id)).toEqual(["c-1"]);
    expect(store.parentId).toBe("c-1");
  });

  it("切换筛选与翻页都回到第一页重新取数", async () => {
    const store = useContentsStore();

    await store.setPage(3);
    expect(store.page).toBe(3);

    await store.setType("skill_point");
    expect(store.page).toBe(1);
    expect(fetchContentsMock).toHaveBeenLastCalledWith(
      expect.objectContaining({content_type: "skill_point", page: 1}),
    );

    await store.setStatus("all");
    expect(fetchContentsMock).toHaveBeenLastCalledWith(
      expect.objectContaining({status: "all"}),
    );
  });
});

describe("创建与相似提示", () => {
  it("创建后选中新内容，并把相似提示留在 store 上", async () => {
    const store = useContentsStore();
    const hint = {id: "c-9", title: "极限定义", similarity: 0.67};

    createContentMock.mockResolvedValue({item: detail, similar_hints: [hint]});

    const created = await store.create({title: "极限的定义", content_type: "knowledge_point"});

    expect(created?.id).toBe("c-1");
    expect(store.similarHints).toEqual([hint]);
    expect(store.selected?.id).toBe("c-1");

    store.clearHints();
    expect(store.similarHints).toEqual([]);
  });
});

describe("关系标题补齐", () => {
  it("关系另一端不在当前列表里时单独取标题", async () => {
    const store = useContentsStore();

    await store.load();
    fetchRelationsMock.mockResolvedValue({
      items: [
        {
          id: "r-1",
          source_content_id: "c-1",
          target_content_id: "c-9",
          relation_type: "prerequisite",
          direction: "outbound",
        },
      ],
      total: 1,
    });

    await store.loadRelations("c-1");

    expect(fetchContentMock).toHaveBeenCalledWith("c-9");
    expect(store.titles["c-9"]).toBe("极限的定义");
  });
});
