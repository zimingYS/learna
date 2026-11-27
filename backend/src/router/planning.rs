use axum::Router;
use axum::routing::{get, post};

use crate::app_state::AppState;
use crate::modules::{roadmap as rm, task as tk};

/// M04 规划域端点：全部嵌套在 `/api/v1/projects/{project_id}` 之下，
/// 由 `router::project` 与本 router 合并后统一挂载。
pub fn router() -> Router<AppState> {
    Router::new()
        // 目标
        .route(
            "/{project_id}/goals",
            get(rm::list_goals).post(rm::create_goal),
        )
        .route(
            "/{project_id}/goals/{goal_id}",
            get(rm::detail_goal)
                .patch(rm::update_goal)
                .delete(rm::delete_goal),
        )
        .route(
            "/{project_id}/goals/{goal_id}/stages",
            get(rm::list_goal_stages).put(rm::replace_goal_stages),
        )
        // 路线
        .route(
            "/{project_id}/roadmaps",
            get(rm::list_roadmaps).post(rm::create_roadmap),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}",
            get(rm::detail_roadmap)
                .patch(rm::update_roadmap)
                .delete(rm::delete_roadmap),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/confirm",
            post(rm::confirm_roadmap),
        )
        // 阶段
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages",
            get(rm::list_stages).post(rm::create_stage),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages/{stage_id}",
            get(rm::detail_stage)
                .patch(rm::update_stage)
                .delete(rm::delete_stage),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages/{stage_id}/start",
            post(rm::stage_start),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages/{stage_id}/complete",
            post(rm::stage_complete),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages/{stage_id}/skip",
            post(rm::stage_skip),
        )
        .route(
            "/{project_id}/roadmaps/{roadmap_id}/stages/{stage_id}/reactivate",
            post(rm::stage_reactivate),
        )
        // 里程碑
        .route(
            "/{project_id}/milestones",
            get(rm::list_milestones).post(rm::create_milestone),
        )
        .route(
            "/{project_id}/milestones/{milestone_id}",
            get(rm::detail_milestone)
                .patch(rm::update_milestone)
                .delete(rm::delete_milestone),
        )
        // 任务
        .route("/{project_id}/tasks", get(tk::list).post(tk::create))
        .route(
            "/{project_id}/tasks/{task_id}",
            get(tk::detail).patch(tk::update).delete(tk::delete),
        )
        // 依赖
        .route(
            "/{project_id}/tasks/{task_id}/dependencies",
            get(tk::list_dependencies).post(tk::create_dependency),
        )
        .route(
            "/{project_id}/tasks/{task_id}/dependencies/{depends_on_task_id}",
            axum::routing::delete(tk::delete_dependency),
        )
}
