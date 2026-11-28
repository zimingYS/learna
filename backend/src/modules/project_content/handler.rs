//! M03 项目内容上下文 HTTP 层。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::project_content::dto::{
    CreateProjectContentRequest, ProjectContentItem, ProjectContentListData, ProjectContentQuery,
    UpdateProjectContentRequest,
};
use crate::modules::project_content::service;
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

pub async fn list_project_contents(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<ProjectContentQuery>,
) -> ApiReply<ProjectContentListData> {
    ApiReply(service::list(&state.db, user_id.0, query).await, req_id)
}

pub async fn create_project_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateProjectContentRequest>,
) -> ApiReply<ProjectContentItem> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

pub async fn detail_project_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_content_id): Path<Uuid>,
) -> ApiReply<ProjectContentItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, project_content_id).await,
        req_id,
    )
}

pub async fn update_project_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_content_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectContentRequest>,
) -> ApiReply<ProjectContentItem> {
    ApiReply(
        service::update(&state.db, user_id.0, project_content_id, payload).await,
        req_id,
    )
}

pub async fn delete_project_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_content_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, project_content_id).await,
        req_id,
    )
}
