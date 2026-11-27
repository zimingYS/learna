<script setup lang="ts">
import {computed, ref} from "vue";
import type {GoalItem, GoalStatus} from "../../api/planning";
import {usePlanningStore} from "../../stores/planning";
import {formatDate} from "../../lib/datetime";
import GoalFormDialog from "./GoalFormDialog.vue";
import GoalStagesDialog from "./GoalStagesDialog.vue";
import PlanStatusPill from "./PlanStatusPill.vue";
import ModalDialog from "../ui/ModalDialog.vue";
import {
  GOAL_STATUS_LABELS,
  GOAL_TYPE_LABELS,
  goalStatusTargets,
  groupGoals,
} from "../../views/planning/planning-rules";

const store = usePlanningStore();

const formOpen = ref(false);
const editing = ref<GoalItem | null>(null);
const linksOpen = ref(false);
const linkingGoal = ref<GoalItem | null>(null);

const pendingDelete = ref<GoalItem | null>(null);
const deleteChildCount = ref(0);

const tree = computed(() => groupGoals(store.goals));
const parentOptions = computed(() => tree.value.roots);

const STATUS_TARGET_LABELS: Record<GoalStatus, string> = {
  active: "恢复进行中",
  achieved: "标记达成",
  cancelled: "标记取消",
};

function openCreate(): void {
  editing.value = null;
  formOpen.value = true;
}

function openEdit(goal: GoalItem): void {
  editing.value = goal;
  formOpen.value = true;
}

function openLinks(goal: GoalItem): void {
  linkingGoal.value = goal;
  linksOpen.value = true;
}

async function applyStatus(goal: GoalItem, status: GoalStatus): Promise<void> {
  await store.editGoal(goal.id, {status});
}

function askDelete(goal: GoalItem): void {
  pendingDelete.value = goal;
  deleteChildCount.value = (tree.value.childrenOf.get(goal.id) ?? []).length;
}

async function confirmDelete(): Promise<void> {
  const goal = pendingDelete.value;

  if (!goal) {
    return;
  }

  const cascade = deleteChildCount.value > 0;
  const deleted = await store.removeGoal(goal.id, cascade);

  if (deleted !== null) {
    pendingDelete.value = null;
  }
}
</script>

<template>
  <section class="panel">
    <header class="panel-head">
      <div>
        <h2 class="panel-title">项目目标</h2>
        <p class="panel-sub">主目标承接项目意图，子目标把大目标拆成可衡量的几块。</p>
      </div>

      <button class="create-btn" type="button" @click="openCreate()">
        <span aria-hidden="true" class="plus">+</span>
        新建目标
      </button>
    </header>

    <p v-if="store.goalsLoading" class="hint">正在加载目标…</p>

    <p v-else-if="store.goals.length === 0" class="empty-box">
      还没有目标。先写下一个主目标，再拆出 2~3 个子目标。
    </p>

    <div v-else class="goal-tree">
      <template v-for="(root, index) in tree.roots" :key="root.id">
        <article class="goal-card" :class="{ 'goal-card--main': index === 0 }">
          <div class="goal-line">
            <div class="goal-title-group">
              <span class="main-tag">{{ index === 0 ? "主目标" : "独立目标" }}</span>
              <h3 class="goal-title">{{ root.title }}</h3>
            </div>

            <PlanStatusPill :label="GOAL_STATUS_LABELS[root.status]" :status="root.status"/>
          </div>

          <p v-if="root.description" class="goal-desc">{{ root.description }}</p>

          <div class="goal-meta">
            <span class="meta-chip">{{ GOAL_TYPE_LABELS[root.goal_type] }}</span>
            <span class="meta-text">目标时间：{{ formatDate(root.target_at) }}</span>
          </div>

          <div class="goal-actions">
            <button
              v-for="target in goalStatusTargets(root.status)"
              :key="target"
              class="ghost-btn"
              type="button"
              @click="applyStatus(root, target)"
            >
              {{ STATUS_TARGET_LABELS[target] }}
            </button>

            <button class="ghost-btn" type="button" @click="openLinks(root)">关联阶段</button>
            <button class="ghost-btn" type="button" @click="openEdit(root)">编辑</button>
            <button class="ghost-btn ghost-btn--danger" type="button" @click="askDelete(root)">
              删除
            </button>
          </div>
        </article>

        <article
          v-for="child in tree.childrenOf.get(root.id) ?? []"
          :key="child.id"
          class="goal-card"
        >
          <div class="goal-line">
            <h3 class="goal-title goal-title--child">{{ child.title }}</h3>
            <PlanStatusPill :label="GOAL_STATUS_LABELS[child.status]" :status="child.status"/>
          </div>

          <p v-if="child.description" class="goal-desc">{{ child.description }}</p>

          <div class="goal-meta">
            <span class="meta-chip">{{ GOAL_TYPE_LABELS[child.goal_type] }}</span>
            <span class="meta-text">目标时间：{{ formatDate(child.target_at) }}</span>
          </div>

          <div class="goal-actions">
            <button
              v-for="target in goalStatusTargets(child.status)"
              :key="target"
              class="ghost-btn"
              type="button"
              @click="applyStatus(child, target)"
            >
              {{ STATUS_TARGET_LABELS[target] }}
            </button>

            <button class="ghost-btn" type="button" @click="openLinks(child)">关联阶段</button>
            <button class="ghost-btn" type="button" @click="openEdit(child)">编辑</button>
            <button class="ghost-btn ghost-btn--danger" type="button" @click="askDelete(child)">
              删除
            </button>
          </div>
        </article>
      </template>
    </div>

    <GoalFormDialog
      :goal="editing"
      :open="formOpen"
      :parent-options="parentOptions"
      @close="formOpen = false"
      @saved="formOpen = false"
    />

    <GoalStagesDialog
      :goal="linkingGoal"
      :open="linksOpen"
      :stages="store.stages"
      @close="linksOpen = false"
      @saved="linksOpen = false"
    />

    <ModalDialog
      :open="pendingDelete !== null"
      title="删除目标"
      @close="pendingDelete = null"
    >
      <div class="pf-form">
        <p class="confirm-text">
          将删除目标「{{ pendingDelete?.title }}」。
          <template v-if="deleteChildCount > 0">
            它还有 <strong>{{ deleteChildCount }}</strong> 个子目标，会一并删除。
          </template>
        </p>

        <p v-if="pendingDelete?.status === 'achieved'" class="pf-error">
          已达成的目标不允许删除：达成是成果事实，删除后不可追溯。如需清理展示，请改为已取消。
        </p>

        <div class="pf-actions">
          <button class="pf-btn pf-btn--ghost" type="button" @click="pendingDelete = null">
            取消
          </button>
          <button
            :disabled="pendingDelete?.status === 'achieved'"
            class="pf-btn pf-btn--danger"
            type="button"
            @click="confirmDelete()"
          >
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
  margin: 0;
  color: #202b40;
  font-size: 17px;
  font-weight: 650;
}

