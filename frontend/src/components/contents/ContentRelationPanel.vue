<script setup lang="ts">
import {computed, ref} from "vue";
import type {ContentItem, RelationItem, RelationType} from "../../api/contents";
import ContentPicker from "./ContentPicker.vue";
import {useContentsStore} from "../../stores/contents";
import {
  RELATION_TYPE_OPTIONS,
  relationLabel,
  relationPeerId,
} from "../../views/contents/contents-rules";

const store = useContentsStore();

const target = ref<ContentItem | null>(null);
const relationType = ref<RelationType>("prerequisite");
const submitting = ref(false);

const current = computed(() => store.selected);

// 不能选自身，也不必再选已经有关系的内容（重复会命中唯一约束）
const excludedIds = computed(() => {
  const ids = store.relations.map((item) => relationPeerId(item, current.value?.id ?? ""));

  return current.value ? [current.value.id, ...ids] : ids;
});

function pickTarget(item: ContentItem): void {
  target.value = item;
}

function clearTarget(): void {
  target.value = null;
}

async function add(): Promise<void> {
  const content = current.value;

  if (!content || !target.value || submitting.value) {
    return;
  }

  submitting.value = true;

  try {
    if (await store.addRelation(content.id, target.value.id, relationType.value)) {
      target.value = null;
    }
  } finally {
    submitting.value = false;
  }
}

function peerId(relation: RelationItem): string {
  return relationPeerId(relation, current.value?.id ?? "");
}

function peerTitle(relation: RelationItem): string {
  return store.titles[peerId(relation)] ?? "（内容不可见）";
}

async function openPeer(relation: RelationItem): Promise<void> {
  await store.select(peerId(relation));
}
</script>

<template>
  <section class="relation-panel">
    <header class="panel-head">
      <h2>内容关系</h2>
      <p>单向存储、双向查询；前置/包含这类关系会被用来判断学习顺序。</p>
    </header>

    <p v-if="store.relations.length === 0" class="panel-state">
      还没有建立关系。前置关系会参与成环检测，逻辑上不能出现 A 前置 B 又 B 前置 A。
    </p>

    <ul v-else class="relation-list">
      <li v-for="relation in store.relations" :key="relation.id" class="relation-row">
        <span
          :class="['relation-tag', `relation-tag--${relation.direction}`]"
          :title="relation.direction === 'outbound' ? '本内容 → 对方' : '对方 → 本内容'"
        >
          {{ relationLabel(relation.relation_type, relation.direction) }}
        </span>

        <button class="relation-peer" type="button" @click="openPeer(relation)">
          {{ peerTitle(relation) }}
        </button>

        <button
          class="relation-remove"
          type="button"
          @click="store.removeRelation(current?.id ?? '', relation.id)"
        >
          删除
        </button>
      </li>
    </ul>

    <div class="relation-add">
      <div class="add-row">
        <select v-model="relationType" aria-label="关系类型" class="pf-input type-select">
          <option v-for="option in RELATION_TYPE_OPTIONS" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>

        <button
          :disabled="!target || submitting"
          class="pf-btn pf-btn--primary add-btn"
          type="button"
          @click="add"
        >
          {{ submitting ? "添加中…" : "添加关系" }}
        </button>
      </div>

      <p class="picked">
        已选：<strong>{{ target ? target.title : "未选择" }}</strong>
      </p>

      <ContentPicker
        :exclude-ids="excludedIds"
        placeholder="搜索要关联的内容…"
        @pick="pickTarget"
        @clear="clearTarget"
      />
    </div>
  </section>
</template>

<style scoped>
.relation-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 20px 22px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.panel-head h2 {
  margin: 0;
  color: var(--ink-1);
  font-size: 15px;
  font-weight: 650;
}

.panel-head p {
  margin: 6px 0 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}

.panel-state {
  margin: 0;
  color: var(--ink-4);
  font-size: 12.5px;
  line-height: 1.75;
}

.relation-list {
  display: flex;
  flex-direction: column;
  gap: 6px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.relation-row {
  display: flex;
  align-items: center;
  gap: 8px;

  padding: 7px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: var(--surface-muted);
}

.relation-tag {
  flex-shrink: 0;

  padding: 2px 8px;

  border-radius: 6px;

  font-size: 11.5px;
  font-weight: 600;
}

.relation-tag--outbound {
  background: var(--brand-soft);
  color: var(--brand);
}

.relation-tag--inbound {
  background: #eef1f8;
  color: var(--ink-3);
}

.relation-peer {
  overflow: hidden;
  flex: 1;
  min-width: 0;

  padding: 0;

  border: none;
  background: transparent;
  color: var(--ink-1);

  font-family: inherit;
  font-size: 12.8px;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.relation-peer:hover {
  color: var(--brand);
}

.relation-remove {
  flex-shrink: 0;

  padding: 3px 8px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-3);

  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
}

.relation-remove:hover {
  border-color: #f7dada;
  color: var(--rose);
}

.relation-add {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding-top: 10px;
  border-top: 1px dashed var(--line);
}

.add-row {
  display: flex;
  gap: 8px;
}

.type-select {
  flex: 1;
  height: 38px;
}

.add-btn {
  min-height: 38px;
  padding: 0 16px;
  font-size: 12.5px;
}

.picked {
  margin: 0;
  color: var(--ink-4);
  font-size: 12px;
}

.picked strong {
  color: var(--ink-2);
  font-weight: 600;
}
</style>
