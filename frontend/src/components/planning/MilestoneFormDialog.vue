<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {MilestoneItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";
import {
  buildMilestoneCreatePayload,
  buildMilestoneUpdatePayload,
  emptyMilestoneForm,
  type MilestoneFormFields,
  milestoneToForm,
} from "../../views/planning/planning-forms";

const props = defineProps<{
  open: boolean;
  milestone: MilestoneItem | null;
  /** 新建时归属的阶段 */
  stageId: string;
}>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

const fields = ref<MilestoneFormFields>(emptyMilestoneForm());
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.milestone !== null);
const errorText = computed(() => formError.value || store.error);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    fields.value = props.milestone ? milestoneToForm(props.milestone) : emptyMilestoneForm();
    formError.value = "";
    store.clearError();
  },
);

async function handleSubmit(): Promise<void> {
  if (submitting.value) {
    return;
  }

  formError.value = "";
  submitting.value = true;

  try {
    let saved = false;

    if (props.milestone) {
      const built = buildMilestoneUpdatePayload(props.milestone, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.editMilestone(props.milestone.id, built.payload)) !== null;
    } else {
      const built = buildMilestoneCreatePayload(props.stageId, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.addMilestone(built.payload)) !== null;
    }

    if (saved) {
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
    :description="
      isEdit ? '更新里程碑标题与说明；状态由阶段内的里程碑列表切换。' : '里程碑挂在阶段下，用来标记阶段内的关键节点。'
    "
    :open="open"
    :title="isEdit ? '编辑里程碑' : '新建里程碑'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="milestone-title">
          里程碑标题
          <span class="pf-required">*</span>
        </label>

        <input
          id="milestone-title"
          v-model="fields.title"
          class="pf-input"
          maxlength="200"
          placeholder="例如：完成极限与连续全部习题"
          required
          type="text"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label" for="milestone-description">
          描述
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="milestone-description"
          v-model="fields.description"
          class="pf-input"
          maxlength="2000"
          placeholder="验收标准、参考材料……"
        ></textarea>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建里程碑" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
