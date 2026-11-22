use crate::app_state::AppState;
use crate::router::{auth, health, learning_record};
use crate::shared::middleware::{auth_middleware, request_id_middleware};
use axum::{Router, middleware};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::router())
        .nest(
            "/records",
            learning_record::router().layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )),
        )
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
