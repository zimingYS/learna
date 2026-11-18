use crate::shared::{AppError, AppResult};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

/// 应用配置，统一从环境变量加载
pub struct AppConfig {
    pub app_env: String,
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub ai_service_base_url: String,
    pub ai_service_token: Option<String>,
    pub encryption_master_key: Option<String>,
    pub storage_provider: String,
    pub storage_root_dir: PathBuf,
}

impl AppConfig {
    /// 从环境变量加载应用配置
    pub fn from_env() -> AppResult<AppConfig> {
        Ok(AppConfig {
            app_env: require_var("APP_ENV")?,
            app_host: require_var("APP_HOST")?,
            app_port: parse_var("APP_PORT", "u16")?,
            database_url: require_var("DATABASE_URL")?,
            database_max_connections: parse_var("DATABASE_MAX_CONNECTIONS", "u32")?,
            ai_service_base_url: require_var("AI_SERVICE_BASE_URL")?,
            ai_service_token: optional_var("AI_SERVICE_TOKEN"),
            encryption_master_key: optional_var("ENCRYPTION_MASTER_KEY"),
            storage_provider: require_var("STORAGE_PROVIDER")?,
            storage_root_dir: PathBuf::from(require_var("STORAGE_ROOT_DIR")?),
        })
    }
}

/// 读取必填环境变量
fn require_var(name: &str) -> AppResult<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Internal(format!("missing required env: {name}(see .env.example)"))
        })
}

/// 读取可选环境变量，空字符串视为未配置
fn optional_var(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.trim().is_empty())
}

/// 读取并解析数字类型环境变量
fn parse_var<T>(name: &str, expected: &str) -> AppResult<T>
where
    T: FromStr,
{
    let value = require_var(name)?;

    value
        .parse()
        .map_err(|_| AppError::Internal(format!("invalid {name}: {value:?}(expect {expected})")))
}
