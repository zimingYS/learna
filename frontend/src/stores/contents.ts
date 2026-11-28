import {defineStore} from "pinia";
import {computed, ref} from "vue";
import {
  attachContent,
  createContent,
  createRelation,
  createTag,
  type ContentDetailItem,
  type ContentHint,
  type ContentItem,
  type ContentListQuery,
  type ContentStatus,
  type ContentTagItem,
  type ContentType,
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
  type ProjectContentItem,
  type ProjectContentStatus,
  type RelationItem,
  type RelationType,
  setContentTags,
  type UpdateContentInput,
  updateContent,
  updateProjectContent,
  updateTag,
} from "../api/contents";
import {describeError} from "../api/http";

export const CONTENT_PAGE_SIZE = 12;

// 「全部」只存在于视图层，不是后端枚举值
export type ContentTypeFilter = ContentType | "all";
export type ContentStatusFilter = ContentStatus | "all";

interface Crumb {
  id: string;
  title: string;
}

export const useContentsStore = defineStore("contents", () => {
  const items = ref<ContentItem[]>([]);
  const total = ref(0);
  const page = ref(1);
  const typeFilter = ref<ContentTypeFilter>("all");
  const statusFilter = ref<ContentStatusFilter>("active");
  const keyword = ref("");
  // null 表示当前浏览根层级；非 null 表示浏览该内容的直接子节点
  const parentId = ref<string | null>(null);
  const crumbs = ref<Crumb[]>([]);

  const selected = ref<ContentDetailItem | null>(null);
  const relations = ref<RelationItem[]>([]);
  const contentTags = ref<ContentTagItem[]>([]);
  const vocabulary = ref<ContentTagItem[]>([]);
  const projectLinks = ref<ProjectContentItem[]>([]);
  // 创建响应里的相似内容提示：只提示，不阻断（D-24）
  const similarHints = ref<ContentHint[]>([]);
  // 关系与项目关联只带 id，标题需要单独补齐（契约未在关系里冗余标题）
  const titles = ref<Record<string, string>>({});

  const loading = ref(false);
  const detailLoading = ref(false);
  const error = ref("");

  const totalPages = computed(() =>
    Math.max(1, Math.ceil(total.value / CONTENT_PAGE_SIZE)),
  );

  function buildQuery(): ContentListQuery {
    return {
      page: page.value,
      page_size: CONTENT_PAGE_SIZE,
      content_type: typeFilter.value === "all" ? undefined : typeFilter.value,
      status: statusFilter.value,
      q: keyword.value.trim() || undefined,
      // 浏览子层级时用 parent_id，根层级用 root_only（两者互斥）
      parent_id: parentId.value ?? undefined,
      root_only: parentId.value === null ? true : undefined,
    };
  }

  function rememberTitles(entries: { id: string; title: string }[]): void {
    for (const entry of entries) {
      titles.value[entry.id] = entry.title;
    }
  }

  function clearError(): void {
    error.value = "";
  }

  /** 相似提示只是创建后的一次提醒，看过了就收起 */
  function clearHints(): void {
    similarHints.value = [];
  }

  /** 关系只带 id，标题在需要时补取；补齐失败不阻断关系列表 */
  async function resolveTitles(ids: string[]): Promise<void> {
    const missing = [...new Set(ids)].filter((id) => titles.value[id] === undefined);

    if (missing.length === 0) {
      return;
    }

    const resolved = await Promise.all(
      missing.map(async (id) => {
        try {
          const item = await fetchContent(id);
          return { id, title: item.title };
        } catch {
          return null;
        }
      }),
    );

    rememberTitles(resolved.filter((entry) => entry !== null));
  }

  async function load(): Promise<void> {
    loading.value = true;
    error.value = "";

    try {
      const data = await fetchContents(buildQuery());
      items.value = data.items;
      total.value = data.total;
      rememberTitles(data.items);
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      loading.value = false;
    }
  }

  async function setType(next: ContentTypeFilter): Promise<void> {
    if (typeFilter.value === next) {
      return;
    }
    typeFilter.value = next;
    page.value = 1;
    await load();
  }

  async function setStatus(next: ContentStatusFilter): Promise<void> {
    if (statusFilter.value === next) {
      return;
    }
    statusFilter.value = next;
    page.value = 1;
    await load();
  }

  async function setKeyword(next: string): Promise<void> {
    keyword.value = next;
    page.value = 1;
    await load();
  }

  async function setPage(next: number): Promise<void> {
    page.value = next;
    await load();
  }

  /** 进入某内容的子层级 */
  async function enter(item: ContentItem): Promise<void> {
    crumbs.value = [...crumbs.value, {id: item.id, title: item.title}];
    parentId.value = item.id;
    page.value = 1;
    await load();
  }

  /** 回到某一层；index 为 -1 表示回到根 */
  async function goTo(index: number): Promise<void> {
    crumbs.value = crumbs.value.slice(0, index + 1);
    parentId.value = index < 0 ? null : crumbs.value[index].id;
    page.value = 1;
    await load();
  }

  async function select(id: string): Promise<void> {
    detailLoading.value = true;
    error.value = "";

    try {
      selected.value = await fetchContent(id);
      rememberTitles([selected.value]);
      await Promise.all([loadRelations(id), loadContentTags(id), loadProjectLinks(id)]);
    } catch (e: unknown) {
      error.value = describeError(e);
      selected.value = null;
    } finally {
      detailLoading.value = false;
    }
  }

  function clearSelection(): void {
    selected.value = null;
    relations.value = [];
    contentTags.value = [];
    projectLinks.value = [];
  }

  async function create(
    payload: Parameters<typeof createContent>[0],
  ): Promise<ContentDetailItem | null> {
    error.value = "";
    try {
      const data = await createContent(payload);
      similarHints.value = data.similar_hints;
      await load();
      await select(data.item.id);
      return data.item;
    } catch (e: unknown) {
      error.value = describeError(e);
      return null;
    }
  }

  async function save(
    id: string,
    payload: UpdateContentInput,
  ): Promise<ContentDetailItem | null> {
    error.value = "";
    try {
      const data = await updateContent(id, payload);
      selected.value = data;
      await load();
      return data;
    } catch (e: unknown) {
      error.value = describeError(e);
      return null;
    }
  }

  /** 默认归档；purge=true 为彻底删除 */
  async function remove(id: string, purge = false): Promise<boolean> {
    error.value = "";
    try {
      await deleteContent(id, purge);
      if (selected.value?.id === id) {
        clearSelection();
      }
      await load();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function loadRelations(contentId: string): Promise<void> {
    try {
      const data = await fetchRelations(contentId);
      relations.value = data.items;

      // 关系的另一端可能是列表外的内容，补齐标题后关系面板才能显示「前置：XXX」
      await resolveTitles(
        data.items.map((item) =>
          item.source_content_id === contentId
            ? item.target_content_id
            : item.source_content_id,
        ),
      );
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function addRelation(
    contentId: string,
    targetId: string,
    relationType: RelationType,
  ): Promise<boolean> {
    error.value = "";
    try {
      await createRelation(contentId, {
        target_content_id: targetId,
        relation_type: relationType,
      });
      await loadRelations(contentId);
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function removeRelation(
    contentId: string,
    relationId: string,
  ): Promise<boolean> {
    error.value = "";
    try {
      await deleteRelation(contentId, relationId);
      await loadRelations(contentId);
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function loadContentTags(contentId: string): Promise<void> {
    try {
      contentTags.value = (await fetchContentTags(contentId)).items;
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function loadVocabulary(): Promise<void> {
    try {
      vocabulary.value = (await fetchTags({page: 1, page_size: 100})).items;
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  /** 整体替换内容标签（D-28） */
  async function replaceTags(contentId: string, tagIds: string[]): Promise<boolean> {
    error.value = "";
    try {
      contentTags.value = (await setContentTags(contentId, tagIds)).items;
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function addTag(
    name: string,
    color?: string,
  ): Promise<ContentTagItem | null> {
    error.value = "";
    try {
      const tag = await createTag({name, color});
      await loadVocabulary();
      return tag;
    } catch (e: unknown) {
      error.value = describeError(e);
      return null;
    }
  }

  async function renameTag(tagId: string, name: string): Promise<boolean> {
    error.value = "";
    try {
      await updateTag(tagId, {name});
      await loadVocabulary();
      if (selected.value) {
        await loadContentTags(selected.value.id);
      }
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function removeTag(tagId: string): Promise<boolean> {
    error.value = "";
    try {
      await deleteTag(tagId);
      await loadVocabulary();
      if (selected.value) {
        await loadContentTags(selected.value.id);
      }
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  /** 「这份内容在哪些项目里、到哪一步了」——跨项目复用查询 */
  async function loadProjectLinks(contentId: string): Promise<void> {
    try {
      projectLinks.value = (
        await fetchProjectContents({content_id: contentId, page_size: 100})
      ).items;
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function attachToProject(
    projectId: string,
    contentId: string,
  ): Promise<boolean> {
    error.value = "";
    try {
      await attachContent({project_id: projectId, content_id: contentId});
      await loadProjectLinks(contentId);
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function updateLink(
    linkId: string,
    payload: Parameters<typeof updateProjectContent>[1],
  ): Promise<boolean> {
    error.value = "";
    try {
      await updateProjectContent(linkId, payload);
      if (selected.value) {
        await loadProjectLinks(selected.value.id);
      }
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function detach(linkId: string): Promise<boolean> {
    error.value = "";
    try {
      await detachContent(linkId);
      if (selected.value) {
        await loadProjectLinks(selected.value.id);
      }
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  return {
    items,
    total,
    page,
    typeFilter,
    statusFilter,
    keyword,
    parentId,
    crumbs,
    selected,
    relations,
    contentTags,
    vocabulary,
    projectLinks,
    similarHints,
    titles,
    loading,
    detailLoading,
    error,
    totalPages,
    clearError,
    clearHints,
    load,
    setType,
    setStatus,
    setKeyword,
    setPage,
    enter,
    goTo,
    select,
    clearSelection,
    create,
    save,
    remove,
    loadRelations,
    addRelation,
    removeRelation,
    loadContentTags,
    loadVocabulary,
    replaceTags,
    addTag,
    renameTag,
    removeTag,
    loadProjectLinks,
    attachToProject,
    updateLink,
    detach,
  };
});

export type {ProjectContentStatus};
