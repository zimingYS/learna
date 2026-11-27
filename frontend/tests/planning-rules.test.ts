import {describe, expect, it} from "vitest";
import {
  canAdvanceStage,
  canConfirm,
  canDeleteStructure,
  canEditMilestone,
  canEditStructure,
  dueLabel,
  goalStatusTargets,
  groupGoals,
  milestoneStatusTargets,
  stageActions,
  statusTone,
  taskSortWeight,
  taskStatusTargets,
} from "../src/views/planning/planning-rules.ts";

describe("阶段四动作矩阵", () => {
  it("pending 只能开始或跳过，不能直接完成", () => {
    expect(stageActions("pending")).toEqual(["start", "skip"]);
    expect(stageActions("pending")).not.toContain("complete");
  });

  it("active 可完成或跳过，不再重复 start", () => {
    expect(stageActions("active")).toEqual(["complete", "skip"]);
  });

  it("completed 与 skipped 只能重新激活", () => {
    expect(stageActions("completed")).toEqual(["reactivate"]);
    expect(stageActions("skipped")).toEqual(["reactivate"]);
  });
});

describe("目标状态边（D-07b）", () => {
  it("active 可达成或取消", () => {
    expect(goalStatusTargets("active")).toEqual(["achieved", "cancelled"]);
  });

  it("achieved 只能回到 active，不能直接转 cancelled", () => {
    expect(goalStatusTargets("achieved")).toEqual(["active"]);
  });

  it("cancelled 只能回到 active", () => {
    expect(goalStatusTargets("cancelled")).toEqual(["active"]);
  });
});

describe("里程碑与任务的统一迁移规则（ADR-007）", () => {
  it("里程碑的终态之间不直转，但可回退到未完态", () => {
    expect(milestoneStatusTargets("completed")).toEqual(["pending", "in_progress"]);
    expect(milestoneStatusTargets("skipped")).toEqual(["pending", "in_progress"]);
  });

  it("里程碑从 pending 可直达任一未完态与 completed", () => {
    expect(milestoneStatusTargets("pending")).toEqual([
      "in_progress",
      "completed",
      "skipped",
    ]);
  });

  it("任务的终态是 completed / cancelled，两者之间必须回退", () => {
    expect(taskStatusTargets("completed")).toEqual(["todo", "in_progress", "blocked"]);
    expect(taskStatusTargets("cancelled")).toEqual(["todo", "in_progress", "blocked"]);
  });

  it("任务从 todo 可跳过中间态直达 completed", () => {
    expect(taskStatusTargets("todo")).toContain("completed");
    expect(taskStatusTargets("todo")).not.toContain("todo");
  });
});

describe("路线准入", () => {
  it("只有 draft 可编辑结构与确认", () => {
    expect(canEditStructure("draft")).toBe(true);
    expect(canEditStructure("active")).toBe(false);
    expect(canEditStructure("superseded")).toBe(false);

    expect(canConfirm("draft")).toBe(true);
    expect(canConfirm("active")).toBe(false);
  });

  it("只有 draft 可删除结构", () => {
    expect(canDeleteStructure("draft")).toBe(true);
    expect(canDeleteStructure("active")).toBe(false);
  });

  it("阶段动作只在 active 路线开放", () => {
    expect(canAdvanceStage("active")).toBe(true);
    expect(canAdvanceStage("draft")).toBe(false);
  });

  it("里程碑在 draft 与 active 都可增改，superseded 只读", () => {
    expect(canEditMilestone("draft")).toBe(true);
    expect(canEditMilestone("active")).toBe(true);
    expect(canEditMilestone("superseded")).toBe(false);
  });
});

describe("目标树分组（D-06 两层上限）", () => {
  const goals = [
    {id: "g1", title: "主目标", parent_goal_id: null},
    {id: "g2", title: "子目标 A", parent_goal_id: "g1"},
    {id: "g3", title: "子目标 B", parent_goal_id: "g1"},
  ];

  it("主目标归入 roots，子目标按父级分组", () => {
    const tree = groupGoals(goals);

    expect(tree.roots.map((goal) => goal.id)).toEqual(["g1"]);
    expect(tree.childrenOf.get("g1")?.map((goal) => goal.id)).toEqual(["g2", "g3"]);
  });

  it("空列表不会产生空分组", () => {
    const tree = groupGoals([]);

    expect(tree.roots).toEqual([]);
    expect(tree.childrenOf.size).toBe(0);
  });
});

describe("剩余天数文案", () => {
  function isoFromToday(offsetDays: number): string {
    const date = new Date();

    date.setDate(date.getDate() + offsetDays);

    return date.toISOString();
  }

  it("无截止返回 null", () => {
    expect(dueLabel(null)).toBeNull();
  });

  it("今天到期与逾期分别给出不同文案", () => {
    expect(dueLabel(isoFromToday(0))?.text).toBe("今天到期");
    expect(dueLabel(isoFromToday(-2))).toEqual({text: "已逾期 2 天", overdue: true});
  });

  it("未来日期给出剩余天数", () => {
    expect(dueLabel(isoFromToday(3))).toEqual({text: "剩 3 天", overdue: false});
  });
});

describe("展示辅助", () => {
  it("进行中的任务排在待办之前，终态靠后", () => {
    expect(taskSortWeight("in_progress")).toBeLessThan(taskSortWeight("todo"));
    expect(taskSortWeight("todo")).toBeLessThan(taskSortWeight("blocked"));
    expect(taskSortWeight("completed")).toBeGreaterThan(taskSortWeight("blocked"));
  });

  it("状态配色按语义归并，未知状态回落中性色", () => {
    expect(statusTone("active")).toBe("primary");
    expect(statusTone("in_progress")).toBe("primary");
    expect(statusTone("completed")).toBe("success");
    expect(statusTone("cancelled")).toBe("danger");
    expect(statusTone("unknown-value")).toBe("neutral");
  });
});
