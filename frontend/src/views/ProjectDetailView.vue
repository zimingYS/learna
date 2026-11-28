<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {RouterLink, useRoute, useRouter} from "vue-router";
import {fetchProject, type ProjectAction, type ProjectItem} from "../api/projects";
import {describeError} from "../api/http";
import ExtendDeadlineDialog from "../components/projects/ExtendDeadlineDialog.vue";
import ProjectFormDialog from "../components/projects/ProjectFormDialog.vue";
import ProjectDangerCard from "../components/projects/ProjectDangerCard.vue";
import ProjectLifecycleCard from "../components/projects/ProjectLifecycleCard.vue";
import ProjectOverviewCard from "../components/projects/ProjectOverviewCard.vue";
import ProjectStatusBadge from "../components/projects/ProjectStatusBadge.vue";
import PurgeProjectDialog from "../components/projects/PurgeProjectDialog.vue";
import {useProjectsStore} from "../stores/projects";
import {
  IMPORTANCE_LABELS,
  isDeadlineEditable,
  PROJECT_ACTION_LABELS,
  PROJECT_STATUS_HINTS,
  PROJECT_TYPE_ICONS,
  PROJECT_TYPE_LABELS,
} from "./projects/project-rules";

const route = useRoute();
const router = useRouter();
const store = useProjectsStore();

const project = ref<ProjectItem | null>(null);
const loadError = ref("");
const loading = ref(true);
const notice = ref("");
const actionPending = ref<ProjectAction | null>(null);

const showEdit = ref(false);
const showExtend = ref(false);
const showPurge = ref(false);

const projectId = computed(() => String(route.params.projectId ?? ""));

const iconPaths = computed(() =>
  project.value ? PROJECT_TYPE_ICONS[project.value.project_type] : [],
);

function showNotice(message: string): void {
  notice.value = message;
}

async function load(): Promise<void> {
  loading.value = true;
  loadError.value = "";

  try {
    project.value = await fetchProject(projectId.value);
  } catch (error: unknown) {
    loadError.value = describeError(error);
  } finally {
    loading.value = false;
  }
}

function handleSaved(updated: ProjectItem): void {
  project.value = updated;
  showNotice("项目已保存");
}

async function handleAction(action: ProjectAction): Promise<void> {
  if (!project.value || actionPending.value !== null) {
    return;
  }

  // 归档不会删除数据，但会退出活跃列表，先确认一次
  if (action === "archive" && !window.confirm("归档后项目将退出活跃列表，历史数据仍会保留。确定归档？")) {
    return;
  }

  actionPending.value = action;
  notice.value = "";

  try {
    const updated = await store.runAction(project.value.id, action);

    if (updated) {
      project.value = updated;
      showNotice(`已${PROJECT_ACTION_LABELS[action]}`);
    }
  } finally {
    actionPending.value = null;
  }
}

async function handleDeleteDraft(): Promise<void> {
  if (!project.value) {
    return;
  }

  const confirmed = window.confirm(
    `草稿项目「${project.value.name}」及其规划数据将被物理删除，无法恢复。确定删除？`,
  );

  if (!confirmed) {
    return;
  }

  const ok = await store.removeDraft(project.value.id);

  if (ok) {
    await router.push("/projects");
  }
}

async function handlePurged(): Promise<void> {
  await router.push("/projects");
}

onMounted(() => {
  void load();
});
</script>

