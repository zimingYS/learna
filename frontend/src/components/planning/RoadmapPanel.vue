<script setup lang="ts">
import {computed, ref} from "vue";
import type {RoadmapItem} from "../../api/planning";
import {usePlanningStore} from "../../stores/planning";
import {formatDateTime} from "../../lib/datetime";
import PlanStatusPill from "./PlanStatusPill.vue";
import RoadmapFormDialog from "./RoadmapFormDialog.vue";
import ModalDialog from "../ui/ModalDialog.vue";
import {
  canConfirm,
  canDeleteStructure,
  canEditStructure,
  ROADMAP_SOURCE_LABELS,
  ROADMAP_STATUS_LABELS,
} from "../../views/planning/planning-rules";

const store = usePlanningStore();

const formOpen = ref(false);
const editing = ref<RoadmapItem | null>(null);
const pendingDelete = ref<RoadmapItem | null>(null);

const roadmaps = computed(() => store.roadmaps);

function openCreate(): void {
  editing.value = null;
  formOpen.value = true;
}

function openEdit(roadmap: RoadmapItem): void {
  editing.value = roadmap;
  formOpen.value = true;
}

async function confirm(roadmap: RoadmapItem): Promise<void> {
  await store.activateRoadmap(roadmap.id);
}

async function remove(): Promise<void> {
  const roadmap = pendingDelete.value;

  if (!roadmap) {
    return;
  }

  const deleted = await store.removeRoadmap(roadmap.id);

  if (deleted !== null) {
    pendingDelete.value = null;
  }
}
</script>

<template>
  <section class="panel">
    <header class="panel-head">
      <div>
        <h2 class="panel-title">路线版本</h2>
        <p class="panel-sub">
          每个版本是一份完整规划。确认后旧版本自动废弃，历史版本只读保留。
        </p>
      </div>

      <button class="create-btn" type="button" @click="openCreate()">
        <span aria-hidden="true" class="plus">+</span>
        新建版本
      </button>
    </header>

    <p v-if="store.roadmapsLoading" class="hint">正在加载路线…</p>

    <p v-else-if="roadmaps.length === 0" class="empty-box">
      还没有路线。新建一个草稿版本，把阶段排好后确认生效。
    </p>

    <template v-else>
      <ul class="roadmap-list">
        <li
          v-for="roadmap in roadmaps"
          :key="roadmap.id"
          class="roadmap-item"
          :class="{
            'roadmap-item--active': roadmap.id === store.activeRoadmapId,
            'roadmap-item--muted': roadmap.status === 'superseded',
          }"
        >
          <button class="roadmap-main" type="button" @click="store.selectRoadmap(roadmap.id)">
            <span class="version">v{{ roadmap.version }}</span>

            <span class="roadmap-body">
              <span class="roadmap-line">
                <PlanStatusPill
                  :label="ROADMAP_STATUS_LABELS[roadmap.status]"
                  :status="roadmap.status"
                />
                <span class="source">{{ ROADMAP_SOURCE_LABELS[roadmap.source] }}</span>
              </span>

              <span class="summary">{{ roadmap.change_summary ?? "（未填写变更说明）" }}</span>

              <span class="time">
                创建 {{ formatDateTime(roadmap.created_at) }}
                <template v-if="roadmap.confirmed_at">
                  · 确认 {{ formatDateTime(roadmap.confirmed_at) }}
                </template>
              </span>
            </span>
          </button>

          <div class="roadmap-actions">
            <button
              v-if="canConfirm(roadmap.status)"
              class="ghost-btn ghost-btn--primary"
              type="button"
              @click="confirm(roadmap)"
            >
              确认生效
            </button>

            <button
              v-if="canEditStructure(roadmap.status)"
              class="ghost-btn"
              type="button"
              @click="openEdit(roadmap)"
            >
              编辑说明
            </button>

            <button
              v-if="canDeleteStructure(roadmap.status)"
              class="ghost-btn ghost-btn--danger"
              type="button"
              @click="pendingDelete = roadmap"
            >
              删除
            </button>

            <span v-if="roadmap.status === 'superseded'" class="readonly-tag">只读历史</span>
          </div>
        </li>
      </ul>

      <footer v-if="store.roadmapsTotal > 0" class="pagination-footer">
        <span class="pagination-info">
          第 <strong>{{ store.roadmapsPage }}</strong> / {{ store.roadmapTotalPages }} 页 · 共
          {{ store.roadmapsTotal }} 个版本
        </span>

        <nav class="pagination">
          <button
            :disabled="store.roadmapsPage <= 1 || store.roadmapsLoading"
            class="page-btn"
            type="button"
            @click="store.goToRoadmapPage(store.roadmapsPage - 1)"
          >
            ‹ 上一页
          </button>

          <button
            :disabled="store.roadmapsPage >= store.roadmapTotalPages || store.roadmapsLoading"
            class="page-btn"
            type="button"
            @click="store.goToRoadmapPage(store.roadmapsPage + 1)"
          >
            下一页 ›
          </button>
        </nav>
      </footer>
    </template>

    <RoadmapFormDialog
      :open="formOpen"
      :roadmap="editing"
      @close="formOpen = false"
      @saved="formOpen = false"
    />

    <ModalDialog :open="pendingDelete !== null" title="删除路线草稿" @close="pendingDelete = null">
      <div class="pf-form">
        <p class="confirm-text">
          将删除 v{{ pendingDelete?.version }} 草稿及其下全部阶段与里程碑。任务不会删除，只会解除阶段与里程碑归属。
        </p>

        <div class="pf-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="pendingDelete = null">
            取消
          </button>
          <button class="pf-btn pf-btn--danger" type="button" @click="remove()">确认删除</button>
        </div>
      </div>
    </ModalDialog>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.panel-title {
  margin: 0;
  color: var(--ink-1);
  font-size: 17px;
  font-weight: 650;
}

