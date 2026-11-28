import type {
  BodyFormat,
  ContentCreatedSource,
  ContentStatus,
  ContentType,
  ProjectContentStatus,
  RelationDirection,
  RelationItem,
  RelationType,
} from "../../api/contents";

// 文案取自 ERD §5.1 / §5.2 枚举注释与契约 §2.2
export const CONTENT_TYPE_LABELS: Record<ContentType, string> = {
  knowledge_point: "知识点",
  skill_point: "技能点",
  language_item: "语言项",
};

export const CONTENT_STATUS_LABELS: Record<ContentStatus, string> = {
  active: "有效",
  archived: "已归档",
};

export const BODY_FORMAT_LABELS: Record<BodyFormat, string> = {
  markdown: "Markdown",
  plain_text: "纯文本",
};

export const CONTENT_SOURCE_LABELS: Record<ContentCreatedSource, string> = {
  user: "手动创建",
  ai_confirmed: "AI 确认",
  template: "模板",
};

export const RELATION_TYPE_LABELS: Record<RelationType, string> = {
  prerequisite: "前置",
  related: "相关",
  contains: "包含",
  extends: "延伸",
  alternative: "替代",
};

/** 关系单向存储、双向查询（D-25）：同一条边从两端读出来的说法必须相反 */
const RELATION_DIRECTION_LABELS: Record<RelationType, [string, string]> = {
  prerequisite: ["前置", "后置"],
  related: ["相关", "相关"],
  contains: ["包含", "属于"],
  extends: ["延伸", "被延伸"],
  alternative: ["替代", "被替代"],
};

export function relationLabel(type: RelationType, direction: RelationDirection): string {
  const labels = RELATION_DIRECTION_LABELS[type];

  return direction === "outbound" ? labels[0] : labels[1];
}

/** 关系的另一端：出边看 target，入边看 source */
export function relationPeerId(relation: RelationItem, currentId: string): string {
  return relation.source_content_id === currentId
    ? relation.target_content_id
    : relation.source_content_id;
}

export const PROJECT_STATUS_LABELS: Record<ProjectContentStatus, string> = {
  not_started: "未开始",
  learning: "学习中",
  reviewing: "复习中",
  achieved: "已达成",
  waived: "已豁免",
};

export const MASTERY_LEVEL_LABELS: Record<number, string> = {
  0: "L0 未评估",
  1: "L1 了解",
  2: "L2 理解",
  3: "L3 应用",
  4: "L4 精通",
};

export const CONTENT_PRIORITY_LABELS: Record<number, string> = {
  1: "P1 最低",
  2: "P2 较低",
  3: "P3 普通",
  4: "P4 较高",
  5: "P5 最高",
};

// ---------------------------------------------------------------------------
// 选项源：显式声明值类型，避免模板里对 Object.entries 做断言
// ---------------------------------------------------------------------------

export const CONTENT_TYPE_OPTIONS: { value: ContentType; label: string }[] = [
  { value: "knowledge_point", label: CONTENT_TYPE_LABELS.knowledge_point },
  { value: "skill_point", label: CONTENT_TYPE_LABELS.skill_point },
  { value: "language_item", label: CONTENT_TYPE_LABELS.language_item },
];

export const CONTENT_TYPE_FILTER_OPTIONS: { value: ContentType | "all"; label: string }[] = [
  { value: "all", label: "全部类型" },
  ...CONTENT_TYPE_OPTIONS,
];

export const CONTENT_STATUS_FILTER_OPTIONS: {
  value: ContentStatus | "all";
  label: string;
}[] = [
  { value: "active", label: CONTENT_STATUS_LABELS.active },
  { value: "archived", label: CONTENT_STATUS_LABELS.archived },
  { value: "all", label: "全部状态" },
];

export const BODY_FORMAT_OPTIONS: { value: BodyFormat; label: string }[] = [
  { value: "markdown", label: BODY_FORMAT_LABELS.markdown },
  { value: "plain_text", label: BODY_FORMAT_LABELS.plain_text },
];

export const RELATION_TYPE_OPTIONS: { value: RelationType; label: string }[] = [
  { value: "prerequisite", label: RELATION_TYPE_LABELS.prerequisite },
  { value: "related", label: RELATION_TYPE_LABELS.related },
  { value: "contains", label: RELATION_TYPE_LABELS.contains },
  { value: "extends", label: RELATION_TYPE_LABELS.extends },
  { value: "alternative", label: RELATION_TYPE_LABELS.alternative },
];

/** 可手工设置的项目内状态：`achieved` 由 M06 掌握度评价写入（D-27），不作为选项 */
export const PROJECT_STATUS_OPTIONS: {
  value: Exclude<ProjectContentStatus, "achieved">;
  label: string;
}[] = [
  { value: "not_started", label: PROJECT_STATUS_LABELS.not_started },
  { value: "learning", label: PROJECT_STATUS_LABELS.learning },
  { value: "reviewing", label: PROJECT_STATUS_LABELS.reviewing },
  { value: "waived", label: PROJECT_STATUS_LABELS.waived },
];

export const MASTERY_LEVEL_OPTIONS: { value: string; label: string }[] = [0, 1, 2, 3, 4].map(
  (level) => ({ value: String(level), label: MASTERY_LEVEL_LABELS[level] }),
);

export const CONTENT_PRIORITY_OPTIONS: { value: string; label: string }[] = [5, 4, 3, 2, 1].map(
  (level) => ({ value: String(level), label: CONTENT_PRIORITY_LABELS[level] }),
);

// ---------------------------------------------------------------------------
// 状态配色与提示
// ---------------------------------------------------------------------------

export type PillTone = "neutral" | "primary" | "success" | "warning" | "danger";

export function contentTone(status: ContentStatus): PillTone {
  return status === "active" ? "primary" : "neutral";
}

const PROJECT_STATUS_TONES: Record<ProjectContentStatus, PillTone> = {
  not_started: "neutral",
  learning: "primary",
  reviewing: "warning",
  achieved: "success",
  waived: "neutral",
};

export function projectStatusTone(status: ProjectContentStatus): PillTone {
  return PROJECT_STATUS_TONES[status];
}

/** 相似度按百分比呈现：`pg_trgm` 给的是 0–1 的相似度，0.4 是提示阈值（D-24） */
export function formatSimilarity(value: number): string {
  return `${Math.round(value * 100)}%`;
}

/**
 * 归档内容不能加入项目（后端 409）。前端先给出原因，而不是让用户点完才失败。
 * 归档是终态——契约未提供恢复端点，因此这里不给「恢复」引导。
 */
export function attachBlockedReason(status: ContentStatus): string | null {
  return status === "archived" ? "已归档的内容不能加入项目" : null;
}
