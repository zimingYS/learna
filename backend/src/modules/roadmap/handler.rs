//! M04 规划域 HTTP 层：目标 / 路线 / 阶段 / 里程碑。
//!
//! 路径参数按资源层级解构为元组，端点签名的嵌套深度即为资源的父链深度。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::roadmap::dto::{
    CreateGoalRequest, CreateMilestoneRequest, CreateRoadmapRequest, CreateStageRequest, GoalItem,
    GoalListData, GoalStageListData, MilestoneItem, MilestoneListData, MilestoneQuery, PageQuery,
    ReplaceGoalStageLinksRequest, RoadmapItem, RoadmapListData, StageItem, StageListData,
    UpdateGoalRequest, UpdateMilestoneRequest, UpdateRoadmapRequest, UpdateStageRequest,
};
use crate::modules::roadmap::rules::StageAction;
use crate::modules::roadmap::{goal_service, milestone_service, roadmap_service, stage_service};
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

/// 级联删除确认参数（D-21）
#[derive(Debug, Deserialize)]
pub struct CascadeQuery {
    #[serde(default)]
    pub cascade: bool,
}

// ---------------------------------------------------------------------------
// 目标
// ---------------------------------------------------------------------------

pub async fn list_goals(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<PageQuery>,
) -> ApiReply<GoalListData> {
    let (limit, offset) = query.window();
    ApiReply(
        goal_service::list(&state.db, user_id.0, project_id, limit, offset).await,
        req_id,
    )
}

