<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {GoalItem} from "../../api/planning";
import ModalDialog from "../ui/ModalDialog.vue";
import {usePlanningStore} from "../../stores/planning";
import {
  buildGoalCreatePayload,
  buildGoalUpdatePayload,
  emptyGoalForm,
  type GoalFormFields,
  goalToForm,
} from "../../views/planning/planning-forms";
import {GOAL_TYPE_OPTIONS} from "../../views/planning/planning-rules";

const props = defineProps<{
  open: boolean;
  goal: GoalItem | null;
  /** 可作为上级的目标：主目标列表（两层上限，D-06） */
  parentOptions: GoalItem[];
}>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = usePlanningStore();

const fields = ref<GoalFormFields>(emptyGoalForm());
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.goal !== null);
const errorText = computed(() => formError.value || store.error);

// 编辑时不能把目标挂到自己或自己的子目标下
const parentChoices = computed(() =>
  props.parentOptions.filter(
    (option) => option.id !== props.goal?.id && option.parent_goal_id === null,
  ),
);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    if (props.goal) {
      fields.value = goalToForm(props.goal);
    } else {
      const form = emptyGoalForm();
      // 项目已有主目标时，新目标默认挂到它下面：主目标唯一，直接建主目标会被后端 409
      form.parentGoalId = props.parentOptions[0]?.id ?? "";
      fields.value = form;
    }

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

    if (props.goal) {
      const built = buildGoalUpdatePayload(props.goal, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.editGoal(props.goal.id, built.payload)) !== null;
    } else {
      const built = buildGoalCreatePayload(fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      saved = (await store.addGoal(built.payload)) !== null;
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
    :description="isEdit ? '更新目标的定义、层级与时间。' : '目标描述要达成的结果，可挂到某个主目标下。'"
    :open="open"
    :title="isEdit ? '编辑目标' : '新建目标'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="goal-title">
          目标标题
          <span class="pf-required">*</span>
        </label>

        <input
          id="goal-title"
          v-model="fields.title"
          class="pf-input"
          maxlength="200"
          placeholder="例如：数学二真题稳定 110 分以上"
          required
          type="text"
        />
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="goal-type">目标类型</label>

          <select id="goal-type" v-model="fields.goalType" class="pf-input">
            <option v-for="option in GOAL_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="goal-target">
            目标时间
            <span class="pf-optional">选填</span>
          </label>

          <input id="goal-target" v-model="fields.targetAt" class="pf-input" type="datetime-local"/>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="goal-parent">
          上级目标
          <span class="pf-optional">留空即主目标</span>
        </label>

        <select id="goal-parent" v-model="fields.parentGoalId" class="pf-input">
          <option value="">（作为主目标）</option>
          <option v-for="option in parentChoices" :key="option.id" :value="option.id">
            {{ option.title }}
          </option>
        </select>

        <p class="pf-hint">
          一个项目最多一个主目标，子目标只能挂在主目标下。已有主目标时新建默认作为它的子目标；若要建新的主目标，请先把原主目标改为子目标。
        </p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="goal-description">
          描述
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="goal-description"
          v-model="fields.description"
          class="pf-input"
          maxlength="2000"
          placeholder="达成标准、衡量方式……"
        ></textarea>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建目标" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>
