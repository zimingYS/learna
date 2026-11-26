//! 认证链路集成测试：注册 / 登录（真实 PostgreSQL）

mod support;

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::*;

#[sqlx::test]
async fn register_returns_token_and_user(pool: PgPool) {
    let app = build_app(pool);

    let (status, envelope) = send_json::<AuthData>(
        &app,
        Method::POST,
        "/auth/register",
        None,
        Some(json!({
            "email": "alice@example.com",
            "password": "passw0rd",
            "display_name": "Alice"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.code, "200");
    assert!(!envelope.request_id.is_empty(), "信封应携带 request_id");

    let data = envelope.data.expect("应返回认证数据");
    assert!(!data.token.is_empty());
    assert_eq!(data.user.email, "alice@example.com");
    assert_eq!(data.user.display_name, "Alice");
    assert!(!data.user.id.is_empty());
}

#[sqlx::test]
async fn register_duplicate_email_is_validation_error(pool: PgPool) {
    let app = build_app(pool);
    register_user(&app, "bob@example.com").await;

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::POST,
        "/auth/register",
        None,
        Some(json!({
            "email": "bob@example.com",
            "password": "another-pass",
            "display_name": "Bob2"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(envelope.code, "VALIDATION_ERROR");
}

#[sqlx::test]
async fn login_with_valid_credentials_returns_token(pool: PgPool) {
    let app = build_app(pool);
    register_user(&app, "carol@example.com").await;

    let (status, envelope) = send_json::<AuthData>(
        &app,
        Method::POST,
        "/auth/login",
        None,
        Some(json!({"email": "carol@example.com", "password": "passw0rd"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.expect("应返回认证数据");
    assert!(!data.token.is_empty());
    assert_eq!(data.user.email, "carol@example.com");
}

#[sqlx::test]
async fn login_with_wrong_password_is_unauthorized(pool: PgPool) {
    let app = build_app(pool);
    register_user(&app, "dave@example.com").await;

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::POST,
        "/auth/login",
        None,
        Some(json!({"email": "dave@example.com", "password": "wrong-pass"})),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(envelope.code, "UNAUTHORIZED");
}

#[sqlx::test]
async fn login_with_unknown_email_is_unauthorized(pool: PgPool) {
    let app = build_app(pool);

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::POST,
        "/auth/login",
        None,
        Some(json!({"email": "nobody@example.com", "password": "passw0rd"})),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(envelope.code, "UNAUTHORIZED");
}
