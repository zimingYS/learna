//! M02 学习项目的 HTTP 层：只做提取器与响应封装，不含业务分支。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::project::dto::{
    CreateProjectRequest, ExtendDeadlineRequest, ProjectItem, ProjectListData, ProjectQuery,
    PurgeProjectRequest, UpdateProjectRequest,
};
use crate::modules::project::rules::ProjectAction;
use crate::modules::project::service;
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

/// 项目列表
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<ProjectQuery>,
) -> ApiReply<ProjectListData> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * page_size;

    ApiReply(
        service::list(&state.db, user_id.0, &query, page_size, offset).await,
        req_id,
    )
}

/// 创建项目
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateProjectRequest>,
) -> ApiReply<ProjectItem> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

/// 项目详情
pub async fn detail(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
) -> ApiReply<ProjectItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, project_id).await,
        req_id,
    )
}

/// 部分更新
pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> ApiReply<ProjectItem> {
    ApiReply(
        service::update(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

/// 删除草稿项目
pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(
        service::delete_draft(&state.db, user_id.0, project_id).await,
        req_id,
    )
}

/// 强制删除
pub async fn purge(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<PurgeProjectRequest>,
) -> ApiReply<()> {
    ApiReply(
        service::purge(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

/// 修改 fixed 截止日期
pub async fn extend_deadline(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<ExtendDeadlineRequest>,
) -> ApiReply<ProjectItem> {
    ApiReply(
        service::extend_deadline(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

/// 六个生命周期动作：共用同一段编排，差异只在动作枚举
macro_rules! action_handler {
    ($name:ident, $action:expr) => {
        pub async fn $name(
            State(state): State<AppState>,
            Extension(user_id): Extension<UserId>,
            Extension(req_id): Extension<RequestId>,
            Path(project_id): Path<Uuid>,
        ) -> ApiReply<ProjectItem> {
            ApiReply(
                service::apply_action(&state.db, user_id.0, project_id, $action).await,
                req_id,
            )
        }
    };
}

action_handler!(activate, ProjectAction::Activate);
action_handler!(pause, ProjectAction::Pause);
action_handler!(resume, ProjectAction::Resume);
action_handler!(complete, ProjectAction::Complete);
action_handler!(reopen, ProjectAction::Reopen);
action_handler!(archive, ProjectAction::Archive);
