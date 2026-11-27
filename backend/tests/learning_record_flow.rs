//! 学习记录 CRUD 集成测试（真实 PostgreSQL，覆盖鉴权与越权）

mod support;

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::*;

#[sqlx::test]
async fn records_require_auth(pool: PgPool) {
    let app = build_app(pool);

    let (status, envelope) =
        send_json::<serde_json::Value>(&app, Method::GET, "/api/v1/records", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(envelope.code, "UNAUTHORIZED");

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/v1/records",
        Some("not-a-jwt"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(envelope.code, "UNAUTHORIZED");
}

#[sqlx::test]
async fn record_crud_roundtrip(pool: PgPool) {
    let app = build_app(pool);
    let token = register_user(&app, "writer@example.com").await;

    // 创建
    let (status, envelope) = send_json::<RecordData>(
        &app,
        Method::POST,
        "/api/v1/records",
        Some(&token),
        Some(json!({
            "title": "数据结构复习",
            "content": "链表与栈",
            "duration_minutes": 45,
            "subject": "408"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.code, "200");
    assert!(!envelope.request_id.is_empty());
    let created = envelope.data.expect("应返回记录");
    assert_eq!(created.title, "数据结构复习");
    assert_eq!(created.duration_minutes, Some(45));
    let record_id = created.id;

    // 详情
    let (status, envelope) = send_json::<RecordData>(
        &app,
        Method::GET,
        &format!("/api/v1/records/{record_id}"),
        Some(&token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let fetched = envelope.data.expect("应返回记录");
    assert_eq!(fetched.id, record_id);
    assert_eq!(fetched.subject.as_deref(), Some("408"));

    // 更新
    let (status, envelope) = send_json::<RecordData>(
        &app,
        Method::PATCH,
        &format!("/api/v1/records/{record_id}"),
        Some(&token),
        Some(json!({"title": "数据结构强化", "duration_minutes": 60})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated = envelope.data.expect("应返回记录");
    assert_eq!(updated.title, "数据结构强化");
    assert_eq!(updated.duration_minutes, Some(60));
    assert_eq!(
        updated.content.as_deref(),
        Some("链表与栈"),
        "未提交字段不应变化"
    );

    // 删除
    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/records/{record_id}"),
        Some(&token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.code, "200");

    // 删除后详情应 404
    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::GET,
        &format!("/api/v1/records/{record_id}"),
        Some(&token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(envelope.code, "NOT_FOUND");
}

#[sqlx::test]
async fn list_paginates_and_counts_total(pool: PgPool) {
    let app = build_app(pool);
    let token = register_user(&app, "pager@example.com").await;

    for i in 1..=3 {
        let (status, _) = send_json::<RecordData>(
            &app,
            Method::POST,
            "/api/v1/records",
            Some(&token),
            Some(json!({"title": format!("记录{i}")})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, envelope) = send_json::<RecordListData>(
        &app,
        Method::GET,
        "/api/v1/records?page=1&page_size=2",
        Some(&token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let page = envelope.data.expect("应返回列表");
    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 2);
}

#[sqlx::test]
async fn detail_of_other_user_is_not_found(pool: PgPool) {
    let app = build_app(pool);
    let owner_token = register_user(&app, "owner@example.com").await;
    let intruder_token = register_user(&app, "intruder@example.com").await;

    let (_, envelope) = send_json::<RecordData>(
        &app,
        Method::POST,
        "/api/v1/records",
        Some(&owner_token),
        Some(json!({"title": "私有记录"})),
    )
    .await;
    let record_id = envelope.data.expect("应返回记录").id;

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::GET,
        &format!("/api/v1/records/{record_id}"),
        Some(&intruder_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(envelope.code, "NOT_FOUND");
}

#[sqlx::test]
async fn update_without_fields_is_validation_error(pool: PgPool) {
    let app = build_app(pool);
    let token = register_user(&app, "updater@example.com").await;

    let (_, envelope) = send_json::<RecordData>(
        &app,
        Method::POST,
        "/api/v1/records",
        Some(&token),
        Some(json!({"title": "待更新"})),
    )
    .await;
    let record_id = envelope.data.expect("应返回记录").id;

    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/records/{record_id}"),
        Some(&token),
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(envelope.code, "VALIDATION_ERROR");
}

#[sqlx::test]
async fn delete_missing_record_is_not_found(pool: PgPool) {
    let app = build_app(pool);
    let token = register_user(&app, "cleaner@example.com").await;

    let missing_id = "00000000-0000-0000-0000-000000000000";
    let (status, envelope) = send_json::<serde_json::Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/records/{missing_id}"),
        Some(&token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(envelope.code, "NOT_FOUND");
}
