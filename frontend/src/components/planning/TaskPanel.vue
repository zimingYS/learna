<script setup lang="ts">
import {computed, ref} from "vue";
import type {TaskItem, TaskStatus} from "../../api/planning";
import {usePlanningStore} from "../../stores/planning";
import {formatDateTime} from "../../lib/datetime";
import ModalDialog from "../ui/ModalDialog.vue";
import PlanStatusPill from "./PlanStatusPill.vue";
import TaskDependencyDialog from "./TaskDependencyDialog.vue";
import TaskFormDialog from "./TaskFormDialog.vue";
import {
  dueLabel,
  PRIORITY_LABELS,
  TASK_STATUS_LABELS,
  TASK_STATUS_OPTIONS,
  taskStatusTargets,
} from "../../views/planning/planning-rules";

const store = usePlanningStore();

const formOpen = ref(false);
const editing = ref<TaskItem | null>(null);
const dependencyTask = ref<TaskItem | null>(null);
const dependenciesOpen = ref(false);
const pendingDelete = ref<TaskItem | null>(null);

const stageNames = computed(() => {
  const map = new Map<string, string>();

  for (const stage of store.stages) {
    map.set(stage.id, `#${stage.sequence_no} ${stage.name}`);
  }

  return map;
});

const milestoneTitles = computed(() => {
  const map = new Map<string, string>();

  for (const milestone of store.milestones) {
    map.set(milestone.id, milestone.title);
  }

  return map;
});

function openCreate(): void {
  editing.value = null;
  formOpen.value = true;
}

function openEdit(task: TaskItem): void {
  editing.value = task;
  formOpen.value = true;
}

function openDependencies(task: TaskItem): void {
  dependencyTask.value = task;
  dependenciesOpen.value = true;
}

async function changeStatus(task: TaskItem, event: Event): Promise<void> {
  const target = (event.target as HTMLSelectElement).value as TaskStatus;

  await store.setTaskStatus(task.id, target);
}

async function remove(): Promise<void> {
  const task = pendingDelete.value;

  if (!task) {
    return;
  }

  const deleted = await store.removeTask(task.id);

  if (deleted !== null) {
    pendingDelete.value = null;
  }
}

function ownerLabel(task: TaskItem): string {
  const parts: string[] = [];

  if (task.stage_id) {
    parts.push(stageNames.value.get(task.stage_id) ?? "所属阶段");
  }

  if (task.milestone_id) {
    parts.push(milestoneTitles.value.get(task.milestone_id) ?? "所属里程碑");
  }

  return parts.length > 0 ? parts.join(" · ") : "仅挂项目";
}
</script>