pub async fn create_goal(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateGoalRequest>,
) -> ApiReply<GoalItem> {
    ApiReply(
        goal_service::create(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

pub async fn detail_goal(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, goal_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<GoalItem> {
    ApiReply(
        goal_service::detail(&state.db, user_id.0, project_id, goal_id).await,
        req_id,
    )
}

pub async fn update_goal(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, goal_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateGoalRequest>,
) -> ApiReply<GoalItem> {
    ApiReply(
        goal_service::update(&state.db, user_id.0, project_id, goal_id, payload).await,
        req_id,
    )
}

pub async fn delete_goal(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, goal_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<CascadeQuery>,
) -> ApiReply<()> {
    ApiReply(
        goal_service::delete(&state.db, user_id.0, project_id, goal_id, query.cascade).await,
        req_id,
    )
}

pub async fn list_goal_stages(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, goal_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<GoalStageListData> {
    ApiReply(
        goal_service::list_goal_stages(&state.db, user_id.0, project_id, goal_id).await,
        req_id,
    )
}

pub async fn replace_goal_stages(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, goal_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ReplaceGoalStageLinksRequest>,
) -> ApiReply<GoalStageListData> {
    ApiReply(
        goal_service::replace_goal_stages(&state.db, user_id.0, project_id, goal_id, payload).await,
        req_id,
    )
}

// ---------------------------------------------------------------------------
// 路线
// ---------------------------------------------------------------------------

pub async fn list_roadmaps(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<PageQuery>,
) -> ApiReply<RoadmapListData> {
    let (limit, offset) = query.window();
    ApiReply(
        roadmap_service::list(&state.db, user_id.0, project_id, limit, offset).await,
        req_id,
    )
}

pub async fn create_roadmap(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateRoadmapRequest>,
) -> ApiReply<RoadmapItem> {
    ApiReply(
        roadmap_service::create(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

pub async fn detail_roadmap(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<RoadmapItem> {
    ApiReply(
        roadmap_service::detail(&state.db, user_id.0, project_id, roadmap_id).await,
        req_id,
    )
}

pub async fn update_roadmap(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateRoadmapRequest>,
) -> ApiReply<RoadmapItem> {
    ApiReply(
        roadmap_service::update(&state.db, user_id.0, project_id, roadmap_id, payload).await,
        req_id,
    )
}

pub async fn confirm_roadmap(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<RoadmapItem> {
    ApiReply(
        roadmap_service::confirm(&state.db, user_id.0, project_id, roadmap_id).await,
        req_id,
    )
}

pub async fn delete_roadmap(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<CascadeQuery>,
) -> ApiReply<()> {
    ApiReply(
        roadmap_service::delete(&state.db, user_id.0, project_id, roadmap_id, query.cascade).await,
        req_id,
    )
}

// ---------------------------------------------------------------------------
// 阶段
// ---------------------------------------------------------------------------

pub async fn list_stages(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PageQuery>,
) -> ApiReply<StageListData> {
    let (limit, offset) = query.window();
    ApiReply(
        stage_service::list(&state.db, user_id.0, project_id, roadmap_id, limit, offset).await,
        req_id,
    )
}

pub async fn create_stage(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateStageRequest>,
) -> ApiReply<StageItem> {
    ApiReply(
        stage_service::create(&state.db, user_id.0, project_id, roadmap_id, payload).await,
        req_id,
    )
}

pub async fn detail_stage(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id, stage_id)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiReply<StageItem> {
    ApiReply(
        stage_service::detail(&state.db, user_id.0, project_id, roadmap_id, stage_id).await,
        req_id,
    )
}

pub async fn update_stage(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id, stage_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateStageRequest>,
) -> ApiReply<StageItem> {
    ApiReply(
        stage_service::update(
            &state.db, user_id.0, project_id, roadmap_id, stage_id, payload,
        )
        .await,
        req_id,
    )
}

pub async fn delete_stage(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, roadmap_id, stage_id)): Path<(Uuid, Uuid, Uuid)>,
    Query(query): Query<CascadeQuery>,
) -> ApiReply<()> {
    ApiReply(
        stage_service::delete(
            &state.db,
            user_id.0,
            project_id,
            roadmap_id,
            stage_id,
            query.cascade,
        )
        .await,
        req_id,
    )
}

/// 四个阶段动作共用一个编排，差异只在动作枚举
macro_rules! stage_action_handler {
    ($name:ident, $action:expr) => {
        pub async fn $name(
            State(state): State<AppState>,
            Extension(user_id): Extension<UserId>,
            Extension(req_id): Extension<RequestId>,
            Path((project_id, roadmap_id, stage_id)): Path<(Uuid, Uuid, Uuid)>,
        ) -> ApiReply<StageItem> {
            ApiReply(
                stage_service::apply_action(
                    &state.db, user_id.0, project_id, roadmap_id, stage_id, $action,
                )
                .await,
                req_id,
            )
        }
    };
}

stage_action_handler!(stage_start, StageAction::Start);
stage_action_handler!(stage_complete, StageAction::Complete);
stage_action_handler!(stage_skip, StageAction::Skip);
stage_action_handler!(stage_reactivate, StageAction::Reactivate);

// ---------------------------------------------------------------------------
// 里程碑
// ---------------------------------------------------------------------------

pub async fn list_milestones(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<MilestoneQuery>,
) -> ApiReply<MilestoneListData> {
    let (limit, offset) = query.window();
    ApiReply(
        milestone_service::list(&state.db, user_id.0, project_id, &query, limit, offset).await,
        req_id,
    )
}

pub async fn create_milestone(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateMilestoneRequest>,
) -> ApiReply<MilestoneItem> {
    ApiReply(
        milestone_service::create(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

pub async fn detail_milestone(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, milestone_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<MilestoneItem> {
    ApiReply(
        milestone_service::detail(&state.db, user_id.0, project_id, milestone_id).await,
        req_id,
    )
}

pub async fn update_milestone(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, milestone_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMilestoneRequest>,
) -> ApiReply<MilestoneItem> {
    ApiReply(
        milestone_service::update(&state.db, user_id.0, project_id, milestone_id, payload).await,
        req_id,
    )
}

pub async fn delete_milestone(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, milestone_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<()> {
    ApiReply(
        milestone_service::delete(&state.db, user_id.0, project_id, milestone_id).await,
        req_id,
    )
}
