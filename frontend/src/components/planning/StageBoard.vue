<script setup lang="ts">
import {computed, ref} from "vue";
import type {MilestoneItem, MilestoneStatus, StageItem} from "../../api/planning";
import {usePlanningStore} from "../../stores/planning";
import {formatDateTime} from "../../lib/datetime";
import ModalDialog from "../ui/ModalDialog.vue";
import MilestoneFormDialog from "./MilestoneFormDialog.vue";
import PlanStatusPill from "./PlanStatusPill.vue";
import StageFormDialog from "./StageFormDialog.vue";
import {
  canAdvanceStage,
  canDeleteStructure,
  canEditMilestone,
  canEditStructure,
  MILESTONE_STATUS_LABELS,
  milestoneStatusTargets,
  ROADMAP_STATUS_LABELS,
  STAGE_STATUS_LABELS,
  stageActions,
  TASK_ACTION_LABELS,
} from "../../views/planning/planning-rules";

const store = usePlanningStore();

const stageFormOpen = ref(false);
const editingStage = ref<StageItem | null>(null);
const pendingDeleteStage = ref<StageItem | null>(null);

const milestoneFormOpen = ref(false);
const editingMilestone = ref<MilestoneItem | null>(null);
const milestoneStageId = ref("");
const pendingDeleteMilestone = ref<MilestoneItem | null>(null);

const roadmap = computed(() => store.activeRoadmap);
const stages = computed(() => store.stages);

// 当前路线允许哪些操作，直接决定按钮的出现与否（与后端准入一一对应）
const structureEditable = computed(() => roadmap.value !== null && canEditStructure(roadmap.value.status));
const stageAdvanceable = computed(() => roadmap.value !== null && canAdvanceStage(roadmap.value.status));
const milestoneEditable = computed(() => roadmap.value !== null && canEditMilestone(roadmap.value.status));
const structureDeletable = computed(
  () => roadmap.value !== null && canDeleteStructure(roadmap.value.status),
);

const guideText = computed(() => {
  if (!roadmap.value) {
    return "先在左侧选择一个路线版本，这里会显示它的阶段与里程碑。";
  }

  if (roadmap.value.status === "draft") {
    return "草稿状态：可以增删改阶段与里程碑；确认生效后才能推进阶段状态。";
  }

  if (roadmap.value.status === "active") {
    return "生效中：阶段可以推进，结构（阶段/里程碑的增删改）已锁定。需要调整请新建版本。";
  }

  return "已废弃：历史版本只读，仅供追溯。";
});

function openCreateStage(): void {
  editingStage.value = null;
  stageFormOpen.value = true;
}

function openEditStage(stage: StageItem): void {
  editingStage.value = stage;
  stageFormOpen.value = true;
}

function openCreateMilestone(stageId: string): void {
  editingMilestone.value = null;
  milestoneStageId.value = stageId;
  milestoneFormOpen.value = true;
}

function openEditMilestone(milestone: MilestoneItem): void {
  editingMilestone.value = milestone;
  milestoneStageId.value = milestone.stage_id;
  milestoneFormOpen.value = true;
}

async function removeStage(): Promise<void> {
  const stage = pendingDeleteStage.value;

  if (!stage) {
    return;
  }

  const deleted = await store.removeStage(stage.id);

  if (deleted !== null) {
    pendingDeleteStage.value = null;
  }
}

async function removeMilestone(): Promise<void> {
  const milestone = pendingDeleteMilestone.value;

  if (!milestone) {
    return;
  }

  const deleted = await store.removeMilestone(milestone.id);

  if (deleted !== null) {
    pendingDeleteMilestone.value = null;
  }
}

async function changeMilestoneStatus(milestone: MilestoneItem, event: Event): Promise<void> {
  const target = (event.target as HTMLSelectElement).value as MilestoneStatus;

  await store.editMilestone(milestone.id, {status: target});
}

function milestonesOf(stageId: string): MilestoneItem[] {
  return store.milestonesByStage.get(stageId) ?? [];
}
</script>

