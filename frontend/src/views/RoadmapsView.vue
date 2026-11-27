<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {useRoute, useRouter} from "vue-router";
import {fetchProjects, type ProjectItem} from "../api/projects";
import {describeError} from "../api/http";
import {usePlanningStore} from "../stores/planning";
import GoalPanel from "../components/planning/GoalPanel.vue";
import RoadmapPanel from "../components/planning/RoadmapPanel.vue";
import StageBoard from "../components/planning/StageBoard.vue";
import TaskPanel from "../components/planning/TaskPanel.vue";

type Tab = "goals" | "roadmaps" | "tasks";

const TABS: { value: Tab; label: string }[] = [
  { value: "goals", label: "目标" },
  { value: "roadmaps", label: "路线与阶段" },
  { value: "tasks", label: "任务" },
];

const store = usePlanningStore();
const route = useRoute();
const router = useRouter();

const projects = ref<ProjectItem[]>([]);
const projectsLoading = ref(false);
const projectsError = ref("");
const tab = ref<Tab>("roadmaps");

const currentProject = computed(
  () => projects.value.find((item) => item.id === store.projectId) ?? null,
);

function normalizeTab(value: unknown): Tab {
  return value === "goals" || value === "tasks" ? value : "roadmaps";
}

async function loadProjects(): Promise<void> {
  projectsLoading.value = true;
  projectsError.value = "";

  try {
    // 规划只对进行中的项目开放（草稿还没开始、归档不再推进）
    const data = await fetchProjects({page: 1, page_size: 100, status: "active"});
    projects.value = data.items;
  } catch (e: unknown) {
    projectsError.value = describeError(e);
  } finally {
    projectsLoading.value = false;
  }
}

/** 项目与分区同步到地址栏，刷新或分享链接能回到同一视图 */
async function syncQuery(projectId: string | null, nextTab: Tab): Promise<void> {
  await router.replace({
    query: {
      ...(projectId ? {project: projectId} : {}),
      tab: nextTab,
    },
  });
}

async function selectProject(projectId: string): Promise<void> {
  if (projectId === store.projectId) {
    return;
  }

  await store.setProject(projectId);
  await syncQuery(projectId, tab.value);
}

async function selectTab(next: Tab): Promise<void> {
  if (tab.value === next) {
    return;
  }

  tab.value = next;
  await syncQuery(store.projectId, next);

  // 结构变更会解挂任务归属，切回来时重取一次，避免看到过期数据
  if (next === "roadmaps") {
    await store.loadStages();
  } else if (next === "tasks") {
    await store.loadTasks();
  }
}

onMounted(async () => {
  tab.value = normalizeTab(route.query.tab);
  await loadProjects();

  const queryProject = typeof route.query.project === "string" ? route.query.project : null;
  const initial =
    projects.value.find((item) => item.id === queryProject)?.id ?? projects.value[0]?.id ?? null;

  if (initial) {
    await store.setProject(initial);
  }
});

function onProjectChange(event: Event): void {
  void selectProject((event.target as HTMLSelectElement).value);
}
</script>

