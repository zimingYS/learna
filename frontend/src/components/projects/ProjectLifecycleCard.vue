<script setup lang="ts">
import {computed} from "vue";
import type {ProjectAction, ProjectItem} from "../../api/projects";
import {
  availableActions,
  canDeleteDraft,
  PROJECT_ACTION_LABELS,
} from "../../views/projects/project-rules";

const props = defineProps<{
  project: ProjectItem;
  pendingAction: ProjectAction | null;
}>();

const emit = defineEmits<{
  action: [action: ProjectAction];
  deleteDraft: [];
}>();

// 推进类动作给主色，暂停/归档留中性，避免一组按钮同色
const ACTION_TONES: Record<ProjectAction, "primary" | "ghost"> = {
  activate: "primary",
  pause: "ghost",
  resume: "primary",
  complete: "primary",
  reopen: "primary",
  archive: "ghost",
};

const actions = computed(() => availableActions(props.project.status));
</script>

<template>
  <section class="card" aria-labelledby="lifecycle-title">
    <h2 id="lifecycle-title">生命周期</h2>
    <p class="card-hint">状态变更通过动作完成，归档或完成后历史数据仍会保留。</p>

    <div v-if="actions.length > 0" class="action-list">
      <button
        v-for="action in actions"
        :key="action"
        :class="['action-btn', `action-btn--${ACTION_TONES[action]}`]"
        :disabled="pendingAction !== null"
        type="button"
        @click="emit('action', action)"
      >
        {{ pendingAction === action ? "处理中…" : PROJECT_ACTION_LABELS[action] }}
      </button>
    </div>

    <p v-else class="card-hint">当前状态没有可执行的动作。</p>

    <button v-if="canDeleteDraft(project.status)" class="danger-link" type="button" @click="emit('deleteDraft')">
      删除草稿项目
    </button>
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

.card-hint {
  margin: 0 0 14px;
  color: #8490a3;
  font-size: 12.5px;
  line-height: 1.7;
}

.action-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.action-btn {
  min-height: 40px;
  padding: 0 16px;

  border-radius: 9px;

  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease, border-color 0.2s ease;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.action-btn--primary {
  border: 1px solid transparent;
  background: #4f46e5;
  color: #fff;
}

.action-btn--primary:hover:not(:disabled) {
  background: #4338ca;
}

.action-btn--ghost {
  border: 1px solid #dce2eb;
  background: #fff;
  color: #596579;
}

.action-btn--ghost:hover:not(:disabled) {
  background: #f8f9fc;
}

.action-btn:focus-visible,
.danger-link:focus-visible {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}

.danger-link {
  padding: 8px 0 0;
  margin-top: 14px;

  border: none;
  border-top: 1px solid #f2f4f8;
  background: transparent;
  color: #cf5f5f;

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.danger-link:hover {
  color: #b93c3c;
}

@media (max-width: 600px) {
  .card {
    padding: 18px;
  }
}
</style>