<template>
  <section class="panel">
    <header class="panel-head">
      <div>
        <h2 class="panel-title">
          阶段与里程碑
          <span v-if="roadmap" class="version-tag">v{{ roadmap.version }}</span>
          <PlanStatusPill
            v-if="roadmap"
            :label="ROADMAP_STATUS_LABELS[roadmap.status]"
            :status="roadmap.status"
          />
        </h2>
        <p class="panel-sub">{{ guideText }}</p>
      </div>

      <button
        v-if="structureEditable"
        class="create-btn"
        type="button"
        @click="openCreateStage()"
      >
        <span aria-hidden="true" class="plus">+</span>
        新建阶段
      </button>
    </header>

    <p v-if="store.stagesLoading" class="hint">正在加载阶段…</p>

    <p v-else-if="!roadmap" class="empty-box">尚未选择路线版本。</p>

    <p v-else-if="stages.length === 0" class="empty-box">
      这个版本还没有阶段。按「基础 → 强化 → 冲刺」这类顺序排出阶段，再确认生效。
    </p>

    <ol v-else class="stage-list">
      <li v-for="stage in stages" :key="stage.id" class="stage-item">
        <div class="stage-seq">{{ stage.sequence_no }}</div>

        <div class="stage-body">
          <div class="stage-head">
            <div class="stage-title-group">
              <h3 class="stage-title">{{ stage.name }}</h3>
              <PlanStatusPill :label="STAGE_STATUS_LABELS[stage.status]" :status="stage.status"/>
            </div>

            <div class="stage-actions">
              <button
                v-for="action in stageAdvanceable ? stageActions(stage.status) : []"
                :key="action"
                class="ghost-btn ghost-btn--primary"
                type="button"
                @click="store.moveStage(stage.id, action)"
              >
                {{ TASK_ACTION_LABELS[action] }}
              </button>

              <button
                v-if="structureEditable"
                class="ghost-btn"
                type="button"
                @click="openEditStage(stage)"
              >
                编辑
              </button>

              <button
                v-if="structureDeletable"
                class="ghost-btn ghost-btn--danger"
                type="button"
                @click="pendingDeleteStage = stage"
              >
                删除
              </button>
            </div>
          </div>

          <p v-if="stage.description" class="stage-desc">{{ stage.description }}</p>

          <div class="stage-meta">
            <span class="meta-text">开始：{{ formatDateTime(stage.started_at) }}</span>
            <span class="meta-text">完成：{{ formatDateTime(stage.completed_at) }}</span>
            <span v-if="stage.completion_method !== 'user'" class="meta-text">
              完成方式：{{ stage.completion_method }}
            </span>
          </div>

          <div class="milestone-block">
            <header class="milestone-head">
              <span class="milestone-label">
                里程碑
                <span class="count">{{ milestonesOf(stage.id).length }}</span>
              </span>

              <button
                v-if="milestoneEditable"
                class="link-btn"
                type="button"
                @click="openCreateMilestone(stage.id)"
              >
                + 添加里程碑
              </button>
            </header>

            <p v-if="milestonesOf(stage.id).length === 0" class="milestone-empty">
              暂无里程碑，用来标记阶段内的关键节点。
            </p>

            <ul v-else class="milestone-list">
              <li
                v-for="milestone in milestonesOf(stage.id)"
                :key="milestone.id"
                class="milestone-row"
              >
                <div class="milestone-main">
                  <span class="milestone-title">{{ milestone.title }}</span>
                  <PlanStatusPill
                    :label="MILESTONE_STATUS_LABELS[milestone.status]"
                    :status="milestone.status"
                  />
                </div>

                <div class="milestone-actions">
                  <select
                    v-if="milestoneEditable"
                    :value="milestone.status"
                    class="status-select"
                    @change="changeMilestoneStatus(milestone, $event)"
                  >
                    <option :value="milestone.status" disabled>
                      {{ MILESTONE_STATUS_LABELS[milestone.status] }}
                    </option>
                    <option
                      v-for="target in milestoneStatusTargets(milestone.status)"
                      :key="target"
                      :value="target"
                    >
                      改为{{ MILESTONE_STATUS_LABELS[target] }}
                    </option>
                  </select>

                  <button
                    v-if="milestoneEditable"
                    class="link-btn"
                    type="button"
                    @click="openEditMilestone(milestone)"
                  >
                    编辑
                  </button>

                  <button
                    v-if="structureDeletable"
                    class="link-btn link-btn--danger"
                    type="button"
                    @click="pendingDeleteMilestone = milestone"
                  >
                    删除
                  </button>
                </div>
              </li>
            </ul>
          </div>
        </div>
      </li>
    </ol>

    <StageFormDialog
      :open="stageFormOpen"
      :stage="editingStage"
      @close="stageFormOpen = false"
      @saved="stageFormOpen = false"
    />

    <MilestoneFormDialog
      :milestone="editingMilestone"
      :open="milestoneFormOpen"
      :stage-id="milestoneStageId"
      @close="milestoneFormOpen = false"
      @saved="milestoneFormOpen = false"
    />

    <ModalDialog
      :open="pendingDeleteStage !== null"
      title="删除阶段"
      @close="pendingDeleteStage = null"
    >
      <div class="pf-form">
        <p class="confirm-text">
          将删除阶段「{{ pendingDeleteStage?.name }}」及其里程碑。挂在它下面的任务不会被删除，只会解除阶段与里程碑归属。
        </p>

        <div class="pf-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="pendingDeleteStage = null">
            取消
          </button>
          <button class="pf-btn pf-btn--danger" type="button" @click="removeStage()">
            确认删除
          </button>
        </div>
      </div>
    </ModalDialog>

    <ModalDialog
      :open="pendingDeleteMilestone !== null"
      title="删除里程碑"
      @close="pendingDeleteMilestone = null"
    >
      <div class="pf-form">
        <p class="confirm-text">
          将删除里程碑「{{ pendingDeleteMilestone?.title }}」。挂载它的任务会保留，但不再关联该里程碑（阶段归属保留）。
        </p>

        <div class="pf-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="pendingDeleteMilestone = null">
            取消
          </button>
          <button class="pf-btn pf-btn--danger" type="button" @click="removeMilestone()">
            确认删除
          </button>
        </div>
      </div>
    </ModalDialog>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 0;

  color: var(--ink-1);
  font-size: 17px;
  font-weight: 650;
}