.panel-sub {
  margin: 6px 0 0;
  color: var(--ink-3);
  font-size: 13px;
  line-height: 1.6;
}

.create-btn {
  display: inline-flex;
  flex-shrink: 0;
  min-height: 38px;
  align-items: center;
  gap: 7px;
  padding: 0 15px;

  border: none;
  border-radius: var(--radius-pill);

  background: var(--brand);
  color: #fff;

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease;
}

.create-btn:hover {
  background: var(--brand-strong);
}

.plus {
  font-size: 17px;
  line-height: 1;
}

.hint {
  margin: 0;
  color: var(--ink-3);
  font-size: 13px;
}

.empty-box {
  padding: 34px 20px;
  margin: 0;

  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-md);

  color: var(--ink-3);
  font-size: 13px;
  text-align: center;
}

.roadmap-list {
  display: flex;
  flex-direction: column;
  gap: 10px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.roadmap-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;

  padding: 13px 16px;

  border: 1px solid var(--line);
  border-radius: var(--radius-md);

  background: #fff;

  transition: border-color 0.18s ease, box-shadow 0.18s ease;
}

.roadmap-item--active {
  border-color: #c9cffa;
  box-shadow: 0 0 0 3px rgb(79 70 229 / 8%);
}

.roadmap-item--muted {
  background: var(--surface-muted);
}

.roadmap-main {
  display: flex;
  flex: 1;
  min-width: 0;
  align-items: flex-start;
  gap: 12px;

  padding: 0;

  border: none;

  background: transparent;
  text-align: left;
  cursor: pointer;
}

.version {
  flex-shrink: 0;

  padding: 3px 9px;

  border-radius: 6px;

  background: var(--surface-muted);
  color: var(--ink-2);

  font-size: 12px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.roadmap-body {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}

.roadmap-line {
  display: flex;
  align-items: center;
  gap: 9px;
}

.source {
  color: var(--ink-3);
  font-size: 12px;
}

.summary {
  overflow: hidden;

  color: var(--ink-2);
  font-size: 13px;
  line-height: 1.6;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.time {
  color: var(--ink-4);
  font-size: 12px;
}

.roadmap-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 8px;
}

.ghost-btn {
  padding: 6px 11px;

  border: 1px solid var(--line-strong);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-2);

  font-size: 12px;
  cursor: pointer;

  transition: background-color 0.18s ease, border-color 0.18s ease;
}

.ghost-btn:hover {
  border-color: var(--line-strong);
  background: var(--surface-muted);
}

.ghost-btn--primary {
  border-color: #cfd4fb;
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.ghost-btn--primary:hover {
  background: #e3e7fd;
}

.ghost-btn--danger {
  border-color: #f7dada;
  color: #bf3a40;
}

.ghost-btn--danger:hover {
  border-color: #e8c3c3;
  background: var(--rose-soft);
}

.readonly-tag {
  color: var(--ink-4);
  font-size: 12px;
}

.pagination-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding-top: 16px;

  border-top: 1px solid var(--line);
}

.pagination-info {
  color: var(--ink-3);
  font-size: 12px;
}

.pagination-info strong {
  color: var(--ink-2);
}

.pagination {
  display: flex;
  gap: 8px;
}

.page-btn {
  min-height: 32px;
  padding: 0 12px;

  border: 1px solid var(--line-strong);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-2);

  font-size: 12px;
  cursor: pointer;
}

.page-btn:hover:not(:disabled) {
  border-color: var(--line-strong);
  background: var(--surface-muted);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirm-text {
  margin: 0;
  color: var(--ink-2);
  font-size: 14px;
  line-height: 1.7;
}
</style>
