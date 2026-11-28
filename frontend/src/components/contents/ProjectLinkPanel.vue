<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import type {ProjectContentStatus} from "../../api/contents";
import {describeError} from "../../api/http";
import {fetchProjects, type ProjectItem} from "../../api/projects";
import StatusPill from "../ui/StatusPill.vue";
import {useContentsStore} from "../../stores/contents";
import {
  attachBlockedReason,
  CONTENT_PRIORITY_OPTIONS,
  MASTERY_LEVEL_OPTIONS,
  PROJECT_STATUS_LABELS,
  PROJECT_STATUS_OPTIONS,
  projectStatusTone,
} from "../../views/contents/contents-rules";

const store = useContentsStore();

const projects = ref<ProjectItem[]>([]);
const projectsError = ref("");
const attachProjectId = ref("");
const attachMastery = ref("0");
const attachPriority = ref("3");
const attachAutoReview = ref(false);
const attaching = ref(false);

const content = computed(() => store.selected);
const blockedReason = computed(() =>
  content.value ? attachBlockedReason(content.value.status) : null,
);

function projectName(id: string): string {
  return projects.value.find((project) => project.id === id)?.name ?? "（项目不可见）";
}

onMounted(async () => {
  try {
    const data = await fetchProjects({page: 1, page_size: 100, include_archived: true});
    projects.value = data.items;
  } catch (e: unknown) {
    projectsError.value = describeError(e);
  }
});

async function attach(): Promise<void> {
  const current = content.value;

  if (!current || !attachProjectId.value || attaching.value) {
    return;
  }

  attaching.value = true;

  try {
    const ok = await store.attachToProject(attachProjectId.value, current.id);

    if (ok) {
      attachProjectId.value = "";
      attachMastery.value = "0";
      attachPriority.value = "3";
      attachAutoReview.value = false;
    }
  } finally {
    attaching.value = false;
  }
}

function updateStatus(linkId: string, value: string): void {
  void store.updateLink(linkId, {
    current_status: value as Exclude<ProjectContentStatus, "achieved">,
  });
}

function updateMastery(linkId: string, value: string): void {
  void store.updateLink(linkId, {target_mastery_level: Number(value)});
}

function updatePriority(linkId: string, value: string): void {
  void store.updateLink(linkId, {priority: Number(value)});
}

function toggleAutoReview(linkId: string, value: boolean): void {
  void store.updateLink(linkId, {auto_review_enabled: value});
}
</script>

