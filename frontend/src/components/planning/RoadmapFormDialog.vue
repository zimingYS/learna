<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {RoadmapItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";

const props = defineProps<{ open: boolean; roadmap: RoadmapItem | null }>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

type Source = "user" | "template";

const summary = ref("");
const source = ref<Source>("user");
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.roadmap !== null);
const errorText = computed(() => formError.value || store.error);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    summary.value = props.roadmap?.change_summary ?? "";
    source.value = "user";
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
    const trimmed = summary.value.trim();
    let saved = false;

    if (props.roadmap) {
      saved = (await store.editRoadmap(props.roadmap.id, trimmed ? trimmed : null)) !== null;
    } else {
      saved =
        (await store.addRoadmap({
          source: source.value,
          ...(trimmed ? {change_summary: trimmed} : {}),
        })) !== null;
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
        ? '版本号与状态由服务端维护，这里只改变更说明。'
        : '新版本以草稿创建，编辑好阶段后再确认生效。'
    "
    :open="open"
    :title="isEdit ? '编辑路线说明' : '新建路线版本'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div v-if="!isEdit" class="pf-group">
        <label class="pf-label" for="roadmap-source">来源</label>

        <select id="roadmap-source" v-model="source" class="pf-input">
          <option value="user">手动规划</option>
          <option value="template">模板</option>
        </select>

        <p class="pf-hint">AI 生成的路线须先经提案确认，不在本页直接创建。</p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="roadmap-summary">
          变更说明
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="roadmap-summary"
          v-model="summary"
          class="pf-input"
          maxlength="500"
          placeholder="本版相对上一版调整了什么，例如：把基础阶段拆成两段"
        ></textarea>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建草稿" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
