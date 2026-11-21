use crate::app_state::AppState;
use crate::router::health;
use crate::shared::types::request_id_middleware;
use axum::routing::get;
use axum::{Router, middleware};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
