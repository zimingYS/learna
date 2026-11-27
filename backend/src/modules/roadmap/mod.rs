//! M04 学习规划与调度：目标、路线、阶段、里程碑。
//!
//! 任务与依赖在 `modules::task`：任务的唯一必需归属是 `project_id`，
//! 可以完全不挂路线（D-05），因此不放在这里。

pub mod dto;
pub mod goal_service;
pub mod handler;
pub mod milestone_service;
pub mod roadmap_service;
pub mod rules;
pub mod stage_service;

pub use dto::{GoalItem, GoalListData, MilestoneItem, RoadmapItem, StageItem};
pub use handler::{
    confirm_roadmap, create_goal, create_milestone, create_roadmap, create_stage, delete_goal,
    delete_milestone, delete_roadmap, delete_stage, detail_goal, detail_milestone, detail_roadmap,
    detail_stage, list_goal_stages, list_goals, list_milestones, list_roadmaps, list_stages,
    replace_goal_stages, stage_complete, stage_reactivate, stage_skip, stage_start, update_goal,
    update_milestone, update_roadmap, update_stage,
};
pub use rules::StageAction;