<template>
  <section class="link-panel">
    <header class="panel-head">
      <div>
        <h2>项目中使用</h2>
        <p>状态是项目级的：同一份内容在 A 项目「已达成」、在 B 项目「未开始」是正常的。</p>
      </div>
    </header>

    <p v-if="projectsError" class="pf-error">{{ projectsError }}</p>

    <p v-if="store.projectLinks.length === 0" class="panel-state">
      这份内容还没有加入任何项目。
    </p>

    <ul v-else class="link-list">
      <li v-for="link in store.projectLinks" :key="link.id" class="link-row">
        <div class="link-head">
          <span class="link-project">{{ projectName(link.project_id) }}</span>
          <StatusPill
            :label="PROJECT_STATUS_LABELS[link.current_status]"
            :tone="projectStatusTone(link.current_status)"
          />

          <button
            class="link-detach"
            type="button"
            @click="store.detach(link.id)"
          >
            解除关联
          </button>
        </div>

        <div class="link-controls">
          <label class="ctl">
            <span class="ctl-label">状态</span>
            <select
              :disabled="link.current_status === 'achieved'"
              :value="link.current_status"
              :title="
                link.current_status === 'achieved'
                  ? '已达成由掌握度评价流程写入，不能手工改回'
                  : '手工可写的状态不含「已达成」'
              "
              class="ctl-select"
              @change="updateStatus(link.id, ($event.target as HTMLSelectElement).value)"
            >
              <option v-if="link.current_status === 'achieved'" value="achieved">已达成</option>
              <option v-for="option in PROJECT_STATUS_OPTIONS" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </label>

          <label class="ctl">
            <span class="ctl-label">目标等级</span>
            <select
              :value="String(link.target_mastery_level)"
              class="ctl-select"
              @change="updateMastery(link.id, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="option in MASTERY_LEVEL_OPTIONS" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </label>

          <label class="ctl">
            <span class="ctl-label">优先级</span>
            <select
              :value="String(link.priority)"
              class="ctl-select"
              @change="updatePriority(link.id, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="option in CONTENT_PRIORITY_OPTIONS" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </label>

          <label class="ctl ctl--checkbox">
            <input
              :checked="link.auto_review_enabled"
              type="checkbox"
              @change="toggleAutoReview(link.id, ($event.target as HTMLInputElement).checked)"
            />
            <span class="ctl-label">自动复习</span>
          </label>
        </div>
      </li>
    </ul>

    <div class="attach-block">
      <p v-if="blockedReason" class="panel-state panel-state--warn">{{ blockedReason }}</p>

      <template v-else>
        <div class="attach-row">
          <select v-model="attachProjectId" aria-label="选择项目" class="pf-input">
            <option value="">加入项目…</option>
            <option v-for="project in projects" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </select>

          <button
            :disabled="!attachProjectId || attaching"
            class="pf-btn pf-btn--primary attach-btn"
            type="button"
            @click="attach"
          >
            {{ attaching ? "加入中…" : "加入" }}
          </button>
        </div>

        <div class="attach-row">
          <select v-model="attachMastery" aria-label="初始目标等级" class="pf-input">
            <option v-for="option in MASTERY_LEVEL_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>

          <select v-model="attachPriority" aria-label="初始优先级" class="pf-input">
            <option v-for="option in CONTENT_PRIORITY_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>

          <label class="ctl ctl--checkbox">
            <input v-model="attachAutoReview" type="checkbox"/>
            <span class="ctl-label">自动复习</span>
          </label>
        </div>

        <p class="hint">重复加入同一项目是幂等的：会返回已有那条关联，不会产生第二条。</p>
      </template>
    </div>
  </section>
</template>

<style scoped>
.link-panel {
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
  line-height: 1.7;
}

.panel-state--warn {
  padding: 9px 12px;

  border: 1px solid #f6e3c4;
  border-radius: var(--radius-sm);

  background: #fff8ec;
  color: #b9761f;
}

.link-list {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.link-row {
  display: flex;
  flex-direction: column;
  gap: 9px;

  padding: 11px 12px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);

  background: var(--surface-muted);
}

.link-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.link-project {
  overflow: hidden;
  flex: 1;
  min-width: 0;

  color: var(--ink-1);
  font-size: 12.8px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-detach {
  flex-shrink: 0;

  padding: 3px 9px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-3);

  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
}

.link-detach:hover {
  border-color: #f7dada;
  color: var(--rose);
}

.link-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.ctl {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ctl-label {
  color: var(--ink-4);
  font-size: 11.5px;
  white-space: nowrap;
}

.ctl-select {
  height: 30px;
  padding: 0 8px;

  border: 1px solid var(--line);
  border-radius: 8px;

  background: #fff;
  color: var(--ink-2);

  font-family: inherit;
  font-size: 12px;
}

.ctl-select:disabled {
  background: var(--surface-sunken);
  color: var(--ink-4);
}

.ctl--checkbox {
  cursor: pointer;
}

.attach-block {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding-top: 10px;
  border-top: 1px dashed var(--line);
}

.attach-row {
  display: flex;
  gap: 8px;
}

.attach-row .pf-input {
  flex: 1;
  min-width: 0;
  height: 38px;
}

.attach-btn {
  min-height: 38px;
  padding: 0 18px;
  font-size: 12.5px;
}

.hint {
  margin: 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}
</style>
