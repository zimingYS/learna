use crate::config::AppConfig;
use crate::infrastructure::ai_client::AiClient;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
    pub ai_client: AiClient,
}
