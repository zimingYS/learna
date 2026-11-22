use crate::app_state::AppState;
use crate::shared::{ApiResponse, AppResult, RequestId};
use axum::extract::State;
use axum::routing::get;
use axum::{Extension, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

pub async fn healthz(Extension(id): Extension<RequestId>) -> Json<ApiResponse<()>> {
    Json(ApiResponse::ok(()).with_request_id(id.as_str()))
}

pub async fn readyz(
    State(state): State<AppState>,
    Extension(id): Extension<RequestId>,
) -> AppResult<Json<ApiResponse<()>>> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(ApiResponse::ok(()).with_request_id(id.as_str())))
}
