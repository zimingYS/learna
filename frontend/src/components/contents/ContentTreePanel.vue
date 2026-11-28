<script setup lang="ts">
import {ref} from "vue";
import StatusPill from "../ui/StatusPill.vue";
import {useContentsStore} from "../../stores/contents";
import {formatDate} from "../../lib/datetime";
import {
  CONTENT_STATUS_FILTER_OPTIONS,
  CONTENT_STATUS_LABELS,
  CONTENT_TYPE_FILTER_OPTIONS,
  CONTENT_TYPE_LABELS,
  contentTone,
} from "../../views/contents/contents-rules";

const emit = defineEmits<{ create: []; open: [id: string] }>();

const store = useContentsStore();

const draftKeyword = ref(store.keyword);

function onTypeChange(event: Event): void {
  void store.setType((event.target as HTMLSelectElement).value as typeof store.typeFilter);
}

function onStatusChange(event: Event): void {
  void store.setStatus((event.target as HTMLSelectElement).value as typeof store.statusFilter);
}

function search(): void {
  void store.setKeyword(draftKeyword.value);
}

function openItem(id: string): void {
  emit("open", id);
}
</script>

<template>
  <section class="tree-panel">
    <header class="panel-head">
      <div>
        <h2>内容库</h2>
        <p>共 {{ store.total }} 条 · 当前层级直接列出子内容</p>
      </div>

      <button class="pf-btn pf-btn--primary create-btn" type="button" @click="emit('create')">
        新建内容
      </button>
    </header>

    <div class="filter-bar">
      <select
        :value="store.typeFilter"
        aria-label="按类型筛选"
        class="pf-input filter-select"
        @change="onTypeChange"
      >
        <option v-for="option in CONTENT_TYPE_FILTER_OPTIONS" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>

      <select
        :value="store.statusFilter"
        aria-label="按状态筛选"
        class="pf-input filter-select"
        @change="onStatusChange"
      >
        <option v-for="option in CONTENT_STATUS_FILTER_OPTIONS" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>

      <div class="search-row">
        <input
          v-model="draftKeyword"
          aria-label="按标题搜索"
          class="pf-input"
          placeholder="搜索标题…"
          type="search"
          @keyup.enter="search"
        />

        <button class="pf-btn pf-btn--ghost" type="button" @click="search">搜索</button>
      </div>
    </div>

    <nav aria-label="内容层级" class="crumbs">
      <button class="crumb" type="button" @click="store.goTo(-1)">全部内容</button>

      <template v-for="(crumb, index) in store.crumbs" :key="crumb.id">
        <span aria-hidden="true" class="crumb-sep">/</span>
        <button
          :class="['crumb', {'crumb--current': index === store.crumbs.length - 1}]"
          type="button"
          @click="store.goTo(index)"
        >
          {{ crumb.title }}
        </button>
      </template>
    </nav>

    <p v-if="store.loading" class="panel-state">加载中…</p>

    <p v-else-if="store.items.length === 0" class="panel-state">
      这一层还没有内容。可以新建，或用搜索在全部内容里找。
    </p>

    <ul v-else class="content-list">
      <li v-for="item in store.items" :key="item.id">
        <article :class="['content-card', {'content-card--active': store.selected?.id === item.id}]">
          <button class="card-main" type="button" @click="openItem(item.id)">
            <span class="card-title">{{ item.title }}</span>

            <span class="card-meta">
              <StatusPill
                :label="CONTENT_TYPE_LABELS[item.content_type]"
                tone="neutral"
              />
              <StatusPill
                :label="CONTENT_STATUS_LABELS[item.status]"
                :tone="contentTone(item.status)"
              />
              <span class="card-date">更新于 {{ formatDate(item.updated_at) }}</span>
            </span>

            <span v-if="item.description" class="card-summary">{{ item.description }}</span>
          </button>

          <button class="card-enter" type="button" @click="store.enter(item)">
            看子级
          </button>
        </article>
      </li>
    </ul>

    <footer v-if="store.totalPages > 1" class="pager">
      <button
        :disabled="store.page <= 1"
        class="pager-btn"
        type="button"
        @click="store.setPage(store.page - 1)"
      >
        上一页
      </button>

      <span class="pager-text">{{ store.page }} / {{ store.totalPages }}</span>

      <button
        :disabled="store.page >= store.totalPages"
        class="pager-btn"
        type="button"
        @click="store.setPage(store.page + 1)"
      >
        下一页
      </button>
    </footer>
  </section>
</template>

<style scoped>
.tree-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.panel-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.panel-head h2 {
  margin: 0;
  color: var(--ink-1);
  font-size: 16px;
  font-weight: 650;
}

.panel-head p {
  margin: 6px 0 0;
  color: var(--ink-4);
  font-size: 12px;
}

.create-btn {
  min-height: 36px;
  padding: 0 16px;
  font-size: 13px;
}

.filter-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-select {
  height: 38px;
}

.search-row {
  display: flex;
  gap: 8px;
}

.search-row .pf-input {
  flex: 1;
  min-width: 0;
  height: 38px;
}

.crumbs {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;

  font-size: 12.5px;
}

.crumb {
  padding: 3px 7px;

  border: none;
  border-radius: 7px;

  background: transparent;
  color: var(--ink-3);

  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
}

.crumb:hover {
  background: var(--surface-muted);
  color: var(--brand);
}

.crumb--current {
  color: var(--ink-1);
  font-weight: 600;
}

.crumb-sep {
  color: var(--ink-4);
}

.panel-state {
  padding: 36px 18px;
  margin: 0;

  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-md);

  color: var(--ink-4);
  font-size: 13px;
  line-height: 1.7;
  text-align: center;
}

.content-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.content-card {
  display: flex;
  align-items: stretch;
  gap: 4px;

  border: 1px solid var(--line);
  border-radius: var(--radius-md);

  background: #fff;

  transition: border-color 0.18s var(--ease), box-shadow 0.18s var(--ease);
}

.content-card:hover {
  border-color: var(--line-strong);
  box-shadow: var(--shadow-xs);
}

.content-card--active {
  border-color: var(--brand);
  box-shadow: 0 0 0 3px var(--brand-ring);
}

.card-main {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 7px;

  padding: 12px 14px;

  border: none;
  border-radius: var(--radius-md) 0 0 var(--radius-md);

  background: transparent;
  color: inherit;

  font-family: inherit;
  text-align: left;
  cursor: pointer;
}

.card-title {
  overflow: hidden;

  color: var(--ink-1);
  font-size: 13.5px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.card-date {
  color: var(--ink-4);
  font-size: 11.5px;
}

.card-summary {
  display: -webkit-box;
  overflow: hidden;

  color: var(--ink-3);
  font-size: 12.5px;
  line-height: 1.6;

  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.card-enter {
  flex-shrink: 0;
  align-self: center;

  margin-right: 10px;
  padding: 6px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: var(--surface);
  color: var(--ink-3);

  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
}

.card-enter:hover {
  border-color: var(--brand);
  color: var(--brand);
}

.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;

  padding-top: 4px;
}

.pager-btn {
  padding: 6px 14px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-2);

  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
}

.pager-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pager-text {
  color: var(--ink-4);
  font-size: 12px;
}
</style>