<template>
  <section class="panel">
    <header class="panel-head">
      <div>
        <h2 class="panel-title">学习任务</h2>
        <p class="panel-sub">
          任务可只挂项目，也可挂到阶段或里程碑。前置依赖保证顺序，删除为软删除。
        </p>
      </div>

      <button class="create-btn" type="button" @click="openCreate()">
        <span aria-hidden="true" class="plus">+</span>
        新建任务
      </button>
    </header>

    <div class="filter-bar">
      <div class="status-tabs">
        <button
          v-for="option in TASK_STATUS_OPTIONS"
          :key="option.value"
          class="tab"
          :class="{ 'tab--active': store.taskStatus === option.value }"
          type="button"
          @click="store.setTaskStatusFilter(option.value)"
        >
          {{ option.label }}
        </button>
      </div>

      <span class="total">共 {{ store.tasksTotal }} 项</span>
    </div>

    <p v-if="store.tasksLoading" class="hint">正在加载任务…</p>

    <p v-else-if="store.tasks.length === 0" class="empty-box">
      {{ store.taskStatus === "all" ? "还没有任务。拆出今天能动手的第一件事。" : "该状态下没有任务。" }}
    </p>

    <ul v-else class="task-list">
      <li v-for="task in store.tasks" :key="task.id" class="task-row">
        <div class="task-main">
          <div class="task-head">
            <span class="priority" :class="`priority--${task.priority}`">
              {{ PRIORITY_LABELS[task.priority] }}
            </span>

            <span class="task-title" :class="{ 'task-title--done': task.status === 'completed' }">
              {{ task.title }}
            </span>

            <PlanStatusPill :label="TASK_STATUS_LABELS[task.status]" :status="task.status"/>
          </div>

          <p v-if="task.description" class="task-desc">{{ task.description }}</p>

          <div class="task-meta">
            <span class="meta-text">{{ ownerLabel(task) }}</span>
            <span v-if="task.due_at" class="meta-text">
              截止 {{ formatDateTime(task.due_at) }}
            </span>
            <span
              v-if="dueLabel(task.due_at)"
              class="due-chip"
              :class="{ 'due-chip--overdue': dueLabel(task.due_at)?.overdue }"
            >
              {{ dueLabel(task.due_at)?.text }}
            </span>
            <span v-if="task.estimated_minutes !== null" class="meta-text">
              预计 {{ task.estimated_minutes }} 分钟
            </span>
          </div>
        </div>

        <div class="task-actions">
          <select :value="task.status" class="status-select" @change="changeStatus(task, $event)">
            <option :value="task.status" disabled>{{ TASK_STATUS_LABELS[task.status] }}</option>
            <option
              v-for="target in taskStatusTargets(task.status)"
              :key="target"
              :value="target"
            >
              改为{{ TASK_STATUS_LABELS[target] }}
            </option>
          </select>

          <button class="link-btn" type="button" @click="openDependencies(task)">依赖</button>
          <button class="link-btn" type="button" @click="openEdit(task)">编辑</button>
          <button class="link-btn link-btn--danger" type="button" @click="pendingDelete = task">
            删除
          </button>
        </div>
      </li>
    </ul>

    <footer v-if="store.tasksTotal > 0" class="pagination-footer">
      <span class="pagination-info">
        第 <strong>{{ store.tasksPage }}</strong> / {{ store.taskTotalPages }} 页 · 每页 20 项
      </span>

      <nav class="pagination">
        <button
          :disabled="store.tasksPage <= 1 || store.tasksLoading"
          class="page-btn"
          type="button"
          @click="store.goToTaskPage(store.tasksPage - 1)"
        >
          ‹ 上一页
        </button>

        <button
          :disabled="store.tasksPage >= store.taskTotalPages || store.tasksLoading"
          class="page-btn"
          type="button"
          @click="store.goToTaskPage(store.tasksPage + 1)"
        >
          下一页 ›
        </button>
      </nav>
    </footer>

    <TaskFormDialog
      :milestones="store.milestones"
      :open="formOpen"
      :stages="store.stages"
      :task="editing"
      @close="formOpen = false"
      @saved="formOpen = false"
    />

    <TaskDependencyDialog
      :open="dependenciesOpen"
      :task="dependencyTask"
      @close="dependenciesOpen = false"
    />

    <ModalDialog :open="pendingDelete !== null" title="删除任务" @close="pendingDelete = null">
      <div class="pf-form">
        <p class="confirm-text">
          将删除任务「{{ pendingDelete?.title }}」。这是软删除：行保留但不再出现在列表与统计中，相关依赖关系会一并解除。
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
  gap: 14px;
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
}

.create-btn:hover {
  background: var(--brand-strong);
}

.plus {
  font-size: 17px;
  line-height: 1;
}

.filter-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding: 7px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: #fff;
}

.status-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.tab {
  padding: 6px 12px;

  border: none;
  border-radius: var(--radius-pill);

  background: transparent;
  color: var(--ink-3);

  font-size: 12px;
  cursor: pointer;
}

.tab:hover {
  background: var(--surface-muted);
  color: var(--brand);
}

.tab--active {
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.total {
  padding-right: 4px;
  color: var(--ink-3);
  font-size: 12px;
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

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.task-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding: 12px 15px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: #fff;
}

.task-main {
  display: flex;
  flex: 1;
  min-width: 220px;
  flex-direction: column;
  gap: 7px;
}

.task-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 9px;
}

.priority {
  flex-shrink: 0;

  padding: 2px 7px;

  border-radius: 5px;

  background: var(--surface-muted);
  color: var(--ink-3);

  font-size: 11px;
  font-weight: 700;
}

.priority--4 {
  background: #fff3e4;
  color: #b9761f;
}

.priority--5 {
  background: var(--rose-soft);
  color: #bf3a40;
}

.task-title {
  color: var(--ink-1);
  font-size: 14px;
  font-weight: 500;
  line-height: 1.5;
}

.task-title--done {
  color: var(--ink-3);
  text-decoration: line-through;
}

.task-desc {
  margin: 0;
  color: #6f7a8d;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
}

.task-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.meta-text {
  color: var(--ink-4);
  font-size: 12px;
}

.due-chip {
  padding: 2px 7px;

  border-radius: 5px;

  background: var(--surface-muted);
  color: var(--ink-3);

  font-size: 11px;
  font-weight: 600;
}

.due-chip--overdue {
  background: var(--rose-soft);
  color: #bf3a40;
}

.task-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 9px;
}

.status-select {
  height: 30px;
  padding: 0 7px;

  border: 1px solid var(--line-strong);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-2);

  font-family: inherit;
  font-size: 12px;
}

.link-btn {
  padding: 0;

  border: none;

  background: transparent;
  color: var(--brand);

  font-size: 12px;
  cursor: pointer;
}

.link-btn:hover {
  text-decoration: underline;
}

.link-btn--danger {
  color: #bf3a40;
}

.pagination-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding-top: 14px;

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
