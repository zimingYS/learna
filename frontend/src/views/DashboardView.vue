<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {RouterLink} from "vue-router";
import {describeError} from "../api/http";
import {fetchProjects, type ProjectItem} from "../api/projects";
import {fetchRecords, type RecordItem} from "../api/records";
import {useAuthStore} from "../stores/auth";
import {daysUntil, PROJECT_STATUS_LABELS, PROJECT_TYPE_LABELS} from "./projects/project-rules";

interface StatCard {
  label: string;
  value: string;
  unit?: string;
  caption: string;
  ratio: number | null;
  tone: "brand" | "amber" | "green" | "violet";
  paths: string[];
}

const authStore = useAuthStore();

const projects = ref<ProjectItem[]>([]);
const records = ref<RecordItem[]>([]);
const recordTotal = ref(0);
const loading = ref(true);
const error = ref("");

const displayName = computed(() => authStore.user?.display_name?.trim() || "Learner");

const greeting = computed(() => {
  const hour = new Date().getHours();

  if (hour < 6) {
    return "夜深了";
  }

  if (hour < 12) {
    return "早上好";
  }

  if (hour < 18) {
    return "下午好";
  }

  return "晚上好";
});

const WEEKDAYS = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

const now = new Date();

const dateText = `${now.getMonth() + 1} 月 ${now.getDate()} 日`;

const weekdayText = WEEKDAYS[now.getDay()];

/** 归档项目不参与推进类统计，只在总数里出现 */
const liveProjects = computed(() => projects.value.filter((item) => item.status !== "archived"));

const activeCount = computed(
  () => liveProjects.value.filter((item) => item.status === "active").length,
);

const completedCount = computed(
  () => liveProjects.value.filter((item) => item.status === "completed").length,
);

const dueSoonCount = computed(
  () =>
    liveProjects.value.filter((item) => {
      if (item.status !== "active" || item.deadline_at === null) {
        return false;
      }

      const days = daysUntil(item.deadline_at);

      return days !== null && days >= 0 && days <= 7;
    }).length,
);

const stats = computed<StatCard[]>(() => {
  const total = liveProjects.value.length || 1;

  return [
    {
      label: "进行中项目",
      value: String(activeCount.value),
      unit: "个",
      caption: `全部 ${liveProjects.value.length} 个学习项目`,
      ratio: activeCount.value / total,
      tone: "brand",
      paths: ["M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"],
    },
    {
      label: "7 天内截止",
      value: String(dueSoonCount.value),
      unit: "个",
      caption: dueSoonCount.value > 0 ? "记得安排推进节奏" : "暂无临近截止的项目",
      ratio: null,
      tone: "amber",
      paths: [
        "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z",
        "M8 3v4M16 3v4M4 10h16",
      ],
    },
    {
      label: "已完成项目",
      value: String(completedCount.value),
      unit: "个",
      caption: `完成率 ${Math.round((completedCount.value / total) * 100)}%`,
      ratio: completedCount.value / total,
      tone: "green",
      paths: ["m20 6-11 11-5-5"],
    },
    {
      label: "学习记录",
      value: String(recordTotal.value),
      unit: "条",
      caption: recordTotal.value > 0 ? "累计沉淀的学习记录" : "还没有记录，写下第一条吧",
      ratio: null,
      tone: "violet",
      paths: ["M4 19.5A2.5 2.5 0 0 1 6.5 17H20", "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z"],
    },
  ];
});

const featuredProjects = computed(() =>
  [...liveProjects.value]
    .filter((item) => item.status !== "completed")
    .sort((a, b) => b.importance_level - a.importance_level || b.updated_at.localeCompare(a.updated_at))
    .slice(0, 4),
);

const subline = computed(() => {
  if (loading.value) {
    return "正在整理你的学习看板…";
  }

  if (dueSoonCount.value > 0) {
    return `有 ${dueSoonCount.value} 个项目将在 7 天内截止，今天也推进一点吧。`;
  }

  return "今天也要加油学习哦！";
});

function projectDeadline(project: ProjectItem): string {
  if (project.deadline_at === null) {
    return "无截止日期";
  }

  const days = daysUntil(project.deadline_at);

  if (days === null) {
    return "无截止日期";
  }

  if (days < 0) {
    return `已逾期 ${Math.abs(days)} 天`;
  }

  return days === 0 ? "今天截止" : `距截止 ${days} 天`;
}

