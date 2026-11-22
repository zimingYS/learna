use crate::app_state::AppState;
use crate::modules;
use axum::Router;
use axum::routing::post;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(modules::auth::register))
        .route("/login", post(modules::auth::login))
}
