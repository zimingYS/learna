import {beforeEach, describe, expect, it, vi} from "vitest";
import {
  applyStageAction,
  confirmRoadmap,
  createDependency,
  deleteDependency,
  deleteGoal,
  fetchGoalStages,
  fetchMilestones,
  fetchStages,
  fetchTasks,
  replaceGoalStages,
  updateGoal,
  updateMilestone,
  updateRoadmap,
  updateStage,
} from "../src/api/planning.ts";

const {requestMock} = vi.hoisted(() => ({requestMock: vi.fn()}));

vi.mock("../src/api/http.ts", () => ({
  describeError: vi.fn(),
  request: requestMock,
}));

beforeEach(() => {
  requestMock.mockReset();
  requestMock.mockResolvedValue({items: [], total: 0});
});

describe("规划域端点契约", () => {
  it("目标列表带分页参数，删除主目标可带 cascade", async () => {
    await fetchGoalStages("p-1", "g-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/goals/g-1/stages",
      method: "GET",
    });

    await deleteGoal("p-1", "g-1", true);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/goals/g-1",
      method: "DELETE",
      params: {cascade: true},
    });

    await deleteGoal("p-1", "g-2", false);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/goals/g-2",
      method: "DELETE",
      params: undefined,
    });
  });

  it("目标更新走 PATCH，阶段关联走整体替换 PUT", async () => {
    await updateGoal("p-1", "g-1", {status: "achieved"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/goals/g-1",
      method: "PATCH",
      data: {status: "achieved"},
    });

    const links = [{stage_id: "s-1", contribution: "primary" as const}];
    await replaceGoalStages("p-1", "g-1", links);
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/goals/g-1/stages",
      method: "PUT",
      data: {links},
    });
  });

  it("路线确认是独立动作端点，更新只改变更说明", async () => {
    await confirmRoadmap("p-1", "r-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/roadmaps/r-1/confirm",
      method: "POST",
    });

    await updateRoadmap("p-1", "r-1", {change_summary: null});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/roadmaps/r-1",
      method: "PATCH",
      data: {change_summary: null},
    });
  });

  it("阶段列表与更新都带 roadmap 段", async () => {
    await fetchStages("p-1", "r-1");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/roadmaps/r-1/stages",
      method: "GET",
    });

    await updateStage("p-1", "r-1", "s-1", {sequence_no: 2});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/roadmaps/r-1/stages/s-1",
      method: "PATCH",
      data: {sequence_no: 2},
    });
  });

  it("阶段四动作共用动作路径", async () => {
    for (const action of ["start", "complete", "skip", "reactivate"] as const) {
      await applyStageAction("p-1", "r-1", "s-1", action);

      expect(requestMock).toHaveBeenLastCalledWith({
        url: `/projects/p-1/roadmaps/r-1/stages/s-1/${action}`,
        method: "POST",
      });
    }
  });

  it("里程碑按阶段过滤，状态变更走 PATCH", async () => {
    await fetchMilestones("p-1", {page: 1, page_size: 100, stage_id: "s-1"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/milestones",
      method: "GET",
      params: {page: 1, page_size: 100, stage_id: "s-1"},
    });

    await updateMilestone("p-1", "m-1", {status: "completed"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/milestones/m-1",
      method: "PATCH",
      data: {status: "completed"},
    });
  });

  it("任务列表的筛选项原样透传，未设置时不产生空参数", async () => {
    await fetchTasks("p-1", {page: 1, page_size: 20, status: "in_progress"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/tasks",
      method: "GET",
      params: {page: 1, page_size: 20, status: "in_progress"},
    });
  });

  it("依赖建立与解除走复合主键路径", async () => {
    await createDependency("p-1", "t-1", {depends_on_task_id: "t-2"});
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/tasks/t-1/dependencies",
      method: "POST",
      data: {depends_on_task_id: "t-2"},
    });

    await deleteDependency("p-1", "t-1", "t-2");
    expect(requestMock).toHaveBeenLastCalledWith({
      url: "/projects/p-1/tasks/t-1/dependencies/t-2",
      method: "DELETE",
    });
  });
});
