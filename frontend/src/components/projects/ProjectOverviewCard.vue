<script setup lang="ts">
import {computed} from "vue";
import type {ProjectItem} from "../../api/projects";
import {
  daysUntil,
  DEADLINE_TYPE_LABELS,
  formatDateTime,
  IMPORTANCE_LABELS,
  PROJECT_TYPE_LABELS,
} from "../../views/projects/project-rules";

const props = defineProps<{ project: ProjectItem }>();

const deadlineText = computed(() => {
  if (props.project.deadline_type === "none") {
    return null;
  }

  const days = daysUntil(props.project.deadline_at);

  if (days === null) {
    return null;
  }

  if (days < 0) {
    return `已逾期 ${Math.abs(days)} 天`;
  }

  return days === 0 ? "今天截止" : `距截止 ${days} 天`;
});
</script>

<template>
  <section class="card" aria-labelledby="overview-title">
    <h2 id="overview-title">项目概览</h2>

    <p v-if="project.description" class="description">{{ project.description }}</p>
    <p v-else class="description description--empty">还没有填写项目描述。</p>

    <dl class="info-grid">
      <div class="info-item">
        <dt>项目类型</dt>
        <dd>{{ PROJECT_TYPE_LABELS[project.project_type] }}</dd>
      </div>

      <div class="info-item">
        <dt>重要程度</dt>
        <dd>{{ IMPORTANCE_LABELS[project.importance_level] }}</dd>
      </div>

      <div class="info-item">
        <dt>重要程度覆盖至</dt>
        <dd>{{ formatDateTime(project.importance_override_until) }}</dd>
      </div>

      <div class="info-item">
        <dt>截止类型</dt>
        <dd>{{ DEADLINE_TYPE_LABELS[project.deadline_type] }}</dd>
      </div>

      <div class="info-item">
        <dt>截止时间</dt>
        <dd>
          {{ formatDateTime(project.deadline_at) }}
          <span v-if="deadlineText" class="deadline-hint">{{ deadlineText }}</span>
        </dd>
      </div>

      <div class="info-item">
        <dt>创建时间</dt>
        <dd>{{ formatDateTime(project.created_at) }}</dd>
      </div>

      <div class="info-item">
        <dt>最近更新</dt>
        <dd>{{ formatDateTime(project.updated_at) }}</dd>
      </div>

      <div class="info-item">
        <dt>最近归档</dt>
        <dd>{{ formatDateTime(project.archived_at) }}</dd>
      </div>

      <div class="info-item">
        <dt>最近重新开启</dt>
        <dd>{{ formatDateTime(project.reopened_at) }}</dd>
      </div>
    </dl>
  </section>
</template>

<style scoped>
.card {
  padding: 22px 24px;

  border: 1px solid #e9edf4;
  border-radius: 14px;

  background: #fff;
  box-shadow: 0 2px 10px rgb(31 45 72 / 4%);
}

.card h2 {
  margin: 0 0 14px;

  color: #263248;
  font-size: 16px;
  font-weight: 650;
}

.description {
  padding: 14px 16px;
  margin: 0 0 20px;

  border-radius: 10px;

  background: #f8f9fd;
  color: #4a5568;

  font-size: 13.5px;
  line-height: 1.8;
  white-space: pre-wrap;
}

.description--empty {
  color: #a6aebc;
}

.info-grid {
  display: grid;
  gap: 14px 20px;
  margin: 0;
  grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
}

.info-item dt {
  color: #98a1b2;
  font-size: 12px;
}

.info-item dd {
  margin: 5px 0 0;

  color: #364152;
  font-size: 13.5px;
  font-weight: 600;
}

.deadline-hint {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 7px;

  border-radius: 5px;

  background: #eef1fe;
  color: #4f46e5;

  font-size: 11px;
  font-weight: 600;
}

@media (max-width: 600px) {
  .card {
    padding: 18px;
  }
}
</style>
