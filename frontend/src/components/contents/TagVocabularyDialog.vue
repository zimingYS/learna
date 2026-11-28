<script setup lang="ts">
import {ref, watch} from "vue";
import ModalDialog from "../ui/ModalDialog.vue";
import {useContentsStore} from "../../stores/contents";

const props = defineProps<{ open: boolean }>();

const emit = defineEmits<{ close: [] }>();

const store = useContentsStore();

const editingId = ref("");
const editingName = ref("");
const newName = ref("");
const newColor = ref("#7d7cf8");
const busy = ref(false);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    editingId.value = "";
    newName.value = "";
    void store.loadVocabulary();
  },
);

function startEdit(id: string, name: string): void {
  editingId.value = id;
  editingName.value = name;
}

function cancelEdit(): void {
  editingId.value = "";
  editingName.value = "";
}

async function saveEdit(): Promise<void> {
  const name = editingName.value.trim();

  if (!editingId.value || !name) {
    return;
  }

  busy.value = true;

  try {
    if (await store.renameTag(editingId.value, name)) {
      cancelEdit();
    }
  } finally {
    busy.value = false;
  }
}

async function create(): Promise<void> {
  const name = newName.value.trim();

  if (!name) {
    return;
  }

  busy.value = true;

  try {
    if (await store.addTag(name, newColor.value)) {
      newName.value = "";
    }
  } finally {
    busy.value = false;
  }
}

async function remove(id: string, name: string): Promise<void> {
  const confirmed = window.confirm(
    `删除标签「${name}」？它会同时从所有已打标的内容上移除。`,
  );

  if (confirmed) {
    await store.removeTag(id);
  }
}
</script>

<template>
  <ModalDialog
    description="标签是当前账号的私有词表：同名标签在同一账号下唯一，删除会从所有内容上摘掉。"
    :open="open"
    title="标签词表"
    @close="emit('close')"
  >
    <div class="vocab">
      <div class="create-row">
        <input
          v-model="newName"
          aria-label="新标签名"
          class="pf-input"
          maxlength="60"
          placeholder="新标签名"
          type="text"
          @keyup.enter="create"
        />

        <input v-model="newColor" aria-label="标签颜色" class="color-input" type="color"/>

        <button
          :disabled="!newName.trim() || busy"
          class="pf-btn pf-btn--primary"
          type="button"
          @click="create"
        >
          新建
        </button>
      </div>

      <p v-if="store.vocabulary.length === 0" class="vocab-state">词表还是空的。</p>

      <ul v-else class="vocab-list">
        <li v-for="tag in store.vocabulary" :key="tag.id" class="vocab-row">
          <template v-if="editingId === tag.id">
            <input
              v-model="editingName"
              aria-label="标签名"
              class="pf-input rename-input"
              maxlength="60"
              type="text"
              @keyup.enter="saveEdit"
            />

            <button :disabled="busy" class="row-btn" type="button" @click="saveEdit">保存</button>
            <button class="row-btn" type="button" @click="cancelEdit">取消</button>
          </template>

          <template v-else>
            <span class="vocab-name">
              <span
                v-if="tag.color"
                :style="{backgroundColor: tag.color}"
                aria-hidden="true"
                class="vocab-dot"
              ></span>
              {{ tag.name }}
            </span>

            <button class="row-btn" type="button" @click="startEdit(tag.id, tag.name)">
              重命名
            </button>

            <button class="row-btn row-btn--danger" type="button" @click="remove(tag.id, tag.name)">
              删除
            </button>
          </template>
        </li>
      </ul>

      <p v-if="store.error" class="pf-error">{{ store.error }}</p>
    </div>
  </ModalDialog>
</template>

<style scoped>
.vocab {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.create-row {
  display: flex;
  gap: 8px;
}

.create-row .pf-input {
  flex: 1;
  min-width: 0;
}

.create-row .pf-btn {
  flex-shrink: 0;
  padding: 0 16px;
  font-size: 12.5px;
}

.color-input {
  flex-shrink: 0;

  width: 42px;
  height: 42px;
  padding: 3px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: #fff;
  cursor: pointer;
}

.vocab-state {
  padding: 24px 16px;
  margin: 0;

  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-sm);

  color: var(--ink-4);
  font-size: 12.5px;
  text-align: center;
}

.vocab-list {
  display: flex;
  max-height: 320px;
  flex-direction: column;
  gap: 6px;

  padding: 0;
  margin: 0;
  overflow-y: auto;

  list-style: none;
}

.vocab-row {
  display: flex;
  align-items: center;
  gap: 8px;

  padding: 7px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: var(--surface-muted);
}

.vocab-name {
  display: flex;
  overflow: hidden;
  flex: 1;
  align-items: center;
  gap: 7px;
  min-width: 0;

  color: var(--ink-1);
  font-size: 12.8px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.vocab-dot {
  flex-shrink: 0;

  width: 8px;
  height: 8px;

  border-radius: 50%;
}

.rename-input {
  flex: 1;
  height: 34px;
}

.row-btn {
  flex-shrink: 0;

  padding: 4px 10px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-3);

  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
}

.row-btn:hover {
  border-color: var(--brand);
  color: var(--brand);
}

.row-btn--danger:hover {
  border-color: #f7dada;
  color: var(--rose);
}
</style>
