use crate::app_state::AppState;
use crate::router::{auth, content, health, learning_record, planning, project};
use crate::shared::middleware::{auth_middleware, request_id_middleware};
use axum::{Router, middleware};

pub fn build_router(state: AppState) -> Router {
    // M02 项目本体与 M04 规划域共用 /projects 前缀，先合并再挂载
    let projects = project::router().merge(planning::router());

    // 需要身份认证的业务路由集中在一个子 router 上挂一次中间件，
    // 避免每加一个业务模块就复制一遍 layer 调用（漏挂即越权）
    let authed = Router::new()
        .nest("/records", learning_record::router())
        .nest("/learning-contents", content::contents_router())
        .nest("/content-tags", content::tags_router())
        .nest("/project-contents", content::project_contents_router())
        .nest("/projects", projects)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // 业务接口统一挂在 /api/v1 下；健康检查保留根路径（基础设施端点，不版本化）
    let api = Router::new().nest("/auth", auth::router()).merge(authed);

    Router::new()
        .merge(health::router())
        .nest("/api/v1", api)
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
