<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {SessionDetailItem} from "../../api/sessions";
import ModalDialog from "../ui/ModalDialog.vue";
import {useSessionsStore} from "../../stores/sessions";
import {
  buildSessionCreatePayload,
  buildSessionUpdatePayload,
  emptySessionForm,
  sessionToForm,
  type SessionFormFields,
} from "../../views/sessions/session-forms";
import {ACTIVITY_TYPE_OPTIONS, formatMinutes, intervalMinutes} from "../../views/sessions/session-rules";

const props = defineProps<{
  open: boolean;
  session: SessionDetailItem | null;
  preset?: {startedAt?: string; endedAt?: string; durationMinutes?: number} | null;
}>();

const emit = defineEmits<{ close: []; saved: [id: string] }>();

const store = useSessionsStore();

const fields = ref<SessionFormFields>(emptySessionForm());
const localError = ref("");

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    fields.value = props.session
      ? sessionToForm(props.session)
      : emptySessionForm(props.preset ?? undefined);
    localError.value = "";
  },
  {immediate: true},
);

const intervalHint = computed(() => {
  const started = new Date(fields.value.startedAt).getTime();
  const ended = new Date(fields.value.endedAt).getTime();

  if (Number.isNaN(started) || Number.isNaN(ended) || ended < started) {
    return "";
  }

  const span = intervalMinutes(
    fields.value.startedAt,
    fields.value.endedAt,
  );

  if (span === null) {
    return "";
  }

  return fields.value.durationMinutes.trim() === ""
    ? `区间 ${span} 分钟，留空则按区间计为有效时长`
    : `区间 ${span} 分钟；有效时长可以小于它（中间有暂停），不能大于`;
});

async function submit(): Promise<void> {
  if (props.session) {
    const built = buildSessionUpdatePayload(props.session, fields.value);

    if (!built.ok) {
      localError.value = built.error;
      return;
    }

    if (Object.keys(built.value).length === 0) {
      emit("close");
      return;
    }

    if (await store.save(built.value)) {
      emit("saved", props.session.id);
      emit("close");
    }
    return;
  }

  const built = buildSessionCreatePayload(fields.value);

  if (!built.ok) {
    localError.value = built.error;
    return;
  }

  if (await store.create(built.value)) {
    emit("saved", store.selected?.id ?? "");
    emit("close");
  }
}
</script>

<template>
  <ModalDialog
    :description="
      session
        ? '只提交真正改过的字段；关联内容与任务分摊在详情区单独维护。'
        : '记录已经发生的一段学习。项目、内容、任务可以先建后补，不影响这次记录成立。'
    "
    :open="open"
    :title="session ? '编辑学习活动' : '记录一次学习'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="submit">
      <div class="pf-group">
        <label class="pf-label" for="session-title">标题<span class="pf-optional">可选</span></label>
        <input
          id="session-title"
          v-model="fields.title"
          class="pf-input"
          maxlength="200"
          placeholder="例如：高数第一轮 · 极限的定义"
          type="text"
        />
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="session-type">活动类型</label>
          <select id="session-type" v-model="fields.activityType" class="pf-input">
            <option v-for="option in ACTIVITY_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="session-project">主要项目<span class="pf-optional">可选</span></label>
          <select id="session-project" v-model="fields.primaryProjectId" class="pf-input">
            <option value="">不归属项目</option>
            <option v-for="project in store.projectOptions" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </select>
        </div>
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="session-start">开始时间<span class="pf-required">必填</span></label>
          <input id="session-start" v-model="fields.startedAt" class="pf-input" type="datetime-local"/>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="session-end">结束时间<span class="pf-required">必填</span></label>
          <input id="session-end" v-model="fields.endedAt" class="pf-input" type="datetime-local"/>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="session-duration">
          有效时长（分钟）<span class="pf-optional">留空按区间</span>
        </label>
        <input
          id="session-duration"
          v-model="fields.durationMinutes"
          class="pf-input"
          inputmode="numeric"
          min="0"
          placeholder="例如 90"
          type="number"
        />
        <p v-if="intervalHint" class="pf-hint">{{ intervalHint }}</p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="session-summary">概述<span class="pf-optional">可选</span></label>
        <textarea
          id="session-summary"
          v-model="fields.summary"
          class="pf-input form-textarea"
          placeholder="这次学了什么，两三句就够"
          rows="2"
        />
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="session-what">做了什么<span class="pf-optional">可选</span></label>
          <textarea id="session-what" v-model="fields.whatDone" class="pf-input form-textarea" rows="2"/>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="session-problems">遇到的问题<span class="pf-optional">可选</span></label>
          <textarea id="session-problems" v-model="fields.problems" class="pf-input form-textarea" rows="2"/>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="session-next">下一步<span class="pf-optional">可选</span></label>
        <textarea id="session-next" v-model="fields.nextSteps" class="pf-input form-textarea" rows="2"/>
        <p class="pf-hint">
          四段式描述全部可选：补录时常常只记得时长，强制填写只会让人干脆不记（D-31）。
        </p>
      </div>

      <p v-if="localError" class="pf-error">{{ localError }}</p>
      <p v-else-if="store.selected && session" class="form-note">
        当前有效时长 {{ formatMinutes(session.duration_minutes) }}。
      </p>

      <div class="pf-actions">
        <button class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">取消</button>
        <button class="pf-btn pf-btn--primary" type="submit">
          {{ session ? "保存修改" : "记录" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>

<style scoped>
.form-textarea {
  min-height: 60px;
  padding: 9px 11px;
  resize: vertical;
  font-family: inherit;
  line-height: 1.6;
}

.form-note {
  margin: 0;
  color: var(--ink-4);
  font-size: 11.5px;
}
</style>
