import type {
  ActivityType,
  EvidenceType,
  SessionContributionType,
  SessionSource,
} from "../../api/sessions";

// 文案取自 ERD §5.3 与契约 §2.2 / ADR-010
export const ACTIVITY_TYPE_LABELS: Record<ActivityType, string> = {
  theory: "理论学习",
  practice: "练习实践",
  review: "复习",
  mixed: "混合",
  other: "其他",
};

export const SESSION_SOURCE_LABELS: Record<SessionSource, string> = {
  user: "手动记录",
  timer: "计时器",
  imported: "迁移导入",
};

export const CONTRIBUTION_TYPE_LABELS: Record<SessionContributionType, string> = {
  primary: "主要内容",
  covered: "覆盖到",
  practiced: "已练习",
};

export const EVIDENCE_TYPE_LABELS: Record<EvidenceType, string> = {
  note: "笔记",
  link: "链接",
  test_result: "测试结果",
};

export const ACTIVITY_TYPE_OPTIONS: { value: ActivityType; label: string }[] = [
  { value: "theory", label: ACTIVITY_TYPE_LABELS.theory },
  { value: "practice", label: ACTIVITY_TYPE_LABELS.practice },
  { value: "review", label: ACTIVITY_TYPE_LABELS.review },
  { value: "mixed", label: ACTIVITY_TYPE_LABELS.mixed },
  { value: "other", label: ACTIVITY_TYPE_LABELS.other },
];

export const ACTIVITY_TYPE_FILTER_OPTIONS: { value: ActivityType | "all"; label: string }[] = [
  { value: "all", label: "全部类型" },
  ...ACTIVITY_TYPE_OPTIONS,
];

export const CONTRIBUTION_TYPE_OPTIONS: {
  value: SessionContributionType;
  label: string;
}[] = [
  { value: "primary", label: CONTRIBUTION_TYPE_LABELS.primary },
  { value: "covered", label: CONTRIBUTION_TYPE_LABELS.covered },
  { value: "practiced", label: CONTRIBUTION_TYPE_LABELS.practiced },
];

export const EVIDENCE_TYPE_OPTIONS: { value: EvidenceType; label: string }[] = [
  { value: "note", label: EVIDENCE_TYPE_LABELS.note },
  { value: "link", label: EVIDENCE_TYPE_LABELS.link },
  { value: "test_result", label: EVIDENCE_TYPE_LABELS.test_result },
];

export type PillTone = "neutral" | "primary" | "success" | "warning" | "danger";

const ACTIVITY_TYPE_TONES: Record<ActivityType, PillTone> = {
  theory: "primary",
  practice: "success",
  review: "warning",
  mixed: "neutral",
  other: "neutral",
};

export function activityTone(type: ActivityType): PillTone {
  return ACTIVITY_TYPE_TONES[type];
}

// ---------------------------------------------------------------------------
// 时间与时长
// ---------------------------------------------------------------------------

/** 区间分钟数（向下取整）；与后端 `min(上报, 区间)` 的口径一致 */
export function intervalMinutes(startedAt: string, endedAt: string): number | null {
  const start = new Date(startedAt).getTime();
  const end = new Date(endedAt).getTime();

  if (Number.isNaN(start) || Number.isNaN(end) || end < start) {
    return null;
  }

  return Math.floor((end - start) / 60_000);
}

/** 「3 小时 20 分钟」/「45 分钟」——有效时长是分钟数，不折算成小数小时 */
export function formatMinutes(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} 分钟`;
  }

  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;

  return rest === 0 ? `${hours} 小时` : `${hours} 小时 ${rest} 分钟`;
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

function shortStamp(date: Date): string {
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** 同一天只写一次日期；跨午夜两侧都写，避免「23:30 → 00:30」看起来像倒流 */
export function formatTimeRange(startedAt: string, endedAt: string): string {
  const start = new Date(startedAt);
  const end = new Date(endedAt);

  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) {
    return "—";
  }

  const sameDay =
    start.getFullYear() === end.getFullYear() &&
    start.getMonth() === end.getMonth() &&
    start.getDate() === end.getDate();

  if (sameDay) {
    return `${shortStamp(start)} → ${pad(end.getHours())}:${pad(end.getMinutes())}`;
  }

  return `${shortStamp(start)} → ${shortStamp(end)}`;
}

/** 计时器展示用：HH:MM:SS */
export function formatElapsed(milliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
}
