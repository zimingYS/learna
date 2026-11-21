use std::sync::Arc;

use backend::app_state::AppState;
use backend::config::AppConfig;
use backend::infrastructure::ai_client::AiClient;
use backend::infrastructure::database;
use backend::infrastructure::telemetry::init_tracing;
use backend::router::routes;
use backend::shared::{AppError, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    // 加载.env
    dotenvy::dotenv().ok();
    // 加载日志
    init_tracing();

    // 加载配置
    let cfg = AppConfig::from_env()?;
    let addr = format!("{}:{}", cfg.app_host, cfg.app_port);

    // 创建数据库池
    let db = database::create_pool(&cfg)?;
    // 执行数据库迁移应用
    database::run_migrations(&db).await?;

    // 创建AI客户端
    let ai_client = AiClient::new(
        cfg.ai_service_base_url.clone(),
        cfg.ai_service_token.clone(),
    )?;

    let state = AppState {
        db,
        config: Arc::new(cfg),
        ai_client,
    };

    // 路由
    let router = routes::build_router(state);
    // 监听
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Internal(format!("绑定地址 {addr} 失败: {e}")))?;
    tracing::info!("后端已启动监听: {}", addr);

    // 启动服务
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Internal(format!("服务器内部错误: {e}")))?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("未能安装 Ctrl+C 信号处理程序");
    tracing::info!("收到关闭信号，正在关闭...");
}
