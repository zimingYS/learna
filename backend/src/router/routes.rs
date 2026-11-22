use crate::app_state::AppState;
use crate::router::{auth, health};
use crate::shared::types::request_id_middleware;
use axum::{Router, middleware};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::router())
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
