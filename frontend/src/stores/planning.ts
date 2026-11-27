import {defineStore} from "pinia";
import {computed, ref} from "vue";
import {describeError} from "../api/http";
import {
  type CreateGoalInput,
  type CreateMilestoneInput,
  type CreateRoadmapInput,
  type CreateStageInput,
  type CreateTaskInput,
  type GoalItem,
  type GoalStageItem,
  type GoalStageLink,
  type MilestoneItem,
  type RoadmapItem,
  type StageItem,
  type TaskAction,
  type TaskDependencyItem,
  type TaskItem,
  type TaskStatus,
  type UpdateGoalInput,
  type UpdateMilestoneInput,
  type UpdateStageInput,
  type UpdateTaskInput,
  applyStageAction,
  confirmRoadmap,
  createDependency,
  createGoal,
  createMilestone,
  createRoadmap,
  createStage,
  createTask,
  deleteDependency,
  deleteGoal,
  deleteMilestone,
  deleteRoadmap,
  deleteStage,
  deleteTask,
  fetchDependencies,
  fetchGoalStages,
  fetchGoals,
  fetchMilestones,
  fetchRoadmaps,
  fetchStages,
  fetchTasks,
  replaceGoalStages,
  updateGoal,
  updateMilestone,
  updateRoadmap,
  updateStage,
  updateTask,
} from "../api/planning";

export const GOAL_PAGE_SIZE = 20;
export const ROADMAP_PAGE_SIZE = 10;
export const TASK_PAGE_SIZE = 20;
const MILESTONE_FETCH_SIZE = 100;

// 列表筛选里的「全部」不是后端状态值，仅用于视图
export type TaskStatusFilter = TaskStatus | "all";