.version-tag {
  padding: 2px 7px;

  border-radius: 5px;

  background: var(--surface-muted);
  color: var(--ink-2);

  font-size: 11px;
  font-weight: 700;
}

.panel-sub {
  margin: 6px 0 0;
  color: var(--ink-3);
  font-size: 13px;
  line-height: 1.6;
}

.create-btn {
  display: inline-flex;
  flex-shrink: 0;
  min-height: 38px;
  align-items: center;
  gap: 7px;
  padding: 0 15px;

  border: none;
  border-radius: var(--radius-pill);

  background: var(--brand);
  color: #fff;

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.create-btn:hover {
  background: var(--brand-strong);
}

.plus {
  font-size: 17px;
  line-height: 1;
}

.hint {
  margin: 0;
  color: var(--ink-3);
  font-size: 13px;
}

.empty-box {
  padding: 34px 20px;
  margin: 0;

  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-md);

  color: var(--ink-3);
  font-size: 13px;
  text-align: center;
}

.stage-list {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.stage-item {
  display: flex;
  gap: 14px;

  padding: 16px 18px;

  border: 1px solid var(--line);
  border-radius: var(--radius-md);

  background: #fff;
}

.stage-seq {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 28px;
  height: 28px;

  border-radius: 8px;

  background: var(--brand-soft);
  color: var(--brand);

  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.stage-body {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 10px;
}

.stage-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.stage-title-group {
  display: flex;
  align-items: center;
  gap: 9px;
}

.stage-title {
  margin: 0;
  color: var(--ink-1);
  font-size: 15px;
  font-weight: 600;
}

.stage-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.ghost-btn {
  padding: 5px 10px;

  border: 1px solid var(--line-strong);
  border-radius: var(--radius-pill);

  background: #fff;
  color: var(--ink-2);

  font-size: 12px;
  cursor: pointer;
}

.ghost-btn:hover {
  border-color: var(--line-strong);
  background: var(--surface-muted);
}

.ghost-btn--primary {
  border-color: #cfd4fb;
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.ghost-btn--primary:hover {
  background: #e3e7fd;
}

.ghost-btn--danger {
  border-color: #f7dada;
  color: #bf3a40;
}

.ghost-btn--danger:hover {
  border-color: #e8c3c3;
  background: var(--rose-soft);
}

.stage-desc {
  margin: 0;
  color: #6f7a8d;
  font-size: 13px;
  line-height: 1.7;
  white-space: pre-wrap;
}

.stage-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
}

.meta-text {
  color: var(--ink-4);
  font-size: 12px;
}

.milestone-block {
  display: flex;
  flex-direction: column;
  gap: 8px;

  padding-top: 10px;
  border-top: 1px solid #f1f4f9;
}

.milestone-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.milestone-label {
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 600;
}

.count {
  margin-left: 5px;
  color: var(--ink-4);
  font-weight: 400;
}

.milestone-empty {
  margin: 0;
  color: var(--ink-4);
  font-size: 12px;
}

.milestone-list {
  display: flex;
  flex-direction: column;
  gap: 6px;

  padding: 0;
  margin: 0;

  list-style: none;
}

.milestone-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;

  padding: 8px 11px;

  border-radius: 8px;

  background: #fafbfe;
}

.milestone-main {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
}

.milestone-title {
  overflow: hidden;

  color: var(--ink-2);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.milestone-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-select {
  height: 28px;
  padding: 0 6px;

  border: 1px solid var(--line-strong);
  border-radius: 6px;

  background: #fff;
  color: var(--ink-2);

  font-family: inherit;
  font-size: 12px;
}

.link-btn {
  padding: 0;

  border: none;

  background: transparent;
  color: var(--brand);

  font-size: 12px;
  cursor: pointer;
}

.link-btn:hover {
  color: var(--brand-strong);
  text-decoration: underline;
}

.link-btn--danger {
  color: #bf3a40;
}

.link-btn--danger:hover {
  color: #a53232;
}

.confirm-text {
  margin: 0;
  color: var(--ink-2);
  font-size: 14px;
  line-height: 1.7;
}
</style>
