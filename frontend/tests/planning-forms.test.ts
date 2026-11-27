import {describe, expect, it} from "vitest";
import type {GoalItem, MilestoneItem, StageItem, TaskItem} from "../src/api/planning.ts";
import {
  buildGoalCreatePayload,
  buildGoalUpdatePayload,
  buildMilestoneUpdatePayload,
  buildStageCreatePayload,
  buildStageUpdatePayload,
  buildTaskCreatePayload,
  buildTaskUpdatePayload,
  emptyGoalForm,
  emptyStageForm,
  emptyTaskForm,
  goalToForm,
  taskToForm,
} from "../src/views/planning/planning-forms.ts";

function goal(overrides: Partial<GoalItem> = {}): GoalItem {
  return {
    id: "g-1",
    project_id: "p-1",
    parent_goal_id: null,
    title: "主目标",
    description: "原描述",
    goal_type: "outcome",
    status: "active",
    target_at: "2027-06-01T02:00:00Z",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function stage(overrides: Partial<StageItem> = {}): StageItem {
  return {
    id: "s-1",
    roadmap_id: "r-1",
    name: "基础",
    description: null,
    sequence_no: 1,
    status: "pending",
    completion_method: "user",
    started_at: null,
    completed_at: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function task(overrides: Partial<TaskItem> = {}): TaskItem {
  return {
    id: "t-1",
    project_id: "p-1",
    milestone_id: null,
    stage_id: "s-1",
    title: "做题",
    description: null,
    status: "todo",
    priority: 3,
    due_at: "2027-06-01T02:00:00Z",
    estimated_minutes: 45,
    completed_at: null,
    created_source: "user",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

describe("目标表单载荷", () => {
  it("标题为空时拒绝提交", () => {
    const built = buildGoalCreatePayload(emptyGoalForm());

    expect(built.ok).toBe(false);
  });

  it("创建只带填写过的可选字段", () => {
    const built = buildGoalCreatePayload({
      ...emptyGoalForm(),
      title: "  数学二 110 分  ",
      goalType: "performance",
      targetAt: "2027-06-01T10:00",
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.title).toBe("数学二 110 分");
      expect(built.payload.goal_type).toBe("performance");
      expect(built.payload.description).toBeUndefined();
      expect(built.payload.parent_goal_id).toBeUndefined();
      expect(built.payload.target_at).toBe(new Date("2027-06-01T10:00").toISOString());
    }
  });

  it("原样打开再保存不会产生修改", () => {
    const built = buildGoalUpdatePayload(goal(), goalToForm(goal()));

    expect(built.ok).toBe(false);
  });

  it("清空描述显式提交 null，而不是省略字段", () => {
    const built = buildGoalUpdatePayload(goal(), {...goalToForm(goal()), description: "  "});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.description).toBeNull();
    }
  });

  it("子目标提升为主目标时提交 null", () => {
    const child = goal({parent_goal_id: "g-0", title: "子目标"});
    const built = buildGoalUpdatePayload(child, {...goalToForm(child), parentGoalId: ""});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.parent_goal_id).toBeNull();
    }
  });
});

describe("阶段表单载荷", () => {
  it("顺序号留空时不下发该字段，由服务端追加到末尾", () => {
    const built = buildStageCreatePayload({...emptyStageForm(), name: "基础"});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.sequence_no).toBeUndefined();
    }
  });

  it("非法顺序号被拒绝", () => {
    expect(buildStageCreatePayload({...emptyStageForm(), name: "基础", sequenceNo: "0"}).ok).toBe(
      false,
    );
    expect(
      buildStageCreatePayload({...emptyStageForm(), name: "基础", sequenceNo: "1.5"}).ok,
    ).toBe(false);
  });

  it("更新只提交变化字段", () => {
    const current = stage({description: "旧描述"});
    const built = buildStageUpdatePayload(current, {
      name: "基础",
      description: "新描述",
      sequenceNo: "1",
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({description: "新描述"});
    }
  });
});

describe("里程碑表单载荷", () => {
  function milestone(): MilestoneItem {
    return {
      id: "m-1",
      stage_id: "s-1",
      title: "极限完成",
      description: null,
      status: "pending",
      completed_at: null,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };
  }

  it("无变化时拒绝提交", () => {
    const built = buildMilestoneUpdatePayload(milestone(), {title: "极限完成", description: ""});

    expect(built.ok).toBe(false);
  });

  it("补上描述时只下发描述", () => {
    const built = buildMilestoneUpdatePayload(milestone(), {
      title: "极限完成",
      description: "做完 30 道题",
    });

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload).toEqual({description: "做完 30 道题"});
    }
  });
});

describe("任务表单载荷", () => {
  it("优先级越界被拒绝", () => {
    expect(buildTaskCreatePayload({...emptyTaskForm(), title: "做题", priority: "6"}).ok).toBe(
      false,
    );
  });

  it("预计时长必须是非负整数", () => {
    expect(
      buildTaskCreatePayload({...emptyTaskForm(), title: "做题", estimatedMinutes: "-5"}).ok,
    ).toBe(false);
    expect(
      buildTaskCreatePayload({...emptyTaskForm(), title: "做题", estimatedMinutes: "2.5"}).ok,
    ).toBe(false);
  });

  it("只挂项目时不带阶段与里程碑", () => {
    const built = buildTaskCreatePayload({...emptyTaskForm(), title: "做题"});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.stage_id).toBeUndefined();
      expect(built.payload.milestone_id).toBeUndefined();
      expect(built.payload.priority).toBe(3);
    }
  });

  it("原样保存不产生修改", () => {
    const built = buildTaskUpdatePayload(task(), taskToForm(task()));

    expect(built.ok).toBe(false);
  });

  it("清空截止时间提交 null", () => {
    const built = buildTaskUpdatePayload(task(), {...taskToForm(task()), dueAt: ""});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.due_at).toBeNull();
    }
  });

  it("解除阶段归属提交 null", () => {
    const built = buildTaskUpdatePayload(task(), {...taskToForm(task()), stageId: ""});

    expect(built.ok).toBe(true);

    if (built.ok) {
      expect(built.payload.stage_id).toBeNull();
    }
  });
});
