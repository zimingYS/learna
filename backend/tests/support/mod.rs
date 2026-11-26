//! 集成测试公共支撑：AppState 构造、HTTP 请求助手、信封解析
// 镜像字段供各测试二进制按需读取，统一豁免死码告警
#![allow(dead_code)]

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use backend::app_state::AppState;
use backend::config::AppConfig;
use backend::infrastructure::ai_client::AiClient;
use backend::router::routes::build_router;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

/// 统一响应信封（测试侧镜像）
#[derive(Debug, serde::Deserialize)]
pub struct Envelope<T> {
    pub code: String,
    pub message: String,
    pub data: Option<T>,
    pub request_id: String,
}

/// 认证响应数据（测试侧镜像）
#[derive(Debug, serde::Deserialize)]
pub struct AuthData {
    pub token: String,
    pub user: UserData,
}

/// 用户响应数据（测试侧镜像）
#[derive(Debug, serde::Deserialize)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub display_name: String,
}

/// 学习记录响应数据（测试侧镜像）
#[derive(Debug, serde::Deserialize)]
pub struct RecordData {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub subject: Option<String>,
}

/// 学习记录列表数据（测试侧镜像）
#[derive(Debug, serde::Deserialize)]
pub struct RecordListData {
    pub items: Vec<RecordData>,
    pub total: i64,
}

/// 用注入的测试数据库构建完整 Router（含全部中间件）
pub fn build_app(pool: PgPool) -> Router {
    let config = AppConfig {
        app_env: "test".to_string(),
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        database_url: "postgres://test".to_string(),
        database_max_connections: 5,
        ai_service_base_url: "http://ai-service.invalid".to_string(),
        ai_service_token: None,
        encryption_master_key: None,
        storage_provider: "local".to_string(),
        storage_root_dir: PathBuf::from("./data/test"),
        jwt_secret: "test-secret".to_string(),
    };

    let ai_client = AiClient::new(config.ai_service_base_url.clone(), None)
        .expect("测试用 AiClient 构造不应失败");

    build_router(AppState {
        db: pool,
        config: Arc::new(config),
        ai_client,
    })
}

/// 发送 JSON 请求并解析信封
pub async fn send_json<T: DeserializeOwned>(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Envelope<T>) {
    let (status, value) = send_raw(app, method, uri, token, body).await;
    let envelope =
        serde_json::from_value(value).unwrap_or_else(|e| panic!("响应体不是合法信封: {e}"));

    (status, envelope)
}

/// 发送 JSON 请求，返回状态码与原始 JSON 体（用于非信封响应）
pub async fn send_raw(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }

    let request = builder
        .header("content-type", "application/json")
        .body(match body {
            Some(body) => Body::from(body.to_string()),
            None => Body::empty(),
        })
        .expect("构建请求失败");

    let response = app.clone().oneshot(request).await.expect("路由应响应");
    let status = response.status();

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取响应体失败");

    if bytes.is_empty() {
        return (status, Value::Null);
    }

    let value = serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!(
            "响应体不是 JSON: {e}; status={status}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });

    (status, value)
}

/// 注册测试用户并返回 token
pub async fn register_user(app: &Router, email: &str) -> String {
    let (status, envelope) = send_json::<AuthData>(
        app,
        Method::POST,
        "/auth/register",
        None,
        Some(json!({
            "email": email,
            "password": "passw0rd",
            "display_name": "测试用户"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.code, "200");
    envelope.data.expect("注册应返回认证数据").token
}