function isOverdue(project: ProjectItem): boolean {
  if (project.deadline_at === null) {
    return false;
  }

  const days = daysUntil(project.deadline_at);

  return days !== null && days < 0;
}

function recordTime(iso: string): string {
  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const pad = (value: number) => String(value).padStart(2, "0");

  return `${date.getMonth() + 1} 月 ${date.getDate()} 日 ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

async function load(): Promise<void> {
  loading.value = true;
  error.value = "";

  try {
    const [projectData, recordData] = await Promise.all([
      fetchProjects({page: 1, page_size: 100, include_archived: true}),
      fetchRecords(1, 5),
    ]);

    projects.value = projectData.items;
    records.value = recordData.items;
    recordTotal.value = recordData.total;
  } catch (e: unknown) {
    error.value = describeError(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>

<template>
  <main class="dashboard">
    <header class="greet">
      <div class="greet-text">
        <h1>{{ greeting }}，{{ displayName }}<span aria-hidden="true" class="wave">👋</span></h1>
        <p>{{ subline }}</p>
      </div>

      <div class="date-card">
        <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
             stroke-linejoin="round" stroke-width="1.7" viewBox="0 0 24 24">
          <path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z"/>
          <path d="M8 3v4M16 3v4M4 10h16"/>
        </svg>
        <div>
          <strong>{{ dateText }}</strong>
          <span>{{ weekdayText }}</span>
        </div>
      </div>
    </header>

    <p v-if="error" class="notice" role="alert">{{ error }}</p>

    <section class="stat-grid" aria-label="学习概览">
      <article v-for="stat in stats" :key="stat.label" class="stat-card">
        <div class="stat-head">
          <span :class="`stat-icon stat-icon--${stat.tone}`">
            <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
                 stroke-linejoin="round" stroke-width="1.8" viewBox="0 0 24 24">
              <path v-for="path in stat.paths" :key="path" :d="path"/>
            </svg>
          </span>
          <span class="stat-label">{{ stat.label }}</span>
        </div>

        <p class="stat-value">
          {{ stat.value }}<span v-if="stat.unit" class="stat-unit">{{ stat.unit }}</span>
        </p>

        <div v-if="stat.ratio !== null" class="stat-bar">
          <span :style="{ width: `${Math.round(stat.ratio * 100)}%` }"/>
        </div>

        <p class="stat-caption">{{ stat.caption }}</p>
      </article>
    </section>

    <div class="panel-grid">
      <section class="panel">
        <header class="panel-head">
          <div class="panel-title">
            <span class="panel-icon panel-icon--brand">
              <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
                   stroke-linejoin="round" stroke-width="1.8" viewBox="0 0 24 24">
                <path d="m3 10.5 9-7 9 7"/>
                <path d="M5 9.5V20h14V9.5"/>
              </svg>
            </span>
            <h2>我的项目</h2>
          </div>

          <RouterLink class="panel-link" to="/projects">管理全部 →</RouterLink>
        </header>

        <div v-if="loading" class="skeleton-list">
          <span v-for="index in 3" :key="index" class="skeleton-row"/>
        </div>

        <ul v-else-if="featuredProjects.length > 0" class="project-list">
          <li v-for="project in featuredProjects" :key="project.id">
            <RouterLink class="project-row" :to="`/projects/${project.id}`">
              <span class="row-name">{{ project.name }}</span>
              <span class="row-meta">
                <span class="row-chip">{{ PROJECT_TYPE_LABELS[project.project_type] }}</span>
                <span class="row-dot">·</span>
                <span>{{ PROJECT_STATUS_LABELS[project.status] }}</span>
                <span class="row-dot">·</span>
                <span :class="{ 'row-overdue': isOverdue(project) }">
                  {{ projectDeadline(project) }}
                </span>
              </span>
              <span class="row-weight">I{{ project.importance_level }}</span>
            </RouterLink>
          </li>
        </ul>

        <div v-else-if="!loading" class="panel-empty">
          <p>还没有在推进的项目。</p>
          <RouterLink class="empty-btn" to="/projects">去创建项目</RouterLink>
        </div>
      </section>

      <section class="panel">
        <header class="panel-head">
          <div class="panel-title">
            <span class="panel-icon panel-icon--violet">
              <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
                   stroke-linejoin="round" stroke-width="1.8" viewBox="0 0 24 24">
                <path d="M12 7v5l3 2"/>
                <circle cx="12" cy="12" r="9"/>
              </svg>
            </span>
            <h2>近期学习</h2>
          </div>

          <RouterLink class="panel-link" to="/records">全部 →</RouterLink>
        </header>

        <div v-if="loading" class="skeleton-list">
          <span v-for="index in 3" :key="index" class="skeleton-row"/>
        </div>

        <ul v-else-if="records.length > 0" class="record-list">
          <li v-for="record in records" :key="record.id" class="record-row">
            <span aria-hidden="true" class="record-dot"/>

            <div class="record-body">
              <p class="record-title">{{ record.title }}</p>
              <p class="record-meta">
                <span v-if="record.subject" class="row-chip">{{ record.subject }}</span>
                <span v-if="record.duration_minutes">{{ record.duration_minutes }} 分钟</span>
                <span>{{ recordTime(record.created_at) }}</span>
              </p>
            </div>
          </li>
        </ul>

        <div v-else-if="!loading" class="panel-empty">
          <p>还没有学习记录。</p>
          <RouterLink class="empty-btn" to="/records">写一条记录</RouterLink>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
  color: var(--ink-1);
}

/* ==============================
   问候
============================== */

.greet {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;

  margin-bottom: 22px;
}

.greet-text h1 {
  margin: 0;

  color: var(--ink-1);
  font-size: 27px;
  font-weight: 720;
  letter-spacing: -0.7px;
}

.wave {
  margin-left: 6px;
  font-size: 22px;
}

.greet-text p {
  margin: 8px 0 0;
  color: var(--ink-3);
  font-size: 13.5px;
}

.date-card {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 10px;

  padding: 10px 16px;

  border: 1px solid var(--line);
  border-radius: var(--radius-md);

  background: rgb(255 255 255 / 78%);
}

.date-card svg {
  width: 18px;
  height: 18px;
  color: var(--ink-3);
}

.date-card strong {
  display: block;

  color: var(--ink-1);
  font-size: 13px;
  font-weight: 650;
}

.date-card span {
  color: var(--ink-4);
  font-size: 11.5px;
}

.notice {
  padding: 10px 14px;
  margin: 0 0 16px;

  border: 1px solid #f7dada;
  border-radius: var(--radius-sm);

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 13px;
}

/* ==============================
   指标卡
============================== */

.stat-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(4, 1fr);

  margin-bottom: 16px;
}

.stat-card {
  display: flex;
  flex-direction: column;

  padding: 18px 18px 16px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: var(--surface);
  box-shadow: var(--shadow-sm);
}

.stat-head {
  display: flex;
  align-items: center;
  gap: 9px;
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 28px;
  height: 28px;

  border-radius: 9px;
}

.stat-icon svg {
  width: 16px;
  height: 16px;
}

.stat-icon--brand {
  background: var(--brand-soft);
  color: var(--brand);
}

.stat-icon--amber {
  background: var(--amber-soft);
  color: var(--amber);
}

.stat-icon--green {
  background: var(--green-soft);
  color: var(--green);
}

.stat-icon--violet {
  background: var(--violet-soft);
  color: var(--violet);
}

.stat-label {
  color: var(--ink-3);
  font-size: 12.5px;
  font-weight: 500;
}

.stat-value {
  margin: 14px 0 10px;

  color: var(--ink-1);
  font-size: 27px;
  font-weight: 720;
  letter-spacing: -0.8px;
  line-height: 1;
}

.stat-unit {
  margin-left: 4px;

  color: var(--ink-4);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0;
}

.stat-bar {
  overflow: hidden;

  height: 6px;
  margin-bottom: 9px;

  border-radius: var(--radius-pill);

  background: var(--surface-sunken);
}

.stat-bar span {
  display: block;
  height: 100%;

  border-radius: var(--radius-pill);

  background: linear-gradient(90deg, #7d7cf8, var(--brand));
}

.stat-caption {
  margin: auto 0 0;
  padding-top: 9px;
  color: var(--ink-4);
  font-size: 11.5px;
}

/* ==============================
   面板
============================== */

.panel-grid {
  display: grid;
  align-items: start;
  gap: 16px;
  grid-template-columns: 1.45fr 1fr;
}

.panel {
  padding: 18px 20px 20px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: var(--surface);
  box-shadow: var(--shadow-sm);
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  margin-bottom: 14px;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 9px;
}

.panel-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 26px;
  height: 26px;

  border-radius: 8px;
}

.panel-icon svg {
  width: 15px;
  height: 15px;
}

.panel-icon--brand {
  background: var(--brand-soft);
  color: var(--brand);
}

.panel-icon--violet {
  background: var(--violet-soft);
  color: var(--violet);
}

.panel-title h2 {
  margin: 0;

  color: var(--ink-1);
  font-size: 14.5px;
  font-weight: 680;
  letter-spacing: -0.2px;
}

.panel-link {
  color: var(--ink-4);
  font-size: 12px;
  font-weight: 500;
  text-decoration: none;

  transition: color 0.18s var(--ease);
}

.panel-link:hover {
  color: var(--brand);
}

/* ==============================
   项目列表
============================== */

.project-list,
.record-list {
  padding: 0;
  margin: 0;
  list-style: none;
}

.project-list li + li {
  border-top: 1px solid var(--line);
}

.project-row {
  display: grid;
  align-items: center;
  gap: 4px 12px;
  padding: 12px 8px;
  margin: 0 -8px;

  border-radius: var(--radius-sm);

  color: inherit;
  grid-template-columns: 1fr auto;
  text-decoration: none;

  transition: background-color 0.18s var(--ease);
}

.project-row:hover {
  background: var(--surface-muted);
}

.row-name {
  overflow: hidden;

  color: var(--ink-1);
  font-size: 13.5px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;

  grid-column: 1;
  color: var(--ink-4);
  font-size: 11.5px;
}

.row-chip {
  padding: 2px 7px;

  border-radius: 6px;

  background: var(--brand-soft);
  color: var(--brand);

  font-size: 11px;
  font-weight: 600;
}

.row-dot {
  color: var(--ink-4);
}

.row-overdue {
  color: var(--rose);
  font-weight: 600;
}

.row-weight {
  grid-row: 1 / span 2;
  grid-column: 2;

  padding: 3px 8px;

  border-radius: var(--radius-pill);

  background: var(--violet-soft);
  color: #6d4bd8;

  font-size: 11px;
  font-weight: 700;
}

/* ==============================
   记录列表
============================== */

.record-row {
  display: flex;
  gap: 10px;

  padding: 11px 0;
}

.record-row + .record-row {
  border-top: 1px solid var(--line);
}

.record-dot {
  flex-shrink: 0;

  width: 7px;
  height: 7px;
  margin-top: 6px;

  border-radius: 50%;

  background: var(--green);
  box-shadow: 0 0 0 3px var(--green-soft);
}

.record-body {
  min-width: 0;
}

.record-title {
  overflow: hidden;

  margin: 0;

  color: var(--ink-1);
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.record-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;

  margin: 5px 0 0;

  color: var(--ink-4);
  font-size: 11.5px;
}

/* ==============================
   空态与骨架
============================== */

.panel-empty {
  padding: 26px 0 10px;
  text-align: center;
}

.panel-empty p {
  margin: 0 0 12px;
  color: var(--ink-3);
  font-size: 13px;
}

.empty-btn {
  display: inline-flex;
  align-items: center;

  height: 32px;
  padding: 0 16px;

  border-radius: var(--radius-pill);

  background: var(--brand-soft);
  color: var(--brand);

  font-size: 12.5px;
  font-weight: 600;
  text-decoration: none;
}

.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.skeleton-row {
  height: 40px;

  border-radius: var(--radius-sm);

  background: linear-gradient(90deg, var(--surface-muted) 0%, #fff 50%, var(--surface-muted) 100%);
  background-size: 200% 100%;

  animation: shimmer 1.4s ease-in-out infinite;
}

@keyframes shimmer {
  to {
    background-position: -200% 0;
  }
}

/* ==============================
   响应式
============================== */

@media (max-width: 1100px) {
  .stat-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .greet {
    flex-direction: column;
  }

  .date-card {
    display: none;
  }

  .stat-grid {
    grid-template-columns: 1fr;
  }
}
</style>
