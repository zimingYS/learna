import {defineStore} from "pinia";
import {computed, ref} from "vue";
import {fetchContents} from "../api/contents";
import {describeError} from "../api/http";
import {fetchTasks} from "../api/planning";
import {fetchProjects} from "../api/projects";
import {
  type ActivityType,
  type CreateEvidenceInput,
  type CreateSessionInput,
  createEvidence,
  createSession,
  deleteEvidence,
  deleteSession,
  type EvidenceItem,
  fetchEvidence,
  fetchSession,
  fetchSessions,
  type SessionContentInput,
  type SessionDetailItem,
  type SessionItem,
  type SessionListQuery,
  type SessionTaskInput,
  setSessionContents,
  setSessionProjects,
  setSessionTasks,
  type UpdateSessionInput,
  updateSession,
} from "../api/sessions";

export const SESSION_PAGE_SIZE = 10;
export const EVIDENCE_PAGE_SIZE = 50;

// 「全部」只存在于视图层，不是后端枚举值
export type ActivityTypeFilter = ActivityType | "all";

export interface ProjectOption {
  id: string;
  name: string;
}

export interface ContentOption {
  id: string;
  title: string;
}

export interface TaskOption {
  id: string;
  title: string;
  projectId: string;
}

/** 计时器快照：只有客户端时间，服务端没有 timer 状态（ADR-010 §2） */
export interface TimerSnapshot {
  startedAt: string;
  endedAt: string;
  durationMinutes: number;
}

