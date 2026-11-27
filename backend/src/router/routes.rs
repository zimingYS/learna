use crate::app_state::AppState;
use crate::router::{auth, health, learning_record};
use crate::shared::middleware::{auth_middleware, request_id_middleware};
use axum::{Router, middleware};

pub fn build_router(state: AppState) -> Router {
    // 业务接口统一挂在 /api/v1 下；健康检查保留根路径（基础设施端点，不版本化）
    let api = Router::new().nest("/auth", auth::router()).nest(
        "/records",
        learning_record::router().layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        )),
    );

    Router::new()
        .merge(health::router())
        .nest("/api/v1", api)
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
