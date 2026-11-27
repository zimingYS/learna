<script setup lang="ts">
import {computed, ref, watch} from "vue";
import ModalDialog from "../ui/ModalDialog.vue";
import type {ProjectItem} from "../../api/projects";
import {useProjectsStore} from "../../stores/projects";

const props = defineProps<{
  open: boolean;
  project: ProjectItem | null;
}>();

const emit = defineEmits<{
  close: [];
  purged: [];
}>();

const store = useProjectsStore();

const confirmName = ref("");
const formError = ref("");
const submitting = ref(false);

const errorText = computed(() => formError.value || store.error);
const nameMatches = computed(
  () => props.project !== null && confirmName.value.trim() === props.project.name,
);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    confirmName.value = "";
    formError.value = "";
    store.clearError();
  },
);

async function handleSubmit(): Promise<void> {
  if (submitting.value || !props.project) {
    return;
  }

  if (!nameMatches.value) {
    formError.value = "项目名称不匹配，请重新输入";
    return;
  }

  formError.value = "";
  submitting.value = true;

  try {
    const ok = await store.purge(props.project.id, confirmName.value.trim());

    if (!ok) {
      return;
    }

    emit("purged");
    emit("close");
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    description="彻底删除不可恢复，请确认后操作。"
    :open="open"
    title="彻底删除项目"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="purge-warning">
        <p>此操作将永久删除：</p>
        <ul>
          <li>项目本身及其全部规划数据（目标、路线、阶段、里程碑、任务与依赖）</li>
          <li>关联的学习记录不会被删除，但会解除与该项目的主要归属关系</li>
        </ul>
        <p>如果只是想让项目退出活跃列表，请改用「归档」。</p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="purge-confirm">
          输入项目名称
          <code class="purge-name">{{ project?.name }}</code>
        </label>

        <input
          id="purge-confirm"
          v-model="confirmName"
          autocomplete="off"
          class="pf-input"
          maxlength="160"
          placeholder="与上方名称完全一致"
          type="text"
        />
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button
          :disabled="submitting || !nameMatches"
          class="pf-btn pf-btn--danger"
          type="submit"
        >
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "删除中..." : "彻底删除" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>

<style scoped>
.purge-warning {
  padding: 14px 16px;

  border: 1px solid #f6d5d5;
  border-radius: 10px;

  background: #fff8f8;
  color: #8d4040;

  font-size: 13px;
  line-height: 1.7;
}

.purge-warning p {
  margin: 0;
}

.purge-warning ul {
  padding-left: 18px;
  margin: 6px 0;
}

.purge-name {
  padding: 2px 6px;
  margin-left: 6px;

  border-radius: 4px;
  background: #f0f2f8;

  color: #364152;
  font-family: inherit;
  font-size: 12px;
}
</style>
