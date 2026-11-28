<script setup lang="ts">
import {computed} from "vue";
import StatusPill from "../ui/StatusPill.vue";
import {useContentsStore} from "../../stores/contents";
import {formatDateTime} from "../../lib/datetime";
import {
  BODY_FORMAT_LABELS,
  CONTENT_SOURCE_LABELS,
  CONTENT_STATUS_LABELS,
  CONTENT_TYPE_LABELS,
  contentTone,
} from "../../views/contents/contents-rules";

const emit = defineEmits<{ edit: [] }>();

const store = useContentsStore();

const item = computed(() => store.selected);

async function archive(): Promise<void> {
  const current = item.value;

  if (!current) {
    return;
  }

  // 归档是终态：契约没有恢复端点，确认文案必须说清
  const confirmed = window.confirm(
    `归档「${current.title}」？归档后它不能加入项目，也不提供恢复入口。`,
  );

  if (confirmed) {
    await store.remove(current.id);
  }
}

async function purge(): Promise<void> {
  const current = item.value;

  if (!current) {
    return;
  }

  const confirmed = window.confirm(
    `彻底删除「${current.title}」？物理删除后不可恢复。`,
  );

  if (confirmed) {
    await store.remove(current.id, true);
  }
}
</script>

<template>
  <section class="detail-panel">
    <p v-if="!item" class="panel-state">
      {{ store.detailLoading ? "加载中…" : "从左侧选择一份内容查看正文、关系、标签与项目使用情况。" }}
    </p>

    <template v-else>
      <header class="detail-head">
        <div class="detail-heading">
          <h2>{{ item.title }}</h2>

          <div class="detail-meta">
            <StatusPill :label="CONTENT_TYPE_LABELS[item.content_type]" tone="neutral"/>
            <StatusPill
              :label="CONTENT_STATUS_LABELS[item.status]"
              :tone="contentTone(item.status)"
            />
            <span class="meta-plain">{{ CONTENT_SOURCE_LABELS[item.created_source] }}</span>
            <span class="meta-plain">第 {{ item.depth + 1 }} 层</span>
          </div>
        </div>

        <div class="detail-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="emit('edit')">
            编辑
          </button>

          <button
            v-if="item.status === 'active'"
            class="pf-btn pf-btn--ghost"
            type="button"
            @click="archive"
          >
            归档
          </button>

          <button
            v-if="item.status === 'archived'"
            class="pf-btn pf-btn--danger"
            type="button"
            @click="purge"
          >
            彻底删除
          </button>
        </div>
      </header>

      <p v-if="item.description" class="detail-summary">{{ item.description }}</p>

      <div class="detail-block">
        <h3>
          正文
          <span class="block-hint">{{ BODY_FORMAT_LABELS[item.body_format] }} 原文</span>
        </h3>

        <pre v-if="item.body" class="body-text">{{ item.body }}</pre>
        <p v-else class="body-empty">这份内容还没有正文。</p>
      </div>

      <div v-if="item.source_url" class="detail-block">
        <h3>出处</h3>

        <a :href="item.source_url" class="source-link" rel="noopener noreferrer" target="_blank">
          {{ item.source_url }}
        </a>
      </div>

      <p class="detail-foot">
        创建于 {{ formatDateTime(item.created_at) }} · 更新于 {{ formatDateTime(item.updated_at) }}
        <template v-if="item.status === 'archived'">
          · 彻底删除要求：无项目关联且无子内容
        </template>
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
  padding: 48px 20px;
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
  gap: 7px;

  margin-top: 10px;
}

.meta-plain {
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

.detail-summary {
  padding: 12px 14px;
  margin: 0;

  border-radius: var(--radius-sm);

  background: var(--surface-muted);
  color: var(--ink-2);

  font-size: 13px;
  line-height: 1.75;
}

.detail-block h3 {
  margin: 0 0 10px;

  color: var(--ink-2);
  font-size: 13px;
  font-weight: 650;
}

.block-hint {
  margin-left: 6px;
  color: var(--ink-4);
  font-size: 11.5px;
  font-weight: 400;
}

/* 正文只做原样展示：不解析 Markdown、不注入 HTML */
.body-text {
  max-height: 460px;
  padding: 14px 16px;
  margin: 0;
  overflow: auto;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: var(--surface-muted);
  color: var(--ink-1);

  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12.8px;
  line-height: 1.8;
  white-space: pre-wrap;
  word-break: break-word;
}

.body-empty {
  margin: 0;
  color: var(--ink-4);
  font-size: 13px;
}

.source-link {
  overflow-wrap: anywhere;

  color: var(--brand);
  font-size: 12.5px;
  text-decoration: none;
}

.source-link:hover {
  text-decoration: underline;
}

.detail-foot {
  margin: 0;

  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.7;
}
</style>
