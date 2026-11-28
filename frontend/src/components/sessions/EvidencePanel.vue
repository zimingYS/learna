<script setup lang="ts">
import {ref, watch} from "vue";
import {formatDateTime} from "../../lib/datetime";
import StatusPill from "../ui/StatusPill.vue";
import {useSessionsStore} from "../../stores/sessions";
import {buildEvidencePayload, emptyEvidenceForm, type EvidenceFormFields} from "../../views/sessions/session-forms";
import {EVIDENCE_TYPE_LABELS, EVIDENCE_TYPE_OPTIONS} from "../../views/sessions/session-rules";

const store = useSessionsStore();

const formOpen = ref(false);
const fields = ref<EvidenceFormFields>(emptyEvidenceForm());
const localError = ref("");

watch(
  () => store.selected,
  () => {
    formOpen.value = false;
    localError.value = "";
  },
);

function openForm(): void {
  fields.value = emptyEvidenceForm(store.selected?.id ?? "");
  localError.value = "";
  formOpen.value = true;
}

async function submit(): Promise<void> {
  const built = buildEvidencePayload(fields.value);

  if (!built.ok) {
    localError.value = built.error;
    return;
  }

  const ok = await store.addEvidence(built.value);

  if (ok) {
    formOpen.value = false;
    localError.value = "";
  }
}

async function remove(id: string): Promise<void> {
  await store.removeEvidence(id);
}
</script>

<template>
  <section class="evidence-panel">
    <header class="panel-head">
      <div>
        <h3>学习证据</h3>
        <p>
          {{ store.evidenceScoped ? "本次活动的证据" : "全部证据（含未关联活动的独立证据）" }}
          · 共 {{ store.evidenceTotal }} 条
        </p>
      </div>

      <button class="pf-btn pf-btn--ghost" type="button" @click="formOpen ? (formOpen = false) : openForm()">
        {{ formOpen ? "取消" : "新增证据" }}
      </button>
    </header>

    <form v-if="formOpen" class="pf-form evidence-form" @submit.prevent="submit">
      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="evidence-type">类型</label>
          <select id="evidence-type" v-model="fields.type" class="pf-input">
            <option v-for="option in EVIDENCE_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="evidence-title">标题<span class="pf-optional">可选</span></label>
          <input id="evidence-title" v-model="fields.title" class="pf-input" maxlength="200" type="text"/>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="evidence-description">
          描述
          <span v-if="fields.type !== 'link'" class="pf-required">必填</span>
        </label>
        <textarea
          id="evidence-description"
          v-model="fields.description"
          class="pf-input evidence-textarea"
          placeholder="记下当时的结论、错因或结果"
          rows="3"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label" for="evidence-url">
          外部链接
          <span v-if="fields.type === 'link'" class="pf-required">必填</span>
        </label>
        <input
          id="evidence-url"
          v-model="fields.externalUrl"
          class="pf-input"
          placeholder="https://…"
          type="url"
        />
        <p class="pf-hint">仅接受 http(s) 地址；图片与附件类证据 v1 不开放（ADR-010 §4）。</p>
      </div>

      <label class="attach-row">
        <input v-model="fields.sessionId" :disabled="!store.selected" type="checkbox" :true-value="store.selected?.id ?? ''" false-value=""/>
        <span>关联到当前活动{{ store.selected ? "" : "（未选中活动，将保存为独立证据）" }}</span>
      </label>

      <p v-if="localError" class="pf-error">{{ localError }}</p>

      <div class="pf-actions">
        <button class="pf-btn pf-btn--primary" type="submit">保存证据</button>
      </div>
    </form>

    <p v-if="store.evidence.length === 0" class="panel-state">
      还没有证据。留一条笔记或链接，之后的评价与复盘才有依据。
    </p>

    <ul v-else class="evidence-list">
      <li v-for="item in store.evidence" :key="item.id" class="evidence-item">
        <div class="evidence-main">
          <div class="evidence-head">
            <StatusPill :label="EVIDENCE_TYPE_LABELS[item.evidence_type]" tone="neutral"/>
            <span class="evidence-title">{{ item.title || EVIDENCE_TYPE_LABELS[item.evidence_type] }}</span>
            <span class="evidence-time">{{ formatDateTime(item.created_at) }}</span>
          </div>

          <p v-if="item.description" class="evidence-body">{{ item.description }}</p>

          <a
            v-if="item.external_url"
            :href="item.external_url"
            class="evidence-link"
            rel="noopener noreferrer"
            target="_blank"
          >
            {{ item.external_url }}
          </a>
        </div>

        <button class="evidence-remove" type="button" @click="remove(item.id)">删除</button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.evidence-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 18px 20px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.panel-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.panel-head h3 {
  margin: 0;
  color: var(--ink-1);
  font-size: 14px;
  font-weight: 660;
}

.panel-head p {
  margin: 5px 0 0;
  color: var(--ink-4);
  font-size: 11.5px;
}

.panel-head .pf-btn {
  min-height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.evidence-form {
  padding: 14px;
  border-radius: var(--radius-md);
  background: var(--surface-muted);
}

.evidence-textarea {
  min-height: 76px;
  padding: 9px 11px;
  resize: vertical;
  font-family: inherit;
  line-height: 1.6;
}

.attach-row {
  display: flex;
  align-items: center;
  gap: 8px;

  color: var(--ink-3);
  font-size: 12px;
  cursor: pointer;
}

.attach-row input {
  accent-color: var(--brand);
}

.panel-state {
  padding: 26px 12px;
  margin: 0;

  color: var(--ink-4);
  font-size: 12.5px;
  line-height: 1.7;
  text-align: center;
}

.evidence-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.evidence-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;

  padding: 11px 13px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
}

.evidence-main {
  min-width: 0;
}

.evidence-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.evidence-title {
  color: var(--ink-1);
  font-size: 12.8px;
  font-weight: 620;
}

.evidence-time {
  color: var(--ink-4);
  font-size: 11px;
}

.evidence-body {
  margin: 7px 0 0;

  color: var(--ink-2);
  font-size: 12.5px;
  line-height: 1.7;
  white-space: pre-wrap;
}

.evidence-link {
  display: inline-block;

  margin-top: 6px;

  color: var(--brand);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.evidence-remove {
  flex-shrink: 0;

  padding: 0 4px;

  border: none;
  background: transparent;
  color: var(--ink-4);

  font-family: inherit;
  font-size: 11.5px;
  text-decoration: underline;
  cursor: pointer;
}

.evidence-remove:hover {
  color: #bf3a40;
}
</style>
