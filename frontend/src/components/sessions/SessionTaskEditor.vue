<script setup lang="ts">
import {computed, ref, watch} from "vue";
import {useSessionsStore, type TaskOption} from "../../stores/sessions";
import {buildTaskItems, type TaskDraftRow} from "../../views/sessions/session-forms";
import {formatMinutes} from "../../views/sessions/session-rules";

const store = useSessionsStore();

const rows = ref<TaskDraftRow[]>([]);
const options = ref<TaskOption[]>([]);
const localError = ref("");
const saved = ref(false);

const duration = computed(() => store.selected?.duration_minutes ?? 0);

const allowedProjectIds = computed(() => {
  const session = store.selected;

  if (!session) {
    return [];
  }

  return [
    ...(session.primary_project_id ? [session.primary_project_id] : []),
    ...session.related_project_ids,
  ];
});

const total = computed(() =>
  rows.value.reduce((sum, row) => sum + (Number(row.minutes) || 0), 0),
);

watch(
  () => store.selected,
  (session) => {
    rows.value = (session?.tasks ?? []).map((item) => ({
      taskId: item.task_id,
      minutes: String(item.contribution_minutes),
      note: item.note ?? "",
    }));
    localError.value = "";
    saved.value = false;
  },
  {immediate: true},
);

// 任务按项目懒加载：只有本次活动的项目下的任务才可能出现在分摊里
watch(
  allowedProjectIds,
  async (projectIds) => {
    const loaded = await Promise.all(projectIds.map((id) => store.loadTasks(id)));
    options.value = loaded.flat();
  },
  {immediate: true},
);

function add(): void {
  const used = new Set(rows.value.map((row) => row.taskId));
  const next = options.value.find((option) => !used.has(option.id));

  rows.value = [...rows.value, {taskId: next?.id ?? "", minutes: "0", note: ""}];
  saved.value = false;
}

function remove(index: number): void {
  rows.value = rows.value.filter((_, position) => position !== index);
  saved.value = false;
}

function taskOptionLabel(option: TaskOption): string {
  return `${option.title} · ${store.projectName(option.projectId)}`;
}

async function save(): Promise<void> {
  const built = buildTaskItems(rows.value, duration.value);

  if (!built.ok) {
    localError.value = built.error;
    return;
  }

  localError.value = "";
  saved.value = await store.replaceTasks(built.value);
}
</script>

<template>
  <section class="editor">
    <header class="editor-head">
      <div>
        <h3>任务分摊</h3>
        <p>
          分摊只是归因，不会叠加成第二个时长；合计不得超过 {{ formatMinutes(duration) }}。
          任务完成仍需在任务页显式操作，记录学习不会自动完成任务。
        </p>
      </div>

      <div class="head-actions">
        <button
          :disabled="allowedProjectIds.length === 0"
          class="pf-btn pf-btn--ghost"
          type="button"
          @click="add"
        >
          加一条
        </button>
        <button class="pf-btn pf-btn--primary" type="button" @click="save">保存</button>
      </div>
    </header>

    <p v-if="allowedProjectIds.length === 0" class="editor-state">
      先给这次活动选上主要项目或关联项目，才能分摊任务。
    </p>

    <p v-else-if="rows.length === 0" class="editor-state">
      没有分摊。本次学习也可以纯粹归因到项目，不必挂在具体任务上。
    </p>

    <ul v-else class="row-list">
      <li v-for="(row, index) in rows" :key="index" class="row">
        <select v-model="row.taskId" aria-label="选择任务" class="pf-input row-select">
          <option value="">选择任务…</option>
          <option v-for="option in options" :key="option.id" :value="option.id">
            {{ taskOptionLabel(option) }}
          </option>
        </select>

        <div class="minutes-field">
          <input
            v-model="row.minutes"
            aria-label="分摊分钟数"
            class="pf-input row-minutes"
            inputmode="numeric"
            type="number"
          />
          <span class="minutes-unit">分钟</span>
        </div>

        <input v-model="row.note" aria-label="备注" class="pf-input row-note" placeholder="备注（可选）"/>

        <button class="row-remove" type="button" @click="remove(index)">移除</button>
      </li>
    </ul>

    <p v-if="rows.length > 0" class="alloc-total">
      合计 {{ total }} 分钟 / 有效时长 {{ duration }} 分钟
    </p>

    <p v-if="localError" class="pf-error">{{ localError }}</p>
    <p v-else-if="saved" class="editor-saved">已保存</p>
  </section>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  gap: 10px;

  padding: 18px 20px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.editor-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.editor-head h3 {
  margin: 0;
  color: var(--ink-1);
  font-size: 14px;
  font-weight: 660;
}

.editor-head p {
  max-width: 62ch;
  margin: 5px 0 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}

.head-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.head-actions .pf-btn {
  min-height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.editor-state {
  margin: 0;
  color: var(--ink-4);
  font-size: 12.5px;
  line-height: 1.7;
}

.row-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.row .pf-input {
  height: 34px;
  font-size: 12.5px;
}

.row-select {
  flex: 1 1 190px;
  min-width: 160px;
}

.minutes-field {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.row-minutes {
  width: 78px;
}

.minutes-unit {
  color: var(--ink-4);
  font-size: 11.5px;
}

.row-note {
  flex: 1 1 130px;
  min-width: 110px;
}

.row-remove {
  padding: 0 6px;

  border: none;
  background: transparent;
  color: var(--ink-4);

  font-family: inherit;
  font-size: 11.5px;
  text-decoration: underline;
  cursor: pointer;
}

.row-remove:hover {
  color: #bf3a40;
}

.alloc-total {
  margin: 0;
  color: var(--ink-3);
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
}

.editor-saved {
  margin: 0;
  color: #12855a;
  font-size: 11.5px;
}
</style>
