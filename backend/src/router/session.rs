//! M05 学习活动与证据路由。
//!
//! 学习活动是顶层资源（一次活动可关联多个项目），证据同样独立：`session_id` 可空，
//! 可空的父资源不该出现在路径里（ADR-010 §4）。

use axum::Router;
use axum::routing::{get, put};

use crate::app_state::AppState;
use crate::modules::evidence::handler as evidence_handler;
use crate::modules::learning_session::handler;

/// `/learning-sessions`：主资源 + 三个整体替换子资源（不提供关联的 `GET`，详情已内联）
pub fn sessions_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handler::list_sessions).post(handler::create_session),
        )
        .route(
            "/{session_id}",
            get(handler::detail_session)
                .patch(handler::update_session)
                .delete(handler::delete_session),
        )
        .route("/{session_id}/contents", put(handler::set_session_contents))
        .route("/{session_id}/projects", put(handler::set_session_projects))
        .route("/{session_id}/tasks", put(handler::set_session_tasks))
}

/// `/evidence`：学习证据（无 `PATCH`：证据是快照）
pub fn evidence_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(evidence_handler::list_evidence).post(evidence_handler::create_evidence),
        )
        .route(
            "/{evidence_id}",
            get(evidence_handler::detail_evidence).delete(evidence_handler::delete_evidence),
        )
}
