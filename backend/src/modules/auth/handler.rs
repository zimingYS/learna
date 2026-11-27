use crate::app_state::AppState;
use crate::modules::auth::dto::{AuthResponse, LoginRequest, RegisterRequest};
use crate::modules::auth::service;
use crate::shared::{ApiReply, RequestId};
use axum::extract::State;
use axum::{Extension, Json};

/// 用户注册
pub async fn register(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
    Json(req): Json<RegisterRequest>,
) -> ApiReply<AuthResponse> {
    ApiReply(service::register(&state.db, &state.config, req).await, id)
}

/// 用户登录
pub async fn login(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
    Json(req): Json<LoginRequest>,
) -> ApiReply<AuthResponse> {
    ApiReply(service::login(&state.db, &state.config, req).await, id)
}
