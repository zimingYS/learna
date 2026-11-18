use crate::config::AppConfig;
use crate::infrastructure::ai_client::AiClient;
use crate::shared::ApiResponse;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
    pub ai_client: AiClient,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(state)
}

async fn healthz() -> Json<ApiResponse<()>> {
    Json(ApiResponse::ok(()))
}
