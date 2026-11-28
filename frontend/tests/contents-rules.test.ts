import {describe, expect, it} from "vitest";
import type {RelationItem} from "../src/api/contents.ts";
import {
  attachBlockedReason,
  CONTENT_PRIORITY_OPTIONS,
  contentTone,
  formatSimilarity,
  MASTERY_LEVEL_OPTIONS,
  PROJECT_STATUS_OPTIONS,
  PROJECT_STATUS_LABELS,
  projectStatusTone,
  relationLabel,
  relationPeerId,
} from "../src/views/contents/contents-rules.ts";

const relation: RelationItem = {
  id: "r-1",
  source_content_id: "c-1",
  target_content_id: "c-2",
  relation_type: "prerequisite",
  direction: "outbound",
};

describe("关系方向读法（D-25 双向查询）", () => {
  it("关系的另一端由 id 决定，与查询方向无关", () => {
    expect(relationPeerId(relation, "c-1")).toBe("c-2");
    expect(relationPeerId({...relation, direction: "inbound"}, "c-2")).toBe("c-1");
  });

  it("同一条边从两端读出的说法相反", () => {
    expect(relationLabel("prerequisite", "outbound")).toBe("前置");
    expect(relationLabel("prerequisite", "inbound")).toBe("后置");
    expect(relationLabel("contains", "outbound")).toBe("包含");
    expect(relationLabel("contains", "inbound")).toBe("属于");
  });

  it("对称关系两端同一说法", () => {
    expect(relationLabel("related", "outbound")).toBe("相关");
    expect(relationLabel("related", "inbound")).toBe("相关");
  });
});

describe("状态徽标配色", () => {
  it("归档内容与未开始内容都走中性色", () => {
    expect(contentTone("active")).toBe("primary");
    expect(contentTone("archived")).toBe("neutral");
    expect(projectStatusTone("not_started")).toBe("neutral");
    expect(projectStatusTone("waived")).toBe("neutral");
  });

  it("学习中 / 复习中 / 已达成用不同色调区分", () => {
    expect(projectStatusTone("learning")).toBe("primary");
    expect(projectStatusTone("reviewing")).toBe("warning");
    expect(projectStatusTone("achieved")).toBe("success");
  });
});

describe("状态选项与提示", () => {
  it("可手工写回的状态不含 achieved（D-27）", () => {
    const values = PROJECT_STATUS_OPTIONS.map((option) => option.value);

    expect(values).not.toContain("achieved");
    expect(values).toHaveLength(Object.keys(PROJECT_STATUS_LABELS).length - 1);
  });

  it("目标等级 0–4、优先级 1–5 与后端范围一致", () => {
    expect(MASTERY_LEVEL_OPTIONS.map((option) => option.value)).toEqual([
      "0",
      "1",
      "2",
      "3",
      "4",
    ]);
    expect(CONTENT_PRIORITY_OPTIONS.map((option) => option.value)).toEqual([
      "5",
      "4",
      "3",
      "2",
      "1",
    ]);
  });

  it("归档内容加入项目前就给出原因", () => {
    expect(attachBlockedReason("archived")).toContain("已归档");
    expect(attachBlockedReason("active")).toBeNull();
  });

  it("相似度按百分比展示，0.4 阈值不会再被四舍五入吃掉", () => {
    expect(formatSimilarity(0.4)).toBe("40%");
    expect(formatSimilarity(0.6666)).toBe("67%");
  });
});
