use crate::shared::{AppError, AppResult};
use sqlx::PgPool;
use sqlx::migrate::Migrator;

pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Migrations failed: {}", e)))
}
