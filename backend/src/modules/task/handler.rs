//! M04 任务 HTTP 层。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::task::dto::{
    CreateTaskDependencyRequest, CreateTaskRequest, TaskDependencyItem, TaskDependencyListData,
    TaskItem, TaskListData, TaskQuery, UpdateTaskRequest,
};
use crate::modules::task::service;
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<TaskQuery>,
) -> ApiReply<TaskListData> {
    let (limit, offset) = query.window();
    ApiReply(
        service::list(&state.db, user_id.0, project_id, &query, limit, offset).await,
        req_id,
    )
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateTaskRequest>,
) -> ApiReply<TaskItem> {
    ApiReply(
        service::create(&state.db, user_id.0, project_id, payload).await,
        req_id,
    )
}

pub async fn detail(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<TaskItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, project_id, task_id).await,
        req_id,
    )
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskRequest>,
) -> ApiReply<TaskItem> {
    ApiReply(
        service::update(&state.db, user_id.0, project_id, task_id, payload).await,
        req_id,
    )
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, project_id, task_id).await,
        req_id,
    )
}

pub async fn list_dependencies(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<TaskDependencyListData> {
    ApiReply(
        service::list_dependencies(&state.db, user_id.0, project_id, task_id).await,
        req_id,
    )
}

pub async fn create_dependency(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateTaskDependencyRequest>,
) -> ApiReply<TaskDependencyItem> {
    ApiReply(
        service::create_dependency(&state.db, user_id.0, project_id, task_id, payload).await,
        req_id,
    )
}

pub async fn delete_dependency(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((project_id, task_id, depends_on_task_id)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiReply<()> {
    ApiReply(
        service::delete_dependency(
            &state.db,
            user_id.0,
            project_id,
            task_id,
            depends_on_task_id,
        )
        .await,
        req_id,
    )
}
