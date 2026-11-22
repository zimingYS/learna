use crate::app_state::AppState;
use crate::modules::learning_record as lr;
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(lr::create).get(lr::list))
        .route("/{id}", get(lr::detail).put(lr::update).delete(lr::delete))
}
