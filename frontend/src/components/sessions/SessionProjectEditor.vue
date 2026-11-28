<script setup lang="ts">
import {computed, ref, watch} from "vue";
import {useSessionsStore} from "../../stores/sessions";
import {buildProjectIds} from "../../views/sessions/session-forms";

const store = useSessionsStore();

const selected = ref<string[]>([]);
const localError = ref("");
const saved = ref(false);

watch(
  () => store.selected,
  (session) => {
    selected.value = [...(session?.related_project_ids ?? [])];
    localError.value = "";
    saved.value = false;
  },
  {immediate: true},
);

const primaryName = computed(() => store.projectName(store.selected?.primary_project_id ?? null));

function toggle(projectId: string): void {
  saved.value = false;
  selected.value = selected.value.includes(projectId)
    ? selected.value.filter((id) => id !== projectId)
    : [...selected.value, projectId];
}

async function save(): Promise<void> {
  const built = buildProjectIds(selected.value, store.selected?.primary_project_id ?? "");

  if (!built.ok) {
    localError.value = built.error;
    return;
  }

  localError.value = "";
  saved.value = await store.replaceProjects(built.value);
}
</script>

<template>
  <section class="editor">
    <header class="editor-head">
      <div>
        <h3>关联项目</h3>
        <p>主要项目：{{ primaryName }}。主要项目只由上方字段表达，不重复出现在这里。</p>
      </div>

      <button class="pf-btn pf-btn--ghost" type="button" @click="save">保存</button>
    </header>

    <p v-if="store.projectOptions.length === 0" class="editor-state">还没有项目可关联。</p>

    <ul v-else class="project-list">
      <li v-for="project in store.projectOptions" :key="project.id">
        <label
          :class="['project-item', {'project-item--primary': project.id === store.selected?.primary_project_id}]"
        >
          <input
            :checked="project.id === store.selected?.primary_project_id || selected.includes(project.id)"
            :disabled="project.id === store.selected?.primary_project_id"
            type="checkbox"
            @change="toggle(project.id)"
          />
          <span>{{ project.name }}</span>
          <span v-if="project.id === store.selected?.primary_project_id" class="primary-tag">
            主要
          </span>
        </label>
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

.editor-head .pf-btn {
  min-height: 32px;
  padding: 0 12px;
  font-size: 12px;
}

.editor-state {
  margin: 0;
  color: var(--ink-4);
  font-size: 12.5px;
}

.project-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.project-item {
  display: inline-flex;
  align-items: center;
  gap: 7px;

  padding: 7px 11px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  color: var(--ink-2);
  font-size: 12.5px;
  cursor: pointer;
}

.project-item--primary {
  border-color: var(--brand);
  background: var(--brand-soft);
  color: var(--brand);
  cursor: default;
}

.project-item input {
  accent-color: var(--brand);
}

.primary-tag {
  font-size: 10.5px;
  font-weight: 700;
}

.editor-saved {
  margin: 0;
  color: #12855a;
  font-size: 11.5px;
}
</style>
