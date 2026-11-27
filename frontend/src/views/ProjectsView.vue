<script setup lang="ts">
import {onMounted, ref} from "vue";
import ProjectCard from "../components/projects/ProjectCard.vue";
import ProjectFormDialog from "../components/projects/ProjectFormDialog.vue";
import ProjectGridSkeleton from "../components/projects/ProjectGridSkeleton.vue";
import ProjectsEmptyState from "../components/projects/ProjectsEmptyState.vue";
import {useProjectsStore} from "../stores/projects";
import {PROJECT_STATUS_OPTIONS} from "./projects/project-rules";

const store = useProjectsStore();

const showCreate = ref(false);

async function handleToggleArchived(event: Event): Promise<void> {
  const target = event.target as HTMLInputElement;

  await store.setIncludeArchived(target.checked);
}

async function clearFilter(): Promise<void> {
  await store.setStatus("all");
}

onMounted(() => {
  void store.load();
});
</script>

<template>
  <main class="projects-page">
    <div class="projects-container">
      <header class="page-header">
        <div>
          <h1>学习项目</h1>
          <p>用一个项目装下一个目标，管理它的阶段、重要程度与截止约束。</p>
        </div>

        <button class="create-btn" type="button" @click="showCreate = true">
          <span aria-hidden="true" class="plus-icon">+</span>
          新建项目
        </button>
      </header>

      <section class="filter-bar" aria-label="项目筛选">
        <div class="status-tabs">
          <button
            v-for="option in PROJECT_STATUS_OPTIONS"
            :key="option.value"
            :class="{'tab--active': store.status === option.value}"
            class="tab"
            type="button"
            @click="store.setStatus(option.value)"
          >
            {{ option.label }}
          </button>
        </div>

        <label v-if="store.status === 'all'" class="archive-toggle">
          <input
            :checked="store.includeArchived"
            type="checkbox"
            @change="handleToggleArchived"
          />
          包含已归档
        </label>
      </section>

      <p v-if="store.error" class="error-banner" role="alert">
        <span class="error-icon">!</span>
        <span>{{ store.error }}</span>
      </p>

      <p v-if="!store.loading && store.total > 0" class="result-summary">
        共 <strong>{{ store.total }}</strong> 个项目
      </p>

      <ProjectGridSkeleton v-if="store.loading && store.items.length === 0"/>

      <section v-else-if="store.items.length > 0" class="project-grid">
        <ProjectCard v-for="project in store.items" :key="project.id" :project="project"/>
      </section>

      <ProjectsEmptyState
        v-else
        :filtered="store.status !== 'all' || store.includeArchived"
        @create="showCreate = true"
        @reset="clearFilter"
      />

      <footer v-if="store.total > 0" class="pagination-footer">
        <span class="pagination-info">
          第 <strong>{{ store.page }}</strong> / {{ store.totalPages }} 页 · 每页 12 个
        </span>

        <nav class="pagination" aria-label="项目分页">
          <button
            :disabled="store.page <= 1 || store.loading"
            class="page-btn"
            type="button"
            @click="store.goToPage(store.page - 1)"
          >
            <span aria-hidden="true">‹</span>
            上一页
          </button>

          <button
            :disabled="store.page >= store.totalPages || store.loading"
            class="page-btn"
            type="button"
            @click="store.goToPage(store.page + 1)"
          >
            下一页
            <span aria-hidden="true">›</span>
          </button>
        </nav>
      </footer>
    </div>

    <ProjectFormDialog :open="showCreate" :project="null" @close="showCreate = false"/>
  </main>
</template>

<style scoped>
.projects-page {
  min-height: 100%;
  padding: 40px 28px 48px;
  background: #f5f7fb;
  color: #263248;
}

.projects-container {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

/* ==================== 页头 ==================== */

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;

  margin-bottom: 22px;
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

.create-btn {
  display: inline-flex;
  flex-shrink: 0;
  min-height: 42px;
  align-items: center;
  gap: 8px;
  padding: 0 18px;

  border: none;
  border-radius: 9px;

  background: #4f46e5;
  box-shadow: 0 3px 10px rgb(79 70 229 / 22%);
  color: #fff;

  font-size: 14px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease, transform 0.2s ease;
}

.create-btn:hover {
  background: #4338ca;
  transform: translateY(-1px);
}

.create-btn:focus-visible,
.tab:focus-visible,
.page-btn:focus-visible,
.ghost-btn:focus-visible {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}

.plus-icon {
  font-size: 20px;
  font-weight: 400;
  line-height: 1;
}

/* ==================== 筛选 ==================== */

.filter-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 14px;

  padding: 8px 12px;
  margin-bottom: 20px;

  border: 1px solid #e9edf4;
  border-radius: 12px;

  background: #fff;
}

.status-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tab {
  padding: 7px 14px;

  border: none;
  border-radius: 8px;

  background: transparent;
  color: #6b7488;

  font-size: 13px;
  font-weight: 500;
  cursor: pointer;

  transition: background-color 0.18s ease, color 0.18s ease;
}

.tab:hover {
  background: #f4f6fc;
  color: #4f46e5;
}

.tab--active {
  background: #eef1fe;
  color: #4f46e5;
  font-weight: 600;
}

.archive-toggle {
  display: inline-flex;
  align-items: center;
  gap: 7px;

  padding-right: 6px;

  color: #7d8798;
  font-size: 13px;
  cursor: pointer;
  user-select: none;
}

.archive-toggle input {
  width: 15px;
  height: 15px;
  accent-color: #4f46e5;
  cursor: pointer;
}

/* ==================== 提示 ==================== */

.error-banner {
  display: flex;
  align-items: center;
  gap: 9px;

  padding: 11px 14px;
  margin: 0 0 18px;

  border: 1px solid #f6d5d5;
  border-radius: 10px;

  background: #fff6f6;
  color: #c04141;

  font-size: 13px;
}

.error-icon {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 16px;
  height: 16px;

  border-radius: 50%;

  background: #e05252;
  color: #fff;

  font-size: 11px;
  font-weight: 700;
}

.result-summary {
  margin: 0 0 14px;
  color: #98a1b2;
  font-size: 13px;
}

.result-summary strong {
  color: #4f46e5;
}

/* ==================== 卡片网格 ==================== */

.project-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
}

/* ==================== 分页 ==================== */

.pagination-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding-top: 22px;
  margin-top: 24px;

  border-top: 1px solid #eceff5;
}

.pagination-info {
  color: #98a1b2;
  font-size: 13px;
}

.pagination-info strong {
  color: #4a5568;
}

.pagination {
  display: flex;
  gap: 8px;
}

.page-btn {
  display: inline-flex;
  min-height: 36px;
  align-items: center;
  gap: 5px;
  padding: 0 14px;

  border: 1px solid #dce2eb;
  border-radius: 8px;

  background: #fff;
  color: #596579;

  font-size: 13px;
  cursor: pointer;

  transition: background-color 0.18s ease, border-color 0.18s ease;
}

.page-btn:hover:not(:disabled) {
  border-color: #c5ccda;
  background: #f8f9fc;
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .projects-page {
    padding: 24px 18px 36px;
  }

  .page-header {
    flex-direction: column;
    align-items: stretch;
  }

  .create-btn {
    justify-content: center;
  }

  .project-grid {
    grid-template-columns: 1fr;
  }
}
</style>
