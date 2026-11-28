<script setup lang="ts">
import {computed} from "vue";
import {formatDateTime} from "../../lib/datetime";
import StatusPill from "../ui/StatusPill.vue";
import {useSessionsStore} from "../../stores/sessions";
import {
  ACTIVITY_TYPE_LABELS,
  CONTRIBUTION_TYPE_LABELS,
  SESSION_SOURCE_LABELS,
  activityTone,
  formatMinutes,
  formatTimeRange,
} from "../../views/sessions/session-rules";

const emit = defineEmits<{ edit: []; remove: [id: string] }>();

const store = useSessionsStore();

const item = computed(() => store.selected);

const sections = computed(() => {
  const session = store.selected;

  if (!session) {
    return [];
  }

  return [
    {label: "做了什么", value: session.what_done},
    {label: "遇到的问题", value: session.problems},
    {label: "下一步", value: session.next_steps},
  ].filter((section) => Boolean(section.value));
});
</script>

<template>
  <section class="detail-panel">
    <p v-if="!item" class="panel-state">
      从左侧选择一次学习活动查看详情；也可以直接在右侧开始计时。
    </p>

    <template v-else>
      <header class="detail-head">
        <div class="detail-heading">
          <h2>{{ item.title || "未命名活动" }}</h2>

          <div class="detail-meta">
            <StatusPill
              :label="ACTIVITY_TYPE_LABELS[item.activity_type]"
              :tone="activityTone(item.activity_type)"
            />
            <span class="meta-plain">{{ formatTimeRange(item.started_at, item.ended_at) }}</span>
            <span class="meta-strong">{{ formatMinutes(item.duration_minutes) }}</span>
            <span class="meta-plain">主要项目：{{ store.projectName(item.primary_project_id) }}</span>
          </div>

          <p class="detail-source">
            {{ SESSION_SOURCE_LABELS[item.created_source] }} · {{ formatDateTime(item.created_at) }} 录入
          </p>
        </div>

        <div class="detail-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="emit('edit')">编辑</button>
          <button class="pf-btn pf-btn--danger" type="button" @click="emit('remove', item.id)">
            删除
          </button>
        </div>
      </header>

      <section v-if="item.summary" class="detail-section">
        <h3>概述</h3>
        <p class="summary-text">{{ item.summary }}</p>
      </section>

      <section v-for="section in sections" :key="section.label" class="detail-section">
        <h3>{{ section.label }}</h3>
        <p class="body-text">{{ section.value }}</p>
      </section>

      <section v-if="item.contents.length > 0" class="detail-section">
        <h3>涉及内容</h3>
        <ul class="chip-list">
          <li v-for="content in item.contents" :key="content.content_id" class="chip">
            <span class="chip-name">{{ store.contentTitle(content.content_id) }}</span>
            <span class="chip-tag">{{ CONTRIBUTION_TYPE_LABELS[content.contribution_type] }}</span>
          </li>
        </ul>
      </section>

      <p v-if="!item.summary && sections.length === 0" class="empty-hint">
        这次活动还没有写描述。补一句「做了什么」会让之后的复盘有据可查。
      </p>

      <p class="soft-delete-note">
        删除是软删除：记录从列表与统计里消失，但归因与证据保留（ADR-010 决策 3）。
      </p>
    </template>
  </section>
</template>

<style scoped>
.detail-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;

  padding: 20px 22px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.panel-state {
  padding: 44px 16px;
  margin: 0;

  color: var(--ink-4);
  font-size: 13px;
  line-height: 1.8;
  text-align: center;
}

.detail-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.detail-heading {
  min-width: 0;
}

.detail-heading h2 {
  margin: 0;

  color: var(--ink-1);
  font-size: 19px;
  font-weight: 680;
  letter-spacing: -0.2px;
  line-height: 1.4;
}

.detail-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;

  margin-top: 10px;
}

.meta-plain {
  color: var(--ink-3);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.meta-strong {
  color: var(--brand);
  font-size: 12.5px;
  font-weight: 650;
}

.detail-source {
  margin: 8px 0 0;
  color: var(--ink-4);
  font-size: 11.5px;
}

.detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.detail-actions .pf-btn {
  min-height: 34px;
  padding: 0 14px;
  font-size: 12.5px;
}

.detail-section h3 {
  margin: 0 0 6px;
  color: var(--ink-3);
  font-size: 12px;
  font-weight: 650;
  letter-spacing: 0.2px;
}

.summary-text,
.body-text {
  margin: 0;
  color: var(--ink-2);
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
}

.summary-text {
  padding: 12px 14px;

  border-radius: var(--radius-sm);

  background: var(--surface-muted);
}

.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;

  padding: 5px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
}

.chip-name {
  color: var(--ink-1);
  font-size: 12.5px;
  font-weight: 600;
}

.chip-tag {
  color: var(--ink-4);
  font-size: 11px;
}

.empty-hint {
  padding: 12px 14px;
  margin: 0;

  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-sm);

  color: var(--ink-4);
  font-size: 12.5px;
  line-height: 1.7;
}

.soft-delete-note {
  margin: 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}
</style>
