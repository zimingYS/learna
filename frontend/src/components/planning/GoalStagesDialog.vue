<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {Contribution, GoalItem, GoalStageLink, StageItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";

const props = defineProps<{
  open: boolean;
  goal: GoalItem | null;
  /** 当前路线下的全部阶段（跨路线的阶段不可选：目标关联须同项目同路线语义） */
  stages: StageItem[];
}>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

const selected = ref<Map<string, Contribution>>(new Map());
const loading = ref(false);
const submitting = ref(false);

const errorText = computed(() => store.error);

watch(
  () => props.open,
  async (open) => {
    if (!open || !props.goal) {
      return;
    }

    store.clearError();
    loading.value = true;

    try {
      const links = await store.loadGoalStages(props.goal.id);
      selected.value = new Map(links.map((link) => [link.stage_id, link.contribution]));
    } finally {
      loading.value = false;
    }
  },
);

function isChecked(stageId: string): boolean {
  return selected.value.has(stageId);
}

function toggle(stageId: string): void {
  const next = new Map(selected.value);

  if (next.has(stageId)) {
    next.delete(stageId);
  } else {
    next.set(stageId, "supporting");
  }

  selected.value = next;
}

function setContribution(stageId: string, contribution: Contribution): void {
  const next = new Map(selected.value);

  next.set(stageId, contribution);
  selected.value = next;
}

async function handleSubmit(): Promise<void> {
  if (!props.goal || submitting.value) {
    return;
  }

  submitting.value = true;

  try {
    const links: GoalStageLink[] = [...selected.value.entries()].map(([stageId, contribution]) => ({
      stage_id: stageId,
      contribution,
    }));

    if (await store.saveGoalStages(props.goal.id, links)) {
      emit("saved");
      emit("close");
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    description="勾选该目标贡献的阶段，primary 为主贡献、supporting 为辅助贡献。整体替换，取消勾选即解除关联。"
    :open="open"
    title="关联阶段"
    @close="emit('close')"
  >
    <div class="pf-form">
      <p v-if="loading" class="pf-hint">正在读取关联…</p>

      <p v-else-if="stages.length === 0" class="pf-hint">
        当前路线还没有阶段。请先在「路线」里创建阶段，再回来关联目标。
      </p>

      <ul v-else class="link-list">
        <li v-for="stage in stages" :key="stage.id" class="link-row">
          <label class="link-main">
            <input :checked="isChecked(stage.id)" type="checkbox" @change="toggle(stage.id)"/>
            <span class="link-seq">#{{ stage.sequence_no }}</span>
            <span class="link-name">{{ stage.name }}</span>
          </label>

          <select
            v-if="isChecked(stage.id)"
            :value="selected.get(stage.id)"
            class="link-select"
            @change="setContribution(stage.id, ($event.target as HTMLSelectElement).value as Contribution)"
          >
            <option value="primary">主贡献</option>
            <option value="supporting">辅助</option>
          </select>
        </li>
      </ul>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button
          :disabled="submitting || loading"
          class="pf-btn pf-btn--primary"
          type="button"
          @click="handleSubmit()"
        >
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : "保存关联" }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<style scoped>
.link-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  max-height: 340px;
  padding: 0;
  margin: 0;
  overflow-y: auto;

  list-style: none;
}

.link-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  padding: 10px 12px;

  border: 1px solid var(--line);
  border-radius: 9px;

  background: var(--surface-muted);
}

.link-main {
  display: flex;
  flex: 1;
  align-items: center;
  gap: 9px;

  color: var(--ink-2);
  font-size: 14px;
  cursor: pointer;
}

.link-main input {
  width: 15px;
  height: 15px;
  accent-color: var(--brand);
  cursor: pointer;
}

.link-seq {
  color: var(--ink-3);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.link-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-select {
  height: 32px;
  padding: 0 8px;

  border: 1px solid var(--line-strong);
  border-radius: 7px;

  background: #fff;
  color: var(--ink-2);

  font-family: inherit;
  font-size: 12px;
}
</style>
