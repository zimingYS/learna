<script setup lang="ts">
import {onMounted, ref} from "vue";
import type {SessionDetailItem} from "../api/sessions";
import EvidencePanel from "../components/sessions/EvidencePanel.vue";
import SessionContentEditor from "../components/sessions/SessionContentEditor.vue";
import SessionDetailPanel from "../components/sessions/SessionDetailPanel.vue";
import SessionFormDialog from "../components/sessions/SessionFormDialog.vue";
import SessionListPanel from "../components/sessions/SessionListPanel.vue";
import SessionProjectEditor from "../components/sessions/SessionProjectEditor.vue";
import SessionTaskEditor from "../components/sessions/SessionTaskEditor.vue";
import SessionTimerPanel from "../components/sessions/SessionTimerPanel.vue";
import {useSessionsStore, type TimerSnapshot} from "../stores/sessions";

const store = useSessionsStore();

const formOpen = ref(false);
const editing = ref<SessionDetailItem | null>(null);
const preset = ref<TimerSnapshot | null>(null);

function openCreate(): void {
  editing.value = null;
  preset.value = null;
  formOpen.value = true;
}

function openEdit(): void {
  editing.value = store.selected;
  preset.value = null;
  formOpen.value = true;
}

/** 计时结束后把区间带进表单：用户还能补标题与描述，再一次性提交 */
function openFromTimer(snapshot: TimerSnapshot): void {
  editing.value = null;
  preset.value = snapshot;
  formOpen.value = true;
}

async function openItem(id: string): Promise<void> {
  await store.select(id);
}

async function remove(id: string): Promise<void> {
  const confirmed = window.confirm(
    "删除后这次学习活动会从列表与统计中消失，但归因数据与证据保留（软删除）。确定删除？",
  );

  if (!confirmed) {
    return;
  }

  await store.remove(id);
}

onMounted(async () => {
  await Promise.all([store.load(), store.loadOptions(), store.loadEvidence()]);
});
</script>

<template>
  <main class="sessions-page">
    <div class="sessions-container">
      <header class="page-header">
        <div>
          <h1>学习活动</h1>
          <p>
            记录已经发生的一段学习：时间区间、有效时长、归因到哪些项目与内容。
            这里不排计划——计划在「学习计划」，记录只写事实。
          </p>
        </div>
      </header>

      <p v-if="store.error" class="error-banner" role="alert">{{ store.error }}</p>

      <div class="sessions-body">
        <article class="column-card">
          <SessionListPanel @create="openCreate" @open="openItem"/>
        </article>

        <div class="detail-column">
          <SessionTimerPanel @finish="openFromTimer"/>

          <p v-if="store.detailLoading" class="loading-hint">加载详情…</p>

          <SessionDetailPanel
            v-if="store.selected"
            @edit="openEdit"
            @remove="remove"
          />

          <template v-if="store.selected">
            <SessionProjectEditor/>
            <SessionContentEditor/>
            <SessionTaskEditor/>
          </template>

          <EvidencePanel/>
        </div>
      </div>
    </div>

    <SessionFormDialog
      :open="formOpen"
      :preset="preset"
      :session="editing"
      @close="formOpen = false"
    />
  </main>
</template>

<style scoped>
.sessions-page {
  min-height: 100%;
  color: var(--ink-1);
}

.sessions-container {
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  color: var(--ink-1);
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.page-header p {
  max-width: 76ch;
  margin: 9px 0 0;

  color: var(--ink-3);
  font-size: 14px;
  line-height: 1.6;
}

.error-banner {
  padding: 11px 14px;
  margin: 0 0 16px;

  border: 1px solid #f7dada;
  border-radius: 10px;

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 13px;
  line-height: 1.6;
}

.sessions-body {
  display: grid;
  gap: 16px;
  align-items: start;
  grid-template-columns: minmax(0, 420px) minmax(0, 1fr);
}

.column-card {
  padding: 20px 22px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.detail-column {
  display: flex;
  flex-direction: column;
  gap: 16px;

  min-width: 0;
}

.loading-hint {
  margin: 0;
  color: var(--ink-4);
  font-size: 12px;
}

@media (max-width: 1080px) {
  .sessions-body {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .column-card {
    padding: 18px 16px;
  }
}
</style>
