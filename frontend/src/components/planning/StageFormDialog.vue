<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {StageItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";
import {
  buildStageCreatePayload,
  buildStageUpdatePayload,
  emptyStageForm,
  type StageFormFields,
  stageToForm,
} from "../../views/planning/planning-forms";

const props = defineProps<{ open: boolean; stage: StageItem | null }>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

const fields = ref<StageFormFields>(emptyStageForm());
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.stage !== null);
const errorText = computed(() => formError.value || store.error);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    fields.value = props.stage ? stageToForm(props.stage) : emptyStageForm();
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

    if (props.stage) {
      const built = buildStageUpdatePayload(props.stage, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.editStage(props.stage.id, built.payload)) !== null;
    } else {
      const built = buildStageCreatePayload(fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.addStage(built.payload)) !== null;
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
      isEdit
        ? '调整阶段名称、说明与顺序；状态请用阶段卡片上的动作按钮推进。'
        : '阶段是路线的结构单元，创建后初始为未开始。'
    "
    :open="open"
    :title="isEdit ? '编辑阶段' : '新建阶段'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="stage-name">
          阶段名称
          <span class="pf-required">*</span>
        </label>

        <input
          id="stage-name"
          v-model="fields.name"
          class="pf-input"
          maxlength="160"
          placeholder="例如：高数基础一轮"
          required
          type="text"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label" for="stage-sequence">
          顺序号
          <span class="pf-optional">留空追加到末尾</span>
        </label>

        <input
          id="stage-sequence"
          v-model="fields.sequenceNo"
          class="pf-input"
          min="1"
          placeholder="1"
          type="number"
        />

        <p class="pf-hint">同一路线内顺序号唯一，冲突时请换一个数字。</p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="stage-description">
          描述
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="stage-description"
          v-model="fields.description"
          class="pf-input"
          maxlength="2000"
          placeholder="这个阶段要完成什么、以什么为完成标准……"
        ></textarea>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建阶段" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