<template>
  <main class="planning-page">
    <div class="planning-container">
      <header class="page-header">
        <div>
          <h1>学习规划</h1>
          <p>把项目拆成目标、路线版本、阶段与任务，并让任务之间保持正确的先后顺序。</p>
        </div>

        <label class="project-picker">
          <span class="picker-label">当前项目</span>
          <select
            :disabled="projectsLoading || projects.length === 0"
            :value="store.projectId ?? ''"
            class="picker-select"
            @change="onProjectChange"
          >
            <option v-if="projects.length === 0" value="">（暂无进行中的项目）</option>
            <option v-for="project in projects" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </select>
        </label>
      </header>

      <p v-if="projectsError" class="error-banner" role="alert">{{ projectsError }}</p>
      <p v-if="store.error" class="error-banner" role="alert">{{ store.error }}</p>

      <nav class="planning-tabs" aria-label="规划分区">
        <button
          v-for="item in TABS"
          :key="item.value"
          :class="{ 'planning-tab--active': tab === item.value }"
          class="planning-tab"
          type="button"
          @click="selectTab(item.value)"
        >
          {{ item.label }}
        </button>
      </nav>

      <p v-if="!projectsLoading && projects.length === 0" class="empty-page">
        目前没有进行中的项目。请先到「学习项目」创建一个项目并开始推进，再回来规划它的目标与路线。
      </p>

      <p v-else-if="!store.projectId" class="empty-page">请选择一个项目开始规划。</p>

      <section v-else class="planning-body">
        <article v-if="tab === 'goals'" class="card-block">
          <GoalPanel/>
        </article>

        <div v-else-if="tab === 'roadmaps'" class="roadmap-layout">
          <article class="card-block">
            <RoadmapPanel/>
          </article>

          <article class="card-block">
            <StageBoard/>
          </article>
        </div>

        <article v-else class="card-block">
          <TaskPanel/>
        </article>
      </section>

      <p v-if="currentProject" class="footnote">
        {{ currentProject.name }} · 重要程度 I{{ currentProject.importance_level }} ·
        目标、路线与任务都只作用于该项目，不影响其他项目。
      </p>
    </div>
  </main>
</template>

<style scoped>
.planning-page {
  min-height: 100%;
  padding: 40px 28px 48px;
  background: #f5f7fb;
  color: #263248;
}

.planning-container {
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;

  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  color: #202b40;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.page-header p {
  margin: 9px 0 0;
  color: #8490a3;
  font-size: 14px;
  line-height: 1.6;
}

.project-picker {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.picker-label {
  color: #98a1b2;
  font-size: 12px;
}

.picker-select {
  min-width: 240px;
  height: 40px;
  padding: 0 12px;

  border: 1px solid #dce2eb;
  border-radius: 9px;

  background: #fff;
  color: #263248;

  font-family: inherit;
  font-size: 14px;
}

.picker-select:focus {
  border-color: #8b93f8;
  outline: none;
  box-shadow: 0 0 0 3px rgb(79 70 229 / 12%);
}

.error-banner {
  padding: 11px 14px;
  margin: 0 0 16px;

  border: 1px solid #f6d5d5;
  border-radius: 10px;

  background: #fff6f6;
  color: #c04141;

  font-size: 13px;
}

.planning-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;

  padding: 6px;
  margin-bottom: 18px;

  border: 1px solid #e9edf4;
  border-radius: 12px;

  background: #fff;
}

.planning-tab {
  padding: 8px 16px;

  border: none;
  border-radius: 8px;

  background: transparent;
  color: #6b7488;

  font-size: 13px;
  font-weight: 500;
  cursor: pointer;

  transition: background-color 0.18s ease, color 0.18s ease;
}

.planning-tab:hover {
  background: #f4f6fc;
  color: #4f46e5;
}

.planning-tab--active {
  background: #eef1fe;
  color: #4f46e5;
  font-weight: 600;
}

.empty-page {
  padding: 60px 24px;
  margin: 0;

  border: 1px dashed #dfe4ee;
  border-radius: 14px;

  background: #fff;
  color: #8a94a6;

  font-size: 14px;
  line-height: 1.8;
  text-align: center;
}

.planning-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-block {
  padding: 22px 24px;

  border: 1px solid #e9edf4;
  border-radius: 14px;

  background: #fff;
}

.roadmap-layout {
  display: grid;
  gap: 16px;
  align-items: start;
  grid-template-columns: minmax(0, 420px) minmax(0, 1fr);
}

.footnote {
  margin: 22px 0 0;
  color: #a0a9b7;
  font-size: 12px;
  text-align: center;
}

@media (max-width: 1080px) {
  .roadmap-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .planning-page {
    padding: 24px 16px 36px;
  }

  .picker-select {
    width: 100%;
    min-width: 0;
  }

  .project-picker {
    width: 100%;
  }

  .card-block {
    padding: 18px 16px;
  }
}
</style>
