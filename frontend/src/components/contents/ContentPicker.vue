<script setup lang="ts">
import {ref} from "vue";
import {fetchContents, type ContentItem} from "../../api/contents";
import {describeError} from "../../api/http";
import {CONTENT_TYPE_LABELS} from "../../views/contents/contents-rules";

// 关系目标与上级内容都只能从「已有内容」里选，因此共用一个搜索选择器
const props = defineProps<{
  /** 不出现在结果里的 id（自身、已选中的目标） */
  excludeIds?: string[];
  placeholder?: string;
}>();

const emit = defineEmits<{ pick: [item: ContentItem]; clear: [] }>();

const keyword = ref("");
const results = ref<ContentItem[]>([]);
const loading = ref(false);
const error = ref("");

async function search(): Promise<void> {
  loading.value = true;
  error.value = "";

  try {
    const data = await fetchContents({
      page: 1,
      page_size: 8,
      status: "active",
      q: keyword.value.trim() || undefined,
    });

    const excluded = new Set(props.excludeIds ?? []);
    results.value = data.items.filter((item) => !excluded.has(item.id));
  } catch (e: unknown) {
    error.value = describeError(e);
    results.value = [];
  } finally {
    loading.value = false;
  }
}

function handlePick(item: ContentItem): void {
  results.value = [];
  keyword.value = "";
  emit("pick", item);
}
</script>

<template>
  <div class="content-picker">
    <div class="picker-row">
      <input
        v-model="keyword"
        class="pf-input"
        :placeholder="placeholder ?? '按标题搜索内容…'"
        type="text"
        @keyup.enter="search"
      />

      <button :disabled="loading" class="pf-btn pf-btn--ghost" type="button" @click="search">
        {{ loading ? "搜索中…" : "搜索" }}
      </button>
    </div>

    <p v-if="error" class="pf-error">{{ error }}</p>

    <ul v-if="results.length > 0" class="picker-results">
      <li v-for="item in results" :key="item.id">
        <button class="picker-item" type="button" @click="handlePick(item)">
          <span class="picker-title">{{ item.title }}</span>
          <span class="picker-type">{{ CONTENT_TYPE_LABELS[item.content_type] }}</span>
        </button>
      </li>
    </ul>

    <button class="picker-clear" type="button" @click="emit('clear')">清除选择</button>
  </div>
</template>

<style scoped>
.content-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.picker-row {
  display: flex;
  gap: 8px;
}

.picker-row .pf-input {
  flex: 1;
  min-width: 0;
}

.picker-results {
  max-height: 200px;
  padding: 4px;
  margin: 0;
  overflow-y: auto;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: var(--surface);

  list-style: none;
}

.picker-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;

  width: 100%;
  padding: 8px 10px;

  border: none;
  border-radius: 8px;

  background: transparent;
  color: var(--ink-1);

  font-family: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.picker-item:hover {
  background: var(--brand-soft);
  color: var(--brand);
}

.picker-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-type {
  flex-shrink: 0;
  color: var(--ink-4);
  font-size: 11.5px;
}

.picker-clear {
  align-self: flex-start;

  padding: 0;

  border: none;
  background: transparent;
  color: var(--ink-4);

  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.picker-clear:hover {
  color: var(--brand);
}
</style>
