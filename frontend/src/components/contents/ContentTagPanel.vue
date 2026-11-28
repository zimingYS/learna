<script setup lang="ts">
import {computed, ref} from "vue";
import TagVocabularyDialog from "./TagVocabularyDialog.vue";
import {useContentsStore} from "../../stores/contents";

const store = useContentsStore();

const vocabularyOpen = ref(false);
const pendingTagId = ref("");
const newName = ref("");
const newColor = ref("#7d7cf8");
const busy = ref(false);

const currentIds = computed(() => store.contentTags.map((tag) => tag.id));

const unusedTags = computed(() =>
  store.vocabulary.filter((tag) => !currentIds.value.includes(tag.id)),
);

async function applyTagIds(ids: string[]): Promise<void> {
  const content = store.selected;

  if (!content) {
    return;
  }

  busy.value = true;

  try {
    await store.replaceTags(content.id, ids);
  } finally {
    busy.value = false;
  }
}

async function addExisting(): Promise<void> {
  if (!pendingTagId.value) {
    return;
  }

  const next = [...currentIds.value, pendingTagId.value];
  pendingTagId.value = "";
  await applyTagIds(next);
}

async function createAndAdd(): Promise<void> {
  const name = newName.value.trim();

  if (!name) {
    return;
  }

  busy.value = true;

  try {
    const tag = await store.addTag(name, newColor.value);

    if (tag) {
      newName.value = "";
      await applyTagIds([...currentIds.value, tag.id]);
    }
  } finally {
    busy.value = false;
  }
}

function removeTag(tagId: string): void {
  void applyTagIds(currentIds.value.filter((id) => id !== tagId));
}
</script>

<template>
  <section class="tag-panel">
    <header class="panel-head">
      <div>
        <h2>标签</h2>
        <p>标签是私有词表，整体替换语义：一次提交就是一份完整标签集。</p>
      </div>

      <button class="link-btn" type="button" @click="vocabularyOpen = true">管理词表</button>
    </header>

    <div v-if="store.contentTags.length > 0" class="chips">
      <span v-for="tag in store.contentTags" :key="tag.id" class="chip">
        <span
          v-if="tag.color"
          :style="{backgroundColor: tag.color}"
          aria-hidden="true"
          class="chip-dot"
        ></span>

        {{ tag.name }}

        <button
          :disabled="busy"
          aria-label="移除标签"
          class="chip-remove"
          type="button"
          @click="removeTag(tag.id)"
        >
          ×
        </button>
      </span>
    </div>

    <p v-else class="panel-state">还没有标签。</p>

    <div class="tag-add">
      <div v-if="unusedTags.length > 0" class="add-row">
        <select v-model="pendingTagId" aria-label="从词表选择标签" class="pf-input">
          <option value="">从词表选择…</option>
          <option v-for="tag in unusedTags" :key="tag.id" :value="tag.id">
            {{ tag.name }}
          </option>
        </select>

        <button
          :disabled="!pendingTagId || busy"
          class="pf-btn pf-btn--ghost"
          type="button"
          @click="addExisting"
        >
          添加
        </button>
      </div>

      <div class="add-row">
        <input
          v-model="newName"
          aria-label="新标签名"
          class="pf-input"
          maxlength="60"
          placeholder="新建标签，例如：高频考点"
          type="text"
          @keyup.enter="createAndAdd"
        />

        <input v-model="newColor" aria-label="标签颜色" class="color-input" type="color"/>

        <button
          :disabled="!newName.trim() || busy"
          class="pf-btn pf-btn--ghost"
          type="button"
          @click="createAndAdd"
        >
          新建并打标
        </button>
      </div>
    </div>

    <TagVocabularyDialog :open="vocabularyOpen" @close="vocabularyOpen = false"/>
  </section>
</template>

<style scoped>
.tag-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 20px 22px;

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

.link-btn {
  padding: 0;

  border: none;
  background: transparent;
  color: var(--brand);

  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.link-btn:hover {
  text-decoration: underline;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;

  padding: 4px 6px 4px 10px;

  border: 1px solid #e3e7f3;
  border-radius: var(--radius-pill);

  background: #f4f5fa;
  color: var(--ink-2);

  font-size: 12px;
}

.chip-remove {
  width: 16px;
  height: 16px;
  padding: 0;

  border: none;
  border-radius: 50%;

  background: rgb(255 255 255 / 55%);
  color: inherit;

  font-size: 12px;
  line-height: 1;
  cursor: pointer;
}

.chip-remove:hover {
  background: #fff;
}

.chip-dot {
  flex-shrink: 0;

  width: 8px;
  height: 8px;

  border-radius: 50%;
}

.panel-state {
  margin: 0;
  color: var(--ink-4);
  font-size: 12.5px;
}

.tag-add {
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

.add-row .pf-input {
  flex: 1;
  min-width: 0;
  height: 38px;
}

.add-row .pf-btn {
  min-height: 38px;
  padding: 0 14px;
  font-size: 12.5px;
}

.color-input {
  flex-shrink: 0;

  width: 40px;
  height: 38px;
  padding: 3px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: #fff;
  cursor: pointer;
}
</style>
