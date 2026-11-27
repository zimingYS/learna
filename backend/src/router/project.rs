use axum::Router;
use axum::routing::{get, post};

use crate::app_state::AppState;
use crate::modules::project as pj;

/// 项目本体与生命周期端点（挂载于 `/api/v1/projects`）
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(pj::list).post(pj::create))
        .route(
            "/{project_id}",
            get(pj::detail).patch(pj::update).delete(pj::delete),
        )
        .route("/{project_id}/activate", post(pj::activate))
        .route("/{project_id}/pause", post(pj::pause))
        .route("/{project_id}/resume", post(pj::resume))
        .route("/{project_id}/complete", post(pj::complete))
        .route("/{project_id}/reopen", post(pj::reopen))
        .route("/{project_id}/archive", post(pj::archive))
        .route("/{project_id}/extend-deadline", post(pj::extend_deadline))
        .route("/{project_id}/purge", post(pj::purge))
}
