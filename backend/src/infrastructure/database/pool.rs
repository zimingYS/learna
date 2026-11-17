use crate::config::AppConfig;
use crate::shared::AppResult;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub fn create_pool(cfg: &AppConfig) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(cfg.database_max_connections)
        .connect_lazy(&cfg.database_url)?;

    Ok(pool)
}