.panel-sub {
  margin: 6px 0 0;
  color: #8490a3;
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
  border-radius: 9px;

  background: #4f46e5;
  color: #fff;

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease;
}

.create-btn:hover {
  background: #4338ca;
}

.plus {
  font-size: 17px;
  line-height: 1;
}

.hint {
  margin: 0;
  color: #98a1b2;
  font-size: 13px;
  line-height: 1.7;
}

.empty-box {
  padding: 34px 20px;
  margin: 0;

  border: 1px dashed #dfe4ee;
  border-radius: 12px;

  color: #8a94a6;
  font-size: 13px;
  text-align: center;
}

.goal-tree {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.goal-card {
  display: flex;
  flex-direction: column;
  gap: 10px;

  padding: 16px 18px;

  border: 1px solid #e9edf4;
  border-radius: 12px;

  background: #fff;
}

.goal-card--main {
  border-color: #dfe3fb;
  background: linear-gradient(180deg, #fbfbff, #fff);
}

.goal-line {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.goal-title-group {
  display: flex;
  align-items: center;
  gap: 9px;
  min-width: 0;
}

.main-tag {
  flex-shrink: 0;

  padding: 3px 8px;

  border-radius: 5px;

  background: #eef1fe;
  color: #4f46e5;

  font-size: 11px;
  font-weight: 700;
}

.goal-title {
  margin: 0;
  color: #263248;
  font-size: 15px;
  font-weight: 600;
  line-height: 1.5;
}

.goal-title--child {
  padding-left: 14px;
  border-left: 2px solid #e4e8f2;
  font-size: 14px;
}

.goal-desc {
  margin: 0;
  color: #6f7a8d;
  font-size: 13px;
  line-height: 1.7;
  white-space: pre-wrap;
}

.goal-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.meta-chip {
  padding: 3px 8px;

  border-radius: 5px;

  background: #f2f4f9;
  color: #6b7385;

  font-size: 11px;
  font-weight: 600;
}

.meta-text {
  color: #98a1b2;
  font-size: 12px;
}

.goal-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;

  padding-top: 10px;
  border-top: 1px solid #f1f4f9;
}

.ghost-btn {
  padding: 6px 11px;

  border: 1px solid #dce2eb;
  border-radius: 7px;

  background: #fff;
  color: #596579;

  font-size: 12px;
  cursor: pointer;

  transition: background-color 0.18s ease, border-color 0.18s ease;
}

.ghost-btn:hover {
  border-color: #c5ccda;
  background: #f8f9fc;
}

.ghost-btn--danger {
  border-color: #f0dada;
  color: #c04141;
}

.ghost-btn--danger:hover {
  border-color: #e8c3c3;
  background: #fff6f6;
}

.confirm-text {
  margin: 0;
  color: #364152;
  font-size: 14px;
  line-height: 1.7;
}
</style>
