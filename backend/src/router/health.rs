use crate::app_state::AppState;
use crate::shared::{ApiReply, AppError, RequestId};
use axum::extract::State;
use axum::routing::get;
use axum::{Extension, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

pub async fn healthz(Extension(id): Extension<RequestId>) -> ApiReply<()> {
    ApiReply(Ok(()), id)
}

pub async fn readyz(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
) -> ApiReply<()> {
    let probe: Result<(), AppError> = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map(|_| ())
        .map_err(AppError::from);

    ApiReply(probe, id)
}
