use crate::app_state::AppState;
use crate::modules::learning_record::dto::{
    CreateRecordRequest, RecordListResponse, RecordQuery, RecordResponse, UpdateRecordRequest,
};
use crate::modules::learning_record::service;
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

/// 创建学习记录
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateRecordRequest>,
) -> ApiReply<RecordResponse> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

/// 获取学习记录列表
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<RecordQuery>,
) -> ApiReply<RecordListResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);

    let offset = (page - 1) * page_size;

    ApiReply(
        service::list(&state.db, user_id.0, page_size, offset).await,
        req_id,
    )
}

/// 获取单条学习记录
pub async fn detail(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
) -> ApiReply<RecordResponse> {
    ApiReply(
        service::detail(&state.db, user_id.0, record_id).await,
        req_id,
    )
}

/// 更新学习记录（PATCH 部分更新）
pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
    Json(payload): Json<UpdateRecordRequest>,
) -> ApiReply<RecordResponse> {
    ApiReply(
        service::update(&state.db, user_id.0, record_id, payload).await,
        req_id,
    )
}

/// 删除学习记录
pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, record_id).await,
        req_id,
    )
}
