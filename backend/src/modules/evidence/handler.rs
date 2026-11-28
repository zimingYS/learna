//! M05 学习证据 HTTP 层。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::evidence::dto::{
    CreateEvidenceRequest, EvidenceItem, EvidenceListData, EvidenceQuery,
};
use crate::modules::evidence::service;
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

pub async fn list_evidence(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<EvidenceQuery>,
) -> ApiReply<EvidenceListData> {
    ApiReply(service::list(&state.db, user_id.0, query).await, req_id)
}

pub async fn create_evidence(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateEvidenceRequest>,
) -> ApiReply<EvidenceItem> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

pub async fn detail_evidence(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(evidence_id): Path<Uuid>,
) -> ApiReply<EvidenceItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, evidence_id).await,
        req_id,
    )
}

pub async fn delete_evidence(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(evidence_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, evidence_id).await,
        req_id,
    )
}
