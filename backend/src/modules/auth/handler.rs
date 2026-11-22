use crate::app_state::AppState;
use crate::modules::auth::dto::{AuthResponse, LoginRequest, RegisterRequest};
use crate::modules::auth::service;
use crate::shared::{ApiResponse, AppResult, RequestId};
use axum::extract::State;
use axum::{Extension, Json};

/// 用户注册
pub async fn register(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    let data = service::register(&state.db, &state.config, req).await?;

    Ok(Json(ApiResponse::ok(data).with_request_id(id.as_str())))
}

/// 用户登录
pub async fn login(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    let data = service::login(&state.db, &state.config, req).await?;

    Ok(Json(ApiResponse::ok(data).with_request_id(id.as_str())))
}
