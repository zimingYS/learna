import {defineStore} from "pinia";
import {computed, ref} from "vue";
import {
  applyProjectAction,
  createProject,
  type CreateProjectInput,
  deleteProject,
  extendProjectDeadline,
  type ExtendDeadlineInput,
  fetchProjects,
  type ProjectAction,
  type ProjectItem,
  type ProjectStatus,
  purgeProject,
  updateProject,
  type UpdateProjectInput,
} from "../api/projects";
import {describeError} from "../api/http";

export const PROJECT_PAGE_SIZE = 12;

// 列表筛选里的「全部」不是后端状态值，仅用于视图
export type ProjectStatusFilter = ProjectStatus | "all";

export const useProjectsStore = defineStore("projects", () => {
  const items = ref<ProjectItem[]>([]);
  const total = ref(0);
  const page = ref(1);
  const status = ref<ProjectStatusFilter>("all");
  // 默认排除归档（PRD §4.3）；切到「已归档」时自动打开
  const includeArchived = ref(false);
  const loading = ref(false);
  const error = ref("");

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / PROJECT_PAGE_SIZE)));

  async function load(): Promise<void> {
    loading.value = true;
    error.value = "";

    try {
      const data = await fetchProjects({
        page: page.value,
        page_size: PROJECT_PAGE_SIZE,
        status: status.value === "all" ? undefined : status.value,
        include_archived: includeArchived.value,
      });

      items.value = data.items;
      total.value = data.total;
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      loading.value = false;
    }
  }

  async function setStatus(next: ProjectStatusFilter): Promise<void> {
    if (status.value === next) {
      return;
    }

    status.value = next;
    page.value = 1;

    await load();
  }

  async function setIncludeArchived(next: boolean): Promise<void> {
    if (includeArchived.value === next) {
      return;
    }

    includeArchived.value = next;
    page.value = 1;

    await load();
  }

  async function goToPage(next: number): Promise<void> {
    if (next < 1 || next > totalPages.value || next === page.value) {
      return;
    }

    page.value = next;

    await load();
  }

  // 以下写操作统一「先清错、成功后重取列表」，避免本地状态与后端漂移
  async function create(input: CreateProjectInput): Promise<ProjectItem | null> {
    error.value = "";

    try {
      const created = await createProject(input);
      page.value = 1;
      await load();

      return created;
    } catch (e: unknown) {
      error.value = describeError(e);

      return null;
    }
  }

  async function update(id: string, input: UpdateProjectInput): Promise<ProjectItem | null> {
    error.value = "";

    try {
      const updated = await updateProject(id, input);
      await load();

      return updated;
    } catch (e: unknown) {
      error.value = describeError(e);

      return null;
    }
  }

  async function removeDraft(id: string): Promise<boolean> {
    error.value = "";

    try {
      await deleteProject(id);

      if (items.value.length === 1 && page.value > 1) {
        page.value--;
      }

      await load();

      return true;
    } catch (e: unknown) {
      error.value = describeError(e);

      return false;
    }
  }

  async function runAction(id: string, action: ProjectAction): Promise<ProjectItem | null> {
    error.value = "";

    try {
      const updated = await applyProjectAction(id, action);
      await load();

      return updated;
    } catch (e: unknown) {
      error.value = describeError(e);

      return null;
    }
  }

  async function extendDeadline(id: string, input: ExtendDeadlineInput): Promise<ProjectItem | null> {
    error.value = "";

    try {
      const updated = await extendProjectDeadline(id, input);
      await load();

      return updated;
    } catch (e: unknown) {
      error.value = describeError(e);

      return null;
    }
  }

  async function purge(id: string, confirmName: string): Promise<boolean> {
    error.value = "";

    try {
      await purgeProject(id, confirmName);
      await load();

      return true;
    } catch (e: unknown) {
      error.value = describeError(e);

      return false;
    }
  }

  function clearError(): void {
    error.value = "";
  }

  return {
    items,
    total,
    page,
    status,
    includeArchived,
    loading,
    error,
    totalPages,

    load,
    setStatus,
    setIncludeArchived,
    goToPage,
    create,
    update,
    removeDraft,
    runAction,
    extendDeadline,
    purge,
    clearError,
  };
});
