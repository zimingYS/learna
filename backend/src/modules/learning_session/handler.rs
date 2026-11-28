//! M05 学习活动 HTTP 层。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::learning_session::dto::{
    CreateSessionRequest, SessionContentListData, SessionDetailItem, SessionListData,
    SessionProjectsData, SessionQuery, SessionTaskListData, SetSessionContentsRequest,
    SetSessionProjectsRequest, SetSessionTasksRequest, UpdateSessionRequest,
};
use crate::modules::learning_session::{associations, service};
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<SessionQuery>,
) -> ApiReply<SessionListData> {
    ApiReply(service::list(&state.db, user_id.0, query).await, req_id)
}

pub async fn create_session(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateSessionRequest>,
) -> ApiReply<SessionDetailItem> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

pub async fn detail_session(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
) -> ApiReply<SessionDetailItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, session_id).await,
        req_id,
    )
}

pub async fn update_session(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<UpdateSessionRequest>,
) -> ApiReply<SessionDetailItem> {
    ApiReply(
        service::update(&state.db, user_id.0, session_id, payload).await,
        req_id,
    )
}

pub async fn delete_session(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, session_id).await,
        req_id,
    )
}

pub async fn set_session_contents(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<SetSessionContentsRequest>,
) -> ApiReply<SessionContentListData> {
    ApiReply(
        associations::set_contents(&state.db, user_id.0, session_id, payload).await,
        req_id,
    )
}

pub async fn set_session_projects(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<SetSessionProjectsRequest>,
) -> ApiReply<SessionProjectsData> {
    ApiReply(
        associations::set_projects(&state.db, user_id.0, session_id, payload).await,
        req_id,
    )
}

pub async fn set_session_tasks(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<SetSessionTasksRequest>,
) -> ApiReply<SessionTaskListData> {
    ApiReply(
        associations::set_tasks(&state.db, user_id.0, session_id, payload).await,
        req_id,
    )
}
