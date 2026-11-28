<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {TaskDependencyItem, TaskItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";
import {TASK_STATUS_LABELS} from "../../views/planning/planning-rules";

const props = defineProps<{ open: boolean; task: TaskItem | null }>();

const emit = defineEmits<{ close: []; changed: [] }>();

const store = usePlanningStore();

const candidates = ref<TaskItem[]>([]);
const dependencies = ref<TaskDependencyItem[]>([]);
const picked = ref("");
const loading = ref(false);
const submitting = ref(false);

const errorText = computed(() => store.error);

const taskTitles = computed(() => {
  const map = new Map<string, string>();

  for (const item of candidates.value) {
    map.set(item.id, item.title);
  }

  return map;
});

// 可选前置：排除自己与已存在的前置，避免自依赖与重复提交
const selectable = computed(() => {
  const existing = new Set(dependencies.value.map((item) => item.depends_on_task_id));

  return candidates.value.filter(
    (item) => item.id !== props.task?.id && !existing.has(item.id),
  );
});

watch(
  () => props.open,
  async (open) => {
    if (!open || !props.task) {
      return;
    }

    store.clearError();
    loading.value = true;
    picked.value = "";

    try {
      const [options, links] = await Promise.all([
        store.loadTaskOptions(),
        store.loadDependencies(props.task.id),
      ]);

      candidates.value = options;
      dependencies.value = links;
    } finally {
      loading.value = false;
    }
  },
);

async function addDependency(): Promise<void> {
  if (!props.task || !picked.value || submitting.value) {
    return;
  }

  submitting.value = true;

  try {
    if (await store.addDependency(props.task.id, picked.value)) {
      dependencies.value = await store.loadDependencies(props.task.id);
      picked.value = "";
      emit("changed");
    }
  } finally {
    submitting.value = false;
  }
}

async function removeDependency(dependsOnTaskId: string): Promise<void> {
  if (!props.task || submitting.value) {
    return;
  }

  submitting.value = true;

  try {
    if (await store.removeDependency(props.task.id, dependsOnTaskId)) {
      dependencies.value = dependencies.value.filter(
        (item) => item.depends_on_task_id !== dependsOnTaskId,
      );
      emit("changed");
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    description="前置任务完成后当前任务才能开始。系统会检测环路：A 依赖 B、B 又依赖 A 会被拒绝。"
    :open="open"
    :title="`前置依赖 · ${task?.title ?? ''}`"
    @close="emit('close')"
  >
    <div class="pf-form">
      <p v-if="loading" class="pf-hint">正在读取依赖…</p>

      <template v-else>
        <section class="dep-section">
          <h3 class="dep-heading">当前前置</h3>

          <p v-if="dependencies.length === 0" class="pf-hint">暂无前置依赖，可独立开始。</p>

          <ul v-else class="dep-list">
            <li v-for="link in dependencies" :key="link.depends_on_task_id" class="dep-row">
              <span class="dep-title">
                {{ taskTitles.get(link.depends_on_task_id) ?? "（已不在列表中的任务）" }}
              </span>

              <button
                :disabled="submitting"
                class="dep-remove"
                type="button"
                @click="removeDependency(link.depends_on_task_id)"
              >
                解除
              </button>
            </li>
          </ul>
        </section>

        <section class="dep-section">
          <h3 class="dep-heading">新增前置</h3>

          <div class="dep-add">
            <select v-model="picked" class="pf-input">
              <option value="">选择一项任务…</option>
              <option v-for="item in selectable" :key="item.id" :value="item.id">
                [{{ TASK_STATUS_LABELS[item.status] }}] {{ item.title }}
              </option>
            </select>

            <button
              :disabled="!picked || submitting"
              class="pf-btn pf-btn--primary"
              type="button"
              @click="addDependency()"
            >
              添加
            </button>
          </div>

          <p v-if="selectable.length === 0" class="pf-hint">
            同项目内已没有可添加的任务（或全部已是前置）。
          </p>
        </section>
      </template>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">关闭</button>
      </div>
    </div>
  </ModalDialog>
</template>

<style scoped>
.dep-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dep-heading {
  margin: 0;

  color: var(--ink-2);
  font-size: 13px;
  font-weight: 600;
}

.dep-list {
  display: flex;
  flex-direction: column;
  gap: 6px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.dep-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding: 9px 12px;

  border: 1px solid var(--line);
  border-radius: 8px;

  background: var(--surface-muted);
}

.dep-title {
  overflow: hidden;

  color: var(--ink-2);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dep-remove {
  flex-shrink: 0;

  padding: 4px 9px;

  border: 1px solid #f7dada;
  border-radius: 6px;

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 12px;
  cursor: pointer;

  transition: background-color 0.18s ease;
}

.dep-remove:hover:not(:disabled) {
  background: #fdeaea;
}

.dep-add {
  display: flex;
  gap: 8px;
}

.dep-add .pf-input {
  flex: 1;
}

.dep-add .pf-btn {
  flex-shrink: 0;
  min-height: 42px;
}
</style>
