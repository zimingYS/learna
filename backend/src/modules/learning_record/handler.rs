use crate::app_state::AppState;
use crate::modules::learning_record::dto::{
    CreateRecordRequest, RecordListResponse, RecordQuery, RecordResponse, UpdateRecordRequest,
};
use crate::modules::learning_record::service;
use crate::shared::types::UserId;
use crate::shared::{ApiResponse, AppResult, RequestId};
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

/// 创建学习记录
pub async fn create(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateRecordRequest>,
) -> AppResult<Json<ApiResponse<RecordResponse>>> {
    let record = service::create(&state.db, user_id.0, payload).await?;

    Ok(Json(ApiResponse::ok(record).with_request_id(req_id.0)))
}

/// 获取学习记录列表
pub async fn list(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<RecordQuery>,
) -> AppResult<Json<ApiResponse<RecordListResponse>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);

    let offset = (page - 1) * page_size;

    let records = service::list(&state.db, user_id.0, page_size, offset).await?;

    Ok(Json(ApiResponse::ok(records).with_request_id(req_id.0)))
}

/// 获取单条学习记录
pub async fn detail(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<RecordResponse>>> {
    let record = service::detail(&state.db, user_id.0, record_id).await?;

    Ok(Json(ApiResponse::ok(record).with_request_id(req_id.0)))
}

/// 更新学习记录
pub async fn update(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
    Json(payload): Json<UpdateRecordRequest>,
) -> AppResult<Json<ApiResponse<RecordResponse>>> {
    let record = service::update(&state.db, user_id.0, record_id, payload).await?;

    Ok(Json(ApiResponse::ok(record).with_request_id(req_id.0)))
}

/// 删除学习记录
pub async fn delete(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(record_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    service::delete(&state.db, user_id.0, record_id).await?;

    Ok(Json(ApiResponse::ok(()).with_request_id(req_id.0)))
}
