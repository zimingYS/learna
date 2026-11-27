<script setup lang="ts">
import {computed, ref, watch} from "vue";
import ModalDialog from "../ui/ModalDialog.vue";
import type {ProjectItem} from "../../api/projects";
import {useProjectsStore} from "../../stores/projects";
import {formatDateTime, toLocalInput} from "../../views/projects/project-rules";

const props = defineProps<{
  open: boolean;
  project: ProjectItem | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [project: ProjectItem];
}>();

const store = useProjectsStore();

const newDeadlineAt = ref("");
const reason = ref("");
const formError = ref("");
const submitting = ref(false);

const errorText = computed(() => formError.value || store.error);
const currentDeadline = computed(() => formatDateTime(props.project?.deadline_at ?? null));

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    newDeadlineAt.value = toLocalInput(props.project?.deadline_at ?? null);
    reason.value = "";
    formError.value = "";
    store.clearError();
  },
);

async function handleSubmit(): Promise<void> {
  if (submitting.value || !props.project) {
    return;
  }

  const trimmedReason = reason.value.trim();

  if (!newDeadlineAt.value) {
    formError.value = "请选择新的截止日期时间";
    return;
  }

  if (!trimmedReason) {
    formError.value = "请填写改期原因，便于日后回溯";
    return;
  }

  const next = new Date(newDeadlineAt.value);

  if (Number.isNaN(next.getTime())) {
    formError.value = "新的截止日期时间格式不正确";
    return;
  }

  const current = props.project.deadline_at ? new Date(props.project.deadline_at) : null;

  if (current && next.getTime() <= current.getTime()) {
    formError.value = "新截止日期必须晚于当前截止日期";
    return;
  }

  formError.value = "";
  submitting.value = true;

  try {
    const updated = await store.extendDeadline(props.project.id, {
      new_deadline_at: next.toISOString(),
      reason: trimmedReason,
    });

    if (!updated) {
      return;
    }

    emit("saved", updated);
    emit("close");
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    description="固定截止日期代表外部约束，改期需要留下原因。"
    :open="open"
    title="修改截止日期"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <p class="deadline-current">当前截止日期：<strong>{{ currentDeadline }}</strong></p>

      <div class="pf-group">
        <label class="pf-label" for="deadline-new">
          新的截止日期
          <span class="pf-required">*</span>
        </label>

        <input id="deadline-new" v-model="newDeadlineAt" class="pf-input" type="datetime-local"/>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="deadline-reason">
          改期原因
          <span class="pf-required">*</span>
        </label>

        <textarea
          id="deadline-reason"
          v-model="reason"
          class="pf-input"
          maxlength="500"
          placeholder="例如：考试时间调整至 12 月 26 日"
        ></textarea>

        <p class="pf-hint">原因会随改期记录一起留痕，供日后复盘。</p>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "提交中..." : "确认改期" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>

<style scoped>
.deadline-current {
  margin: 0;
  padding: 10px 12px;

  border-radius: 8px;
  background: #f5f6fb;

  color: #5b6577;
  font-size: 13px;
}

.deadline-current strong {
  color: #263248;
}
</style>
