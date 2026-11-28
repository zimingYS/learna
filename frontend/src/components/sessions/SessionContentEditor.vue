<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {SessionContributionType} from "../../api/sessions";
import {useSessionsStore} from "../../stores/sessions";
import {buildContentItems, type ContentDraftRow} from "../../views/sessions/session-forms";
import {CONTRIBUTION_TYPE_OPTIONS} from "../../views/sessions/session-rules";

const store = useSessionsStore();

const rows = ref<ContentDraftRow[]>([]);
const localError = ref("");
const saved = ref(false);

watch(
  () => store.selected,
  (session) => {
    rows.value = (session?.contents ?? []).map((item) => ({
      contentId: item.content_id,
      contributionType: item.contribution_type,
      note: item.note ?? "",
    }));
    localError.value = "";
    saved.value = false;
  },
  {immediate: true},
);

/** 下拉选项要覆盖「已选中但不在第一页」的内容，否则回显会变成空白 */
const options = computed(() => {
  const known = new Map(store.contentOptions.map((item) => [item.id, item.title]));

  for (const row of rows.value) {
    if (row.contentId && !known.has(row.contentId)) {
      known.set(row.contentId, store.contentTitle(row.contentId));
    }
  }

  return [...known.entries()].map(([id, title]) => ({id, title}));
});

function add(): void {
  const used = new Set(rows.value.map((row) => row.contentId));
  const next = options.value.find((option) => !used.has(option.id));

  rows.value = [
    ...rows.value,
    {contentId: next?.id ?? "", contributionType: "covered", note: ""},
  ];
  saved.value = false;
}

function remove(index: number): void {
  rows.value = rows.value.filter((_, position) => position !== index);
  saved.value = false;
}

function onContributionChange(index: number, event: Event): void {
  const value = (event.target as HTMLSelectElement).value as SessionContributionType;

  rows.value = rows.value.map((row, position) =>
    position === index ? {...row, contributionType: value} : row,
  );
  saved.value = false;
}

async function save(): Promise<void> {
  const built = buildContentItems(rows.value);

  if (!built.ok) {
    localError.value = built.error;
    return;
  }

  localError.value = "";
  saved.value = await store.replaceContents(built.value);
}
</script>

<template>
  <section class="editor">
    <header class="editor-head">
      <div>
        <h3>关联内容</h3>
        <p>一次活动可以关联多个知识点；保存即整体替换，传空即清空。</p>
      </div>

      <div class="head-actions">
        <button class="pf-btn pf-btn--ghost" type="button" @click="add">加一条</button>
        <button class="pf-btn pf-btn--primary" type="button" @click="save">保存</button>
      </div>
    </header>

    <p v-if="rows.length === 0" class="editor-state">
      还没有关联内容。加一条，把这次学习和内容库里的知识点连起来。
    </p>

    <ul v-else class="row-list">
      <li v-for="(row, index) in rows" :key="index" class="row">
        <select v-model="row.contentId" aria-label="选择内容" class="pf-input row-select">
          <option value="">选择内容…</option>
          <option v-for="option in options" :key="option.id" :value="option.id">
            {{ option.title }}
          </option>
        </select>

        <select
          :value="row.contributionType"
          aria-label="贡献方式"
          class="pf-input row-contribution"
          @change="onContributionChange(index, $event)"
        >
          <option v-for="option in CONTRIBUTION_TYPE_OPTIONS" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>

        <input v-model="row.note" aria-label="备注" class="pf-input row-note" placeholder="备注（可选）"/>

        <button class="row-remove" type="button" @click="remove(index)">移除</button>
      </li>
    </ul>

    <p v-if="localError" class="pf-error">{{ localError }}</p>
    <p v-else-if="saved" class="editor-saved">已保存</p>
  </section>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  gap: 10px;

  padding: 18px 20px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.editor-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.editor-head h3 {
  margin: 0;
  color: var(--ink-1);
  font-size: 14px;
  font-weight: 660;
}

.editor-head p {
  margin: 5px 0 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}

.head-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.head-actions .pf-btn {
  min-height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.editor-state {
  margin: 0;
  color: var(--ink-4);
  font-size: 12.5px;
  line-height: 1.7;
}

.row-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.row .pf-input {
  height: 34px;
  font-size: 12.5px;
}

.row-select {
  flex: 1 1 180px;
  min-width: 150px;
}

.row-contribution {
  width: 116px;
}

.row-note {
  flex: 1 1 140px;
  min-width: 120px;
}

.row-remove {
  padding: 0 6px;

  border: none;
  background: transparent;
  color: var(--ink-4);

  font-family: inherit;
  font-size: 11.5px;
  text-decoration: underline;
  cursor: pointer;
}

.row-remove:hover {
  color: #bf3a40;
}

.editor-saved {
  margin: 0;
  color: #12855a;
  font-size: 11.5px;
}
</style>