<template>
  <main class="detail-page">
    <div class="detail-container">
      <RouterLink class="back-link" to="/projects">
        <span aria-hidden="true">←</span>
        返回项目列表
      </RouterLink>

      <p v-if="loading" class="state-block">正在加载项目…</p>

      <section v-else-if="loadError" class="state-block state-block--error" role="alert">
        <p>{{ loadError }}</p>
        <button class="ghost-btn" type="button" @click="load">重试</button>
      </section>

      <template v-else-if="project">
        <header class="detail-header">
          <div class="title-row">
            <div class="type-icon">
              <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
                   stroke-linejoin="round" stroke-width="1.7" viewBox="0 0 24 24">
                <path v-for="path in iconPaths" :key="path" :d="path"/>
              </svg>
            </div>

            <div class="title-block">
              <div class="name-row">
                <h1>{{ project.name }}</h1>
                <ProjectStatusBadge :status="project.status"/>
              </div>

              <p class="subtitle">
                {{ PROJECT_TYPE_LABELS[project.project_type] }}
                <span class="dot">·</span>
                {{ IMPORTANCE_LABELS[project.importance_level] }}
                <span class="dot">·</span>
                {{ PROJECT_STATUS_HINTS[project.status] }}
              </p>
            </div>

            <div class="header-actions">
              <button class="ghost-btn" type="button" @click="showEdit = true">编辑项目</button>

              <button
                v-if="!isDeadlineEditable(project)"
                class="primary-btn"
                type="button"
                @click="showExtend = true"
              >
                修改截止日期
              </button>
            </div>
          </div>
        </header>

        <p v-if="notice" class="notice" role="status">{{ notice }}</p>

        <p v-if="store.error" class="error-banner" role="alert">{{ store.error }}</p>

        <div class="detail-grid">
          <ProjectOverviewCard :project="project"/>

          <div class="side-column">
            <ProjectLifecycleCard
              :pending-action="actionPending"
              :project="project"
              @action="handleAction"
              @delete-draft="handleDeleteDraft"
            />

            <ProjectDangerCard @purge="showPurge = true"/>
          </div>
        </div>

        <ProjectFormDialog
          :open="showEdit"
          :project="project"
          @close="showEdit = false"
          @saved="handleSaved"
        />

        <ExtendDeadlineDialog
          :open="showExtend"
          :project="project"
          @close="showExtend = false"
          @saved="handleSaved"
        />

        <PurgeProjectDialog
          :open="showPurge"
          :project="project"
          @close="showPurge = false"
          @purged="handlePurged"
        />
      </template>
    </div>
  </main>
</template>

<style scoped>
.detail-page {
  min-height: 100%;
  color: var(--ink-1);
}

.detail-container {
  width: 100%;
  max-width: 1120px;
  margin: 0 auto;
}

.back-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;

  margin-bottom: 18px;

  color: var(--ink-3);
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
}

.back-link:hover {
  color: var(--brand);
}

.back-link:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 3px;
}

.state-block {
  padding: 28px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
  color: var(--ink-3);
  font-size: 14px;
}

.state-block--error {
  border-color: #f7dada;
  background: var(--rose-soft);
  color: #bf3a40;
}

.state-block p {
  margin: 0 0 14px;
}

/* ==================== 头部 ==================== */

.detail-header {
  margin-bottom: 20px;
}

.title-row {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.type-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 46px;
  height: 46px;

  border-radius: 13px;

  background: var(--brand-soft);
  color: var(--brand);
}

.type-icon svg {
  width: 22px;
  height: 22px;
}

.title-block {
  min-width: 0;
  flex: 1;
}

.name-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.name-row h1 {
  margin: 0;

  color: var(--ink-1);
  font-size: 25px;
  font-weight: 700;
  letter-spacing: -0.4px;
  overflow-wrap: anywhere;
}

.subtitle {
  margin: 7px 0 0;
  color: var(--ink-3);
  font-size: 13px;
}

.dot {
  margin: 0 4px;
  color: #c3cad6;
}

.header-actions {
  display: flex;
  flex-shrink: 0;
  gap: 9px;
}

.ghost-btn,
.primary-btn {
  min-height: 38px;
  padding: 0 16px;

  border-radius: var(--radius-pill);

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease, border-color 0.2s ease;
}

.ghost-btn {
  border: 1px solid var(--line-strong);
  background: #fff;
  color: var(--ink-2);
}

.ghost-btn:hover {
  background: var(--surface-muted);
}

.primary-btn {
  border: 1px solid transparent;
  background: var(--brand);
  color: #fff;
}

.primary-btn:hover {
  background: var(--brand-strong);
}

.ghost-btn:focus-visible,
.primary-btn:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 2px;
}

/* ==================== 提示 ==================== */

.notice {
  padding: 10px 14px;
  margin: 0 0 16px;

  border: 1px solid #d7ecdf;
  border-radius: 10px;

  background: #f2fbf6;
  color: #157a52;

  font-size: 13px;
}

.error-banner {
  padding: 10px 14px;
  margin: 0 0 16px;

  border: 1px solid #f7dada;
  border-radius: 10px;

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 13px;
}

/* ==================== 布局 ==================== */

.detail-grid {
  display: grid;
  align-items: start;
  gap: 18px;
  grid-template-columns: minmax(0, 1.55fr) minmax(300px, 1fr);
}

.side-column {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

@media (max-width: 900px) {
  .detail-grid {
    grid-template-columns: 1fr;
  }

  .title-row {
    flex-wrap: wrap;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions > button {
    flex: 1;
  }
}

@media (max-width: 600px) {
  .detail-page {
    padding: 20px 16px 36px;
  }

  .card {
    padding: 18px;
  }
}
</style>
