<script setup lang="ts">
import {ref} from "vue";
import {formatDateTime} from "../../lib/datetime";
import StatusPill from "../ui/StatusPill.vue";
import {useSessionsStore} from "../../stores/sessions";
import {
  ACTIVITY_TYPE_FILTER_OPTIONS,
  ACTIVITY_TYPE_LABELS,
  activityTone,
  formatMinutes,
  formatTimeRange,
} from "../../views/sessions/session-rules";

const emit = defineEmits<{ create: []; open: [id: string] }>();

const store = useSessionsStore();

const draftFrom = ref(store.fromDate);
const draftTo = ref(store.toDate);

function onTypeChange(event: Event): void {
  void store.setType((event.target as HTMLSelectElement).value as typeof store.typeFilter);
}

function onProjectChange(event: Event): void {
  void store.setProject((event.target as HTMLSelectElement).value);
}

function applyRange(): void {
  void store.setRange(draftFrom.value, draftTo.value);
}

function clearRange(): void {
  draftFrom.value = "";
  draftTo.value = "";
  void store.setRange("", "");
}
</script>

<template>
  <section class="list-panel">
    <header class="panel-head">
      <div>
        <h2>学习活动</h2>
        <p>共 {{ store.total }} 次 · 按开始时间倒序</p>
      </div>

      <button class="pf-btn pf-btn--primary create-btn" type="button" @click="emit('create')">
        记录一次学习
      </button>
    </header>

    <div class="filter-bar">
      <select
        :value="store.typeFilter"
        aria-label="按活动类型筛选"
        class="pf-input filter-select"
        @change="onTypeChange"
      >
        <option
          v-for="option in ACTIVITY_TYPE_FILTER_OPTIONS"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>

      <select
        :value="store.projectFilter"
        aria-label="按主要项目筛选"
        class="pf-input filter-select"
        @change="onProjectChange"
      >
        <option value="">全部项目</option>
        <option v-for="project in store.projectOptions" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>

      <div class="range-row">
        <input v-model="draftFrom" aria-label="开始日期" class="pf-input" type="date"/>
        <span aria-hidden="true" class="range-sep">→</span>
        <input v-model="draftTo" aria-label="结束日期" class="pf-input" type="date"/>
        <button class="pf-btn pf-btn--ghost" type="button" @click="applyRange">筛选</button>
        <button
          v-if="store.fromDate || store.toDate"
          class="link-btn"
          type="button"
          @click="clearRange"
        >
          清除
        </button>
      </div>
    </div>

    <p v-if="store.loading" class="panel-state">加载中…</p>

    <p v-else-if="store.items.length === 0" class="panel-state">
      还没有学习活动。点右上角「记录一次学习」，或用计时器开始计时。
    </p>

    <ul v-else class="session-list">
      <li v-for="item in store.items" :key="item.id">
        <article :class="['session-card', {'session-card--active': store.selected?.id === item.id}]">
          <button class="card-main" type="button" @click="emit('open', item.id)">
            <span class="card-title">{{ item.title || "未命名活动" }}</span>

            <span class="card-meta">
              <StatusPill
                :label="ACTIVITY_TYPE_LABELS[item.activity_type]"
                :tone="activityTone(item.activity_type)"
              />
              <span class="card-when">{{ formatTimeRange(item.started_at, item.ended_at) }}</span>
            </span>

            <span class="card-foot">
              <span class="card-duration">{{ formatMinutes(item.duration_minutes) }}</span>
              <span class="card-project">{{ store.projectName(item.primary_project_id) }}</span>
              <span class="card-created">{{ formatDateTime(item.created_at) }} 录入</span>
            </span>
          </button>
        </article>
      </li>
    </ul>

    <nav v-if="store.totalPages > 1" aria-label="分页" class="pager">
      <button
        :disabled="store.page <= 1"
        class="pf-btn pf-btn--ghost"
        type="button"
        @click="store.setPage(store.page - 1)"
      >
        上一页
      </button>
      <span class="pager-info">{{ store.page }} / {{ store.totalPages }}</span>
      <button
        :disabled="store.page >= store.totalPages"
        class="pf-btn pf-btn--ghost"
        type="button"
        @click="store.setPage(store.page + 1)"
      >
        下一页
      </button>
    </nav>
  </section>
</template>

<style scoped>
.list-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;

  padding: 20px 22px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.panel-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.panel-head h2 {
  margin: 0;
  color: var(--ink-1);
  font-size: 16px;
  font-weight: 680;
}

.panel-head p {
  margin: 5px 0 0;
  color: var(--ink-4);
  font-size: 12px;
}

.create-btn {
  min-height: 34px;
  padding: 0 14px;
  font-size: 12.5px;
}

.filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-select {
  width: auto;
  min-width: 108px;
  height: 34px;
  padding: 0 10px;
  font-size: 12.5px;
}

.range-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.range-row .pf-input {
  width: 138px;
  height: 34px;
  padding: 0 8px;
  font-size: 12.5px;
}

.range-sep {
  color: var(--ink-4);
  font-size: 12px;
}

.range-row .pf-btn {
  min-height: 34px;
  padding: 0 12px;
  font-size: 12.5px;
}

.link-btn {
  padding: 0 4px;

  border: none;
  background: transparent;
  color: var(--ink-4);

  font-family: inherit;
  font-size: 12px;
  text-decoration: underline;
  cursor: pointer;
}

.panel-state {
  padding: 40px 16px;
  margin: 0;

  color: var(--ink-4);
  font-size: 13px;
  line-height: 1.8;
  text-align: center;
}

.session-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.session-card {
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: #fff;

  transition: border-color 0.18s var(--ease), box-shadow 0.18s var(--ease);
}

.session-card:hover {
  border-color: var(--line-strong);
  box-shadow: var(--shadow-xs);
}

.session-card--active {
  border-color: var(--brand);
  box-shadow: 0 0 0 3px var(--brand-soft);
}

.card-main {
  display: flex;
  flex-direction: column;
  gap: 8px;

  width: 100%;
  padding: 12px 14px;

  border: none;
  background: transparent;

  font-family: inherit;
  text-align: left;
  cursor: pointer;
}

.card-title {
  color: var(--ink-1);
  font-size: 13.5px;
  font-weight: 620;
  line-height: 1.4;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.card-when {
  color: var(--ink-2);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.card-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;

  color: var(--ink-4);
  font-size: 11.5px;
}

.card-duration {
  color: var(--brand);
  font-weight: 600;
}

.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.pager .pf-btn {
  min-height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.pager-info {
  color: var(--ink-4);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}
</style>