export const usePlanningStore = defineStore("planning", () => {
  const projectId = ref<string | null>(null);
  const error = ref("");

  const goals = ref<GoalItem[]>([]);
  const goalsTotal = ref(0);
  const goalsLoading = ref(false);

  const roadmaps = ref<RoadmapItem[]>([]);
  const roadmapsTotal = ref(0);
  const roadmapsPage = ref(1);
  const roadmapsLoading = ref(false);

  const activeRoadmapId = ref<string | null>(null);
  const stages = ref<StageItem[]>([]);
  const milestones = ref<MilestoneItem[]>([]);
  const stagesLoading = ref(false);

  const tasks = ref<TaskItem[]>([]);
  const tasksTotal = ref(0);
  const tasksPage = ref(1);
  const taskStatus = ref<TaskStatusFilter>("all");
  const tasksLoading = ref(false);

  const roadmapTotalPages = computed(() =>
    Math.max(1, Math.ceil(roadmapsTotal.value / ROADMAP_PAGE_SIZE)),
  );

  const taskTotalPages = computed(() => Math.max(1, Math.ceil(tasksTotal.value / TASK_PAGE_SIZE)));

  // 当前查看的路线对象：阶段动作与结构准入都依赖它的状态
  const activeRoadmap = computed(
    () => roadmaps.value.find((roadmap) => roadmap.id === activeRoadmapId.value) ?? null,
  );

  // 里程碑一次取全项目，按阶段分组交给视图（避免逐阶段请求）
  const milestonesByStage = computed(() => {
    const grouped = new Map<string, MilestoneItem[]>();

    for (const milestone of milestones.value) {
      const siblings = grouped.get(milestone.stage_id) ?? [];
      siblings.push(milestone);
      grouped.set(milestone.stage_id, siblings);
    }

    return grouped;
  });

  async function loadGoals(): Promise<void> {
    const project = projectId.value;

    if (!project) {
      return;
    }

    goalsLoading.value = true;
    error.value = "";

    try {
      const data = await fetchGoals(project, 1, GOAL_PAGE_SIZE);
      goals.value = data.items;
      goalsTotal.value = data.total;
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      goalsLoading.value = false;
    }
  }

  async function loadRoadmaps(): Promise<void> {
    const project = projectId.value;

    if (!project) {
      return;
    }

    roadmapsLoading.value = true;
    error.value = "";

    try {
      const data = await fetchRoadmaps(project, roadmapsPage.value, ROADMAP_PAGE_SIZE);
      roadmaps.value = data.items;
      roadmapsTotal.value = data.total;

      // 当前查看的路线被删除或翻页后不在列表里，回落到第一条
      if (!data.items.some((item) => item.id === activeRoadmapId.value)) {
        activeRoadmapId.value = data.items[0]?.id ?? null;
      }
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      roadmapsLoading.value = false;
    }
  }

  async function loadStages(): Promise<void> {
    const project = projectId.value;
    const roadmap = activeRoadmapId.value;

    if (!project || !roadmap) {
      stages.value = [];
      milestones.value = [];

      return;
    }

    stagesLoading.value = true;
    error.value = "";

    try {
      const [stageData, milestoneData] = await Promise.all([
        fetchStages(project, roadmap),
        fetchMilestones(project, {page: 1, page_size: MILESTONE_FETCH_SIZE}),
      ]);

      stages.value = stageData.items;
      milestones.value = milestoneData.items;
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      stagesLoading.value = false;
    }
  }

  async function loadTasks(): Promise<void> {
    const project = projectId.value;

    if (!project) {
      return;
    }

    tasksLoading.value = true;
    error.value = "";

    try {
      const data = await fetchTasks(project, {
        page: tasksPage.value,
        page_size: TASK_PAGE_SIZE,
        status: taskStatus.value === "all" ? undefined : taskStatus.value,
      });

      tasks.value = data.items;
      tasksTotal.value = data.total;
    } catch (e: unknown) {
      error.value = describeError(e);
    } finally {
      tasksLoading.value = false;
    }
  }

  /** 删除结构（阶段/里程碑/路线）后：任务被解挂，三块都要重取 */
  async function reloadAfterDetach(): Promise<void> {
    await loadRoadmaps();
    await loadStages();
    await loadTasks();
  }

  /** 切换项目：清空上一个项目的残留，避免串数据 */
  async function setProject(next: string | null): Promise<void> {
    if (projectId.value === next) {
      return;
    }

    projectId.value = next;
    activeRoadmapId.value = null;
    roadmapsPage.value = 1;
    tasksPage.value = 1;
    taskStatus.value = "all";
    goals.value = [];
    roadmaps.value = [];
    stages.value = [];
    milestones.value = [];
    tasks.value = [];

    if (!next) {
      return;
    }

    await Promise.all([loadGoals(), loadRoadmaps(), loadTasks()]);
    await loadStages();
  }

  async function selectRoadmap(roadmapId: string): Promise<void> {
    if (activeRoadmapId.value === roadmapId) {
      return;
    }

    activeRoadmapId.value = roadmapId;

    await loadStages();
  }

  async function setTaskStatusFilter(next: TaskStatusFilter): Promise<void> {
    if (taskStatus.value === next) {
      return;
    }

    taskStatus.value = next;
    tasksPage.value = 1;

    await loadTasks();
  }

  async function goToTaskPage(next: number): Promise<void> {
    if (next < 1 || next > taskTotalPages.value || next === tasksPage.value) {
      return;
    }

    tasksPage.value = next;

    await loadTasks();
  }

  async function goToRoadmapPage(next: number): Promise<void> {
    if (next < 1 || next > roadmapTotalPages.value || next === roadmapsPage.value) {
      return;
    }

    roadmapsPage.value = next;

    await loadRoadmaps();
    await loadStages();
  }

  // 写操作统一「先清错、成功后重取对应分区」，避免本地状态与后端漂移
  async function run<T>(action: () => Promise<T>, reload: () => Promise<void>): Promise<T | null> {
    error.value = "";

    try {
      const result = await action();
      await reload();

      return result;
    } catch (e: unknown) {
      error.value = describeError(e);

      return null;
    }
  }

  /** 项目级写操作：缺少项目上下文时直接放弃（视图已禁用入口） */
  function withProject<T>(
    action: (project: string) => Promise<T>,
    reload: () => Promise<void>,
  ): Promise<T | null> {
    const project = projectId.value;

    if (!project) {
      return Promise.resolve(null);
    }

    return run(() => action(project), reload);
  }

  /** 阶段级写操作：同时需要项目与当前路线 */
  function withRoadmap<T>(
    action: (project: string, roadmap: string) => Promise<T>,
    reload: () => Promise<void>,
  ): Promise<T | null> {
    const project = projectId.value;
    const roadmap = activeRoadmapId.value;

    if (!project || !roadmap) {
      return Promise.resolve(null);
    }

    return run(() => action(project, roadmap), reload);
  }

  function addGoal(input: CreateGoalInput): Promise<GoalItem | null> {
    return withProject((project) => createGoal(project, input), loadGoals);
  }

  function editGoal(goalId: string, input: UpdateGoalInput): Promise<GoalItem | null> {
    return withProject((project) => updateGoal(project, goalId, input), loadGoals);
  }

  /** 删除目标：主目标带子目标时须显式 cascade，否则后端 409 并给出 impact */
  function removeGoal(goalId: string, cascade: boolean): Promise<void | null> {
    return withProject((project) => deleteGoal(project, goalId, cascade), loadGoals);
  }

  async function loadGoalStages(goalId: string): Promise<GoalStageItem[]> {
    const project = projectId.value;

    if (!project) {
      return [];
    }

    error.value = "";

    try {
      const data = await fetchGoalStages(project, goalId);

      return data.items;
    } catch (e: unknown) {
      error.value = describeError(e);

      return [];
    }
  }

  async function saveGoalStages(goalId: string, links: GoalStageLink[]): Promise<boolean> {
    const project = projectId.value;

    if (!project) {
      return false;
    }

    error.value = "";

    try {
      await replaceGoalStages(project, goalId, links);

      return true;
    } catch (e: unknown) {
      error.value = describeError(e);

      return false;
    }
  }

  function addRoadmap(input: CreateRoadmapInput): Promise<RoadmapItem | null> {
    return withProject(async (project) => {
      const created = await createRoadmap(project, input);
      activeRoadmapId.value = created.id;

      return created;
    }, loadRoadmaps);
  }

  function editRoadmap(roadmapId: string, summary: string | null): Promise<RoadmapItem | null> {
    return withProject(
      (project) => updateRoadmap(project, roadmapId, {change_summary: summary}),
      loadRoadmaps,
    );
  }

  function removeRoadmap(roadmapId: string): Promise<void | null> {
    return withProject((project) => deleteRoadmap(project, roadmapId), reloadAfterDetach);
  }

  function activateRoadmap(roadmapId: string): Promise<RoadmapItem | null> {
    return withProject((project) => confirmRoadmap(project, roadmapId), loadRoadmaps);
  }

  function addStage(input: CreateStageInput): Promise<StageItem | null> {
    return withRoadmap((project, roadmap) => createStage(project, roadmap, input), loadStages);
  }

  function editStage(stageId: string, input: UpdateStageInput): Promise<StageItem | null> {
    return withRoadmap(
      (project, roadmap) => updateStage(project, roadmap, stageId, input),
      loadStages,
    );
  }

  function removeStage(stageId: string): Promise<void | null> {
    return withRoadmap((project, roadmap) => deleteStage(project, roadmap, stageId), reloadAfterDetach);
  }

  function moveStage(stageId: string, action: TaskAction): Promise<StageItem | null> {
    return withRoadmap(
      (project, roadmap) => applyStageAction(project, roadmap, stageId, action),
      loadStages,
    );
  }

  function addMilestone(input: CreateMilestoneInput): Promise<MilestoneItem | null> {
    return withProject((project) => createMilestone(project, input), loadStages);
  }

  function editMilestone(
    milestoneId: string,
    input: UpdateMilestoneInput,
  ): Promise<MilestoneItem | null> {
    return withProject((project) => updateMilestone(project, milestoneId, input), loadStages);
  }

  function removeMilestone(milestoneId: string): Promise<void | null> {
    return withProject((project) => deleteMilestone(project, milestoneId), reloadAfterDetach);
  }

  function addTask(input: CreateTaskInput): Promise<TaskItem | null> {
    return withProject((project) => createTask(project, input), loadTasks);
  }

  function editTask(taskId: string, input: UpdateTaskInput): Promise<TaskItem | null> {
    return withProject((project) => updateTask(project, taskId, input), loadTasks);
  }

  function setTaskStatus(taskId: string, status: TaskStatus): Promise<TaskItem | null> {
    return editTask(taskId, {status});
  }

  function removeTask(taskId: string): Promise<void | null> {
    return withProject((project) => deleteTask(project, taskId), loadTasks);
  }

  /** 依赖选择器的候选集：一次取满 100 条，避免只看到当前页 */
  async function loadTaskOptions(): Promise<TaskItem[]> {
    const project = projectId.value;

    if (!project) {
      return [];
    }

    try {
      const data = await fetchTasks(project, {page: 1, page_size: 100});

      return data.items;
    } catch (e: unknown) {
      error.value = describeError(e);

      return [];
    }
  }

  async function loadDependencies(taskId: string): Promise<TaskDependencyItem[]> {
    const project = projectId.value;

    if (!project) {
      return [];
    }

    error.value = "";

    try {
      const data = await fetchDependencies(project, taskId);

      return data.items;
    } catch (e: unknown) {
      error.value = describeError(e);

      return [];
    }
  }

  async function addDependency(taskId: string, dependsOnTaskId: string): Promise<boolean> {
    const project = projectId.value;

    if (!project) {
      return false;
    }

    error.value = "";

    try {
      await createDependency(project, taskId, {depends_on_task_id: dependsOnTaskId});

      return true;
    } catch (e: unknown) {
      error.value = describeError(e);

      return false;
    }
  }

  async function removeDependency(taskId: string, dependsOnTaskId: string): Promise<boolean> {
    const project = projectId.value;

    if (!project) {
      return false;
    }

    error.value = "";

    try {
      await deleteDependency(project, taskId, dependsOnTaskId);

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
    projectId,
    error,

    goals,
    goalsTotal,
    goalsLoading,

    roadmaps,
    roadmapsTotal,
    roadmapsPage,
    roadmapsLoading,
    roadmapTotalPages,
    activeRoadmapId,
    activeRoadmap,
    stages,
    milestones,
    stagesLoading,
    milestonesByStage,

    tasks,
    tasksTotal,
    tasksPage,
    taskStatus,
    tasksLoading,
    taskTotalPages,

    setProject,
    selectRoadmap,
    loadGoals,
    loadRoadmaps,
    loadStages,
    loadTasks,
    setTaskStatusFilter,
    goToTaskPage,
    goToRoadmapPage,

    addGoal,
    editGoal,
    removeGoal,
    loadGoalStages,
    saveGoalStages,

    addRoadmap,
    editRoadmap,
    removeRoadmap,
    activateRoadmap,

    addStage,
    editStage,
    removeStage,
    moveStage,

    addMilestone,
    editMilestone,
    removeMilestone,

    addTask,
    editTask,
    setTaskStatus,
    removeTask,
    loadTaskOptions,
    loadDependencies,
    addDependency,
    removeDependency,
    clearError,
  };
});
