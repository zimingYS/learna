<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {MilestoneItem, StageItem, TaskItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";
import {
  buildTaskCreatePayload,
  buildTaskUpdatePayload,
  emptyTaskForm,
  type TaskFormFields,
  taskToForm,
} from "../../views/planning/planning-forms";
import {PRIORITY_OPTIONS} from "../../views/planning/planning-rules";

const props = defineProps<{
  open: boolean;
  task: TaskItem | null;
  stages: StageItem[];
  milestones: MilestoneItem[];
}>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

const fields = ref<TaskFormFields>(emptyTaskForm());
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.task !== null);
const errorText = computed(() => formError.value || store.error);

// 选了里程碑就必须与它所属阶段一致（D-05），否则后端 422
const derivedStageId = computed(
  () => props.milestones.find((item) => item.id === fields.value.milestoneId)?.stage_id ?? null,
);

const stageLocked = computed(() => derivedStageId.value !== null);

// 只展示与所选阶段匹配的里程碑，避免用户选出非法组合
const milestoneChoices = computed(() => {
  if (!fields.value.stageId) {
    return props.milestones;
  }

  return props.milestones.filter((item) => item.stage_id === fields.value.stageId);
});

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    fields.value = props.task ? taskToForm(props.task) : emptyTaskForm();
    formError.value = "";
    store.clearError();
  },
);

watch(derivedStageId, (stageId) => {
  if (stageId && stageId !== fields.value.stageId) {
    fields.value.stageId = stageId;
  }
});

// 换阶段后原先选的里程碑可能不再匹配，直接清掉
watch(
  () => fields.value.stageId,
  (stageId) => {
    if (!fields.value.milestoneId) {
      return;
    }

    const milestone = props.milestones.find((item) => item.id === fields.value.milestoneId);

    if (milestone && milestone.stage_id !== stageId) {
      fields.value.milestoneId = "";
    }
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

    if (props.task) {
      const built = buildTaskUpdatePayload(props.task, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.editTask(props.task.id, built.payload)) !== null;
    } else {
      const built = buildTaskCreatePayload(fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.addTask(built.payload)) !== null;
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
        ? '修改任务内容与归属；状态请在任务行上直接切换。'
        : '任务只挂项目也可以；挂到阶段或里程碑后会自动参与该层的统计。'
    "
    :open="open"
    :title="isEdit ? '编辑任务' : '新建任务'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="task-title">
          任务标题
          <span class="pf-required">*</span>
        </label>

        <input
          id="task-title"
          v-model="fields.title"
          class="pf-input"
          maxlength="200"
          placeholder="例如：做完 2019 年数二真题第 1 套"
          required
          type="text"
        />
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="task-stage">
            所属阶段
            <span class="pf-optional">选填</span>
          </label>

          <select
            id="task-stage"
            v-model="fields.stageId"
            :disabled="stageLocked"
            class="pf-input"
          >
            <option value="">（只挂项目）</option>
            <option v-for="stage in stages" :key="stage.id" :value="stage.id">
              #{{ stage.sequence_no }} {{ stage.name }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="task-milestone">
            所属里程碑
            <span class="pf-optional">选填</span>
          </label>

          <select id="task-milestone" v-model="fields.milestoneId" class="pf-input">
            <option value="">（不挂里程碑）</option>
            <option v-for="item in milestoneChoices" :key="item.id" :value="item.id">
              {{ item.title }}
            </option>
          </select>
        </div>
      </div>

      <p v-if="stageLocked" class="pf-hint">已选里程碑，阶段自动锁定为里程碑所属阶段。</p>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="task-priority">优先级</label>

          <select id="task-priority" v-model="fields.priority" class="pf-input">
            <option v-for="option in PRIORITY_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="task-due">
            截止时间
            <span class="pf-optional">选填</span>
          </label>

          <input id="task-due" v-model="fields.dueAt" class="pf-input" type="datetime-local"/>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="task-minutes">
          预计时长（分钟）
          <span class="pf-optional">选填</span>
        </label>

        <input
          id="task-minutes"
          v-model="fields.estimatedMinutes"
          class="pf-input"
          min="0"
          placeholder="45"
          type="number"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label" for="task-description">
          描述
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="task-description"
          v-model="fields.description"
          class="pf-input"
          maxlength="2000"
          placeholder="完成标准、参考资料……"
        ></textarea>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建任务" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
