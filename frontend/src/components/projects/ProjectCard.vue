<script setup lang="ts">
import {computed} from "vue";
import {RouterLink} from "vue-router";
import type {ProjectItem} from "../../api/projects";
import {
  daysUntil,
  formatDateTime,
  PROJECT_TYPE_ICONS,
  PROJECT_TYPE_LABELS,
} from "../../views/projects/project-rules";
import ProjectStatusBadge from "./ProjectStatusBadge.vue";

const props = defineProps<{ project: ProjectItem }>();

const iconPaths = computed(() => PROJECT_TYPE_ICONS[props.project.project_type]);
const typeLabel = computed(() => PROJECT_TYPE_LABELS[props.project.project_type]);
const isArchived = computed(() => props.project.status === "archived");

const deadlineDays = computed(() => daysUntil(props.project.deadline_at));

const deadlineText = computed(() => {
  if (props.project.deadline_type === "none" || props.project.deadline_at === null) {
    return "无截止日期";
  }

  const days = deadlineDays.value;

  if (days === null) {
    return "无截止日期";
  }

  if (days < 0) {
    return `已逾期 ${Math.abs(days)} 天`;
  }

  return days === 0 ? "今天截止" : `距截止 ${days} 天`;
});

const isOverdue = computed(() => {
  const days = deadlineDays.value;

  return props.project.deadline_type !== "none" && days !== null && days < 0;
});
</script>

<template>
  <RouterLink :class="{'project-card--archived': isArchived}" :to="`/projects/${project.id}`"
              class="project-card">
    <div class="card-head">
      <div class="type-icon">
        <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
             stroke-linejoin="round" stroke-width="1.7" viewBox="0 0 24 24">
          <path v-for="path in iconPaths" :key="path" :d="path"/>
        </svg>
      </div>

      <div class="title-block">
        <h3 class="project-name">{{ project.name }}</h3>
        <p class="project-type">{{ typeLabel }}</p>
      </div>

      <ProjectStatusBadge :status="project.status"/>
    </div>

    <p v-if="project.description" class="project-desc">{{ project.description }}</p>
    <p v-else class="project-desc project-desc--empty">还没有填写项目描述</p>

    <dl class="card-meta">
      <div class="meta-item">
        <dt>重要程度</dt>
        <dd class="importance">
          <span class="importance-badge">I{{ project.importance_level }}</span>
        </dd>
      </div>

      <div class="meta-item">
        <dt>截止</dt>
        <dd :class="{'meta-overdue': isOverdue}">
          <span>{{ deadlineText }}</span>
          <span v-if="project.deadline_at" class="meta-date">
            {{ formatDateTime(project.deadline_at) }}
          </span>
        </dd>
      </div>
    </dl>

    <div class="card-foot">
      <span class="updated-at">更新于 {{ formatDateTime(project.updated_at) }}</span>
      <span class="enter-link">查看详情<span aria-hidden="true">→</span></span>
    </div>
  </RouterLink>
</template>

<style scoped>
.project-card {
  display: flex;
  flex-direction: column;
  gap: 14px;

  padding: 20px 20px 16px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
  box-shadow: var(--shadow-sm);

  color: inherit;
  text-decoration: none;

  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.project-card:hover {
  border-color: #d9dff2;
  box-shadow: 0 10px 24px rgb(31 45 72 / 9%);
  transform: translateY(-2px);
}

.project-card:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 3px;
}

.project-card--archived {
  background: var(--surface-muted);
}

.card-head {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.type-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 40px;
  height: 40px;

  border-radius: 11px;

  background: var(--brand-soft);
  color: var(--brand);
}

.type-icon svg {
  width: 20px;
  height: 20px;
}

.title-block {
  min-width: 0;
  flex: 1;
}

.project-name {
  overflow: hidden;

  margin: 0;

  color: var(--ink-1);
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.2px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-type {
  margin: 4px 0 0;
  color: var(--ink-3);
  font-size: 12px;
}

.project-desc {
  display: -webkit-box;
  overflow: hidden;
  min-height: 40px;
  margin: 0;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;

  color: var(--ink-3);
  font-size: 13px;
  line-height: 1.65;
}

.project-desc--empty {
  color: var(--ink-4);
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;

  padding: 12px 0 0;
  margin: 0;

  border-top: 1px dashed var(--line);
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 7px;

  font-size: 12px;
}

.meta-item dt {
  color: var(--ink-3);
}

.meta-item dd {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;

  color: var(--ink-2);
  font-weight: 600;
}

.importance-badge {
  padding: 2px 7px;

  border-radius: 5px;

  background: var(--violet-soft);
  color: #6d4bd8;

  font-size: 11px;
  font-weight: 700;
}

.meta-date {
  color: var(--ink-4);
  font-weight: 400;
}

.meta-overdue {
  color: var(--rose) !important;
}

.card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.updated-at {
  color: var(--ink-4);
  font-size: 12px;
}

.enter-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;

  color: var(--brand);
  font-size: 12px;
  font-weight: 600;
}
</style>