export const useSessionsStore = defineStore("sessions", () => {
  const items = ref<SessionItem[]>([]);
  const total = ref(0);
  const page = ref(1);
  const typeFilter = ref<ActivityTypeFilter>("all");
  const projectFilter = ref("");
  // 半开区间 [from, to)，与后端口径一致；空串表示不限
  const fromDate = ref("");
  const toDate = ref("");

  const selected = ref<SessionDetailItem | null>(null);

  const evidence = ref<EvidenceItem[]>([]);
  const evidenceTotal = ref(0);
  // 选中活动时看本次活动的证据，否则看全部（含独立证据）
  const evidenceScoped = computed(() => selected.value !== null);

  const projectOptions = ref<ProjectOption[]>([]);
  const contentOptions = ref<ContentOption[]>([]);
  const tasksByProject = ref<Record<string, TaskOption[]>>({});

  const loading = ref(false);
  const detailLoading = ref(false);
  const error = ref("");

  // 计时状态全部在内存：重新加载页面即丢失，这是 ADR-010 选定的代价
  const timerFirstStartedAt = ref<number | null>(null);
  const timerRunStartedAt = ref<number | null>(null);
  const timerAccumulatedMs = ref(0);

  const timerRunning = computed(() => timerRunStartedAt.value !== null);
  const timerStarted = computed(() => timerFirstStartedAt.value !== null);

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / SESSION_PAGE_SIZE)));

  function buildQuery(): SessionListQuery {
    return {
      page: page.value,
      page_size: SESSION_PAGE_SIZE,
      activity_type: typeFilter.value === "all" ? undefined : typeFilter.value,
      primary_project_id: projectFilter.value || undefined,
      from: fromDate.value ? new Date(`${fromDate.value}T00:00:00`).toISOString() : undefined,
      to: toDate.value ? new Date(`${toDate.value}T23:59:59`).toISOString() : undefined,
    };
  }

  async function load(): Promise<void> {
    loading.value = true;
    error.value = "";

    try {
      const data = await fetchSessions(buildQuery());
      items.value = data.items;
      total.value = data.total;
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      loading.value = false;
    }
  }

  async function setType(next: ActivityTypeFilter): Promise<void> {
    typeFilter.value = next;
    page.value = 1;
    await load();
  }

  async function setProject(next: string): Promise<void> {
    projectFilter.value = next;
    page.value = 1;
    await load();
  }

  async function setRange(from: string, to: string): Promise<void> {
    fromDate.value = from;
    toDate.value = to;
    page.value = 1;
    await load();
  }

  async function setPage(next: number): Promise<void> {
    page.value = Math.min(Math.max(1, next), totalPages.value);
    await load();
  }

  async function select(id: string): Promise<void> {
    detailLoading.value = true;
    error.value = "";

    try {
      selected.value = await fetchSession(id);
      await loadEvidence();
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      detailLoading.value = false;
    }
  }

  function clearSelection(): void {
    selected.value = null;
    void loadEvidence();
  }

  /** 关联替换后重取详情：关联是独立端点，改完必须回读才能看到服务端结果 */
  async function reloadSelected(): Promise<void> {
    if (!selected.value) {
      return;
    }

    const id = selected.value.id;

    try {
      selected.value = await fetchSession(id);
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function create(input: CreateSessionInput): Promise<boolean> {
    error.value = "";

    try {
      const created = await createSession(input);
      selected.value = created;
      page.value = 1;
      await Promise.all([load(), loadEvidence()]);
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function save(input: UpdateSessionInput): Promise<boolean> {
    if (!selected.value) {
      return false;
    }

    error.value = "";

    try {
      selected.value = await updateSession(selected.value.id, input);
      await load();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function remove(id: string): Promise<boolean> {
    error.value = "";

    try {
      await deleteSession(id);

      if (selected.value?.id === id) {
        selected.value = null;
      }
      await Promise.all([load(), loadEvidence()]);
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function replaceContents(rows: SessionContentInput[]): Promise<boolean> {
    if (!selected.value) {
      return false;
    }

    error.value = "";

    try {
      await setSessionContents(selected.value.id, rows);
      await reloadSelected();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function replaceProjects(projectIds: string[]): Promise<boolean> {
    if (!selected.value) {
      return false;
    }

    error.value = "";

    try {
      await setSessionProjects(selected.value.id, projectIds);
      await reloadSelected();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function replaceTasks(rows: SessionTaskInput[]): Promise<boolean> {
    if (!selected.value) {
      return false;
    }

    error.value = "";

    try {
      await setSessionTasks(selected.value.id, rows);
      await reloadSelected();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function loadEvidence(): Promise<void> {
    try {
      const data = await fetchEvidence({
        page: 1,
        page_size: EVIDENCE_PAGE_SIZE,
        session_id: selected.value?.id,
      });
      evidence.value = data.items;
      evidenceTotal.value = data.total;
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function addEvidence(input: CreateEvidenceInput): Promise<boolean> {
    error.value = "";

    try {
      await createEvidence(input);
      await loadEvidence();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  async function removeEvidence(id: string): Promise<boolean> {
    error.value = "";

    try {
      await deleteEvidence(id);
      await loadEvidence();
      return true;
    } catch (e: unknown) {
      error.value = describeError(e);
      return false;
    }
  }

  /** 选择器数据：项目与内容各取一页，个人规模足够；任务按项目懒加载 */
  async function loadOptions(): Promise<void> {
    try {
      const [projects, contents] = await Promise.all([
        fetchProjects({page: 1, page_size: 100}),
        fetchContents({page: 1, page_size: 100, status: "active"}),
      ]);

      projectOptions.value = projects.items.map((item) => ({id: item.id, name: item.name}));
      contentOptions.value = contents.items.map((item) => ({id: item.id, title: item.title}));
    } catch (e: unknown) {
      error.value = describeError(e);
    }
  }

  async function loadTasks(projectId: string): Promise<TaskOption[]> {
    if (!projectId) {
      return [];
    }

    const cached = tasksByProject.value[projectId];

    if (cached) {
      return cached;
    }

    try {
      const data = await fetchTasks(projectId, {page: 1, page_size: 100});
      const options = data.items.map((item) => ({
        id: item.id,
        title: item.title,
        projectId,
      }));
      tasksByProject.value = {...tasksByProject.value, [projectId]: options};
      return options;
    } catch (e: unknown) {
      error.value = describeError(e);
      return [];
    }
  }

  function projectName(id: string | null): string {
    if (!id) {
      return "未归属项目";
    }

    return projectOptions.value.find((item) => item.id === id)?.name ?? "未知项目";
  }

  function contentTitle(id: string): string {
    return contentOptions.value.find((item) => item.id === id)?.title ?? id;
  }

  function taskTitle(id: string): string {
    for (const options of Object.values(tasksByProject.value)) {
      const found = options.find((item) => item.id === id);

      if (found) {
        return found.title;
      }
    }

    return id;
  }

  // -------------------------------------------------------------------------
  // 客户端计时（不建服务端 timer，ADR-010 §2）
  // -------------------------------------------------------------------------

  function startTimer(): void {
    const now = Date.now();

    timerFirstStartedAt.value ??= now;
    timerRunStartedAt.value = now;
  }

  function pauseTimer(): void {
    if (timerRunStartedAt.value === null) {
      return;
    }

    timerAccumulatedMs.value += Date.now() - timerRunStartedAt.value;
    timerRunStartedAt.value = null;
  }

  function resetTimer(): void {
    timerFirstStartedAt.value = null;
    timerRunStartedAt.value = null;
    timerAccumulatedMs.value = 0;
  }

  /** 当前累计毫秒：运行中要把「本轮已过时间」算进去 */
  function timerElapsedMs(now: number): number {
    const running = timerRunStartedAt.value === null ? 0 : now - timerRunStartedAt.value;

    return timerAccumulatedMs.value + running;
  }

  /** 结束计时并给出可提交的时间区间；不足 1 分钟按 1 分钟算，避免 0 分钟记录 */
  function timerSnapshot(now = Date.now()): TimerSnapshot | null {
    if (timerFirstStartedAt.value === null) {
      return null;
    }

    pauseTimer();

    return {
      startedAt: new Date(timerFirstStartedAt.value).toISOString(),
      endedAt: new Date(now).toISOString(),
      durationMinutes: Math.max(1, Math.floor(timerAccumulatedMs.value / 60_000)),
    };
  }

  return {
    items,
    total,
    page,
    typeFilter,
    projectFilter,
    fromDate,
    toDate,
    selected,
    evidence,
    evidenceTotal,
    evidenceScoped,
    projectOptions,
    contentOptions,
    tasksByProject,
    loading,
    detailLoading,
    error,
    timerFirstStartedAt,
    timerRunning,
    timerStarted,
    totalPages,
    load,
    setType,
    setProject,
    setRange,
    setPage,
    select,
    clearSelection,
    reloadSelected,
    create,
    save,
    remove,
    replaceContents,
    replaceProjects,
    replaceTasks,
    loadEvidence,
    addEvidence,
    removeEvidence,
    loadOptions,
    loadTasks,
    projectName,
    contentTitle,
    taskTitle,
    startTimer,
    pauseTimer,
    resetTimer,
    timerElapsedMs,
    timerSnapshot,
  };
});
