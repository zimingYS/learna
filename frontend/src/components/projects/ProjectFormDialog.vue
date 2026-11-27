<script setup lang="ts">
import {computed, ref, watch} from "vue";
import ModalDialog from "../ui/ModalDialog.vue";
import type {ProjectItem} from "../../api/projects";
import {useProjectsStore} from "../../stores/projects";
import {
  buildCreatePayload,
  buildUpdatePayload,
  DEADLINE_TYPE_OPTIONS,
  emptyProjectForm,
  IMPORTANCE_OPTIONS,
  isDeadlineEditable,
  PROJECT_TYPE_OPTIONS,
  type ProjectFormFields,
  projectToForm,
} from "../../views/projects/project-rules";

const props = defineProps<{
  open: boolean;
  project: ProjectItem | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [project: ProjectItem];
}>();

const store = useProjectsStore();

const fields = ref<ProjectFormFields>(emptyProjectForm());
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.project !== null);
// fixed 项目的截止日期只能走 extend-deadline（D-04），表单里置灰
const deadlineLocked = computed(() => props.project !== null && !isDeadlineEditable(props.project));
const errorText = computed(() => formError.value || store.error);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    fields.value = props.project ? projectToForm(props.project) : emptyProjectForm();
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
    if (props.project) {
      const built = buildUpdatePayload(props.project, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      const updated = await store.update(props.project.id, built.payload);

      if (!updated) {
        return;
      }

      emit("saved", updated);
    } else {
      const built = buildCreatePayload(fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      const created = await store.create(built.payload);

      if (!created) {
        return;
      }

      emit("saved", created);
    }

    emit("close");
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    :description="isEdit ? '更新项目的基础信息与约束。' : '新建项目先进入草稿，确认后再开始推进。'"
    :open="open"
    :title="isEdit ? '编辑项目' : '新建项目'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="project-name">
          项目名称
          <span class="pf-required">*</span>
        </label>

        <input
          id="project-name"
          v-model="fields.name"
          class="pf-input"
          maxlength="160"
          placeholder="例如：2027 考研数学强化"
          required
          type="text"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label" for="project-description">
          项目描述
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="project-description"
          v-model="fields.description"
          class="pf-input"
          maxlength="2000"
          placeholder="这个项目要达成什么、为什么重要……"
        ></textarea>
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="project-type">项目类型</label>

          <select id="project-type" v-model="fields.projectType" class="pf-input">
            <option v-for="option in PROJECT_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="project-importance">重要程度</label>

          <select id="project-importance" v-model="fields.importanceLevel" class="pf-input">
            <option v-for="option in IMPORTANCE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="project-deadline-type">截止类型</label>

          <select
            id="project-deadline-type"
            v-model="fields.deadlineType"
            :disabled="deadlineLocked"
            class="pf-input"
          >
            <option v-for="option in DEADLINE_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="project-deadline-at">
            截止时间
            <span v-if="fields.deadlineType !== 'none'" class="pf-required">*</span>
          </label>

          <input
            id="project-deadline-at"
            v-model="fields.deadlineAt"
            :disabled="deadlineLocked || fields.deadlineType === 'none'"
            class="pf-input"
            type="datetime-local"
          />
        </div>
      </div>

      <p v-if="deadlineLocked" class="pf-hint">
        固定截止日期属外部约束，不能在此修改；保存后请用「修改截止日期」并填写原因。
      </p>

      <div class="pf-group">
        <label class="pf-label" for="project-override">
          重要程度覆盖至
          <span class="pf-optional">选填</span>
        </label>

        <input
          id="project-override"
          v-model="fields.overrideUntil"
          class="pf-input"
          type="datetime-local"
        />

        <p class="pf-hint">到期后回落到上方的基础重要程度。</p>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建项目" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
