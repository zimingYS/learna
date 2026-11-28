//! M05 学习活动与证据集成测试：时长生成、归属隔离、关联替换事务、软删、证据守卫。

mod support;

use axum::Router;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use support::{register_user, send_json};

async fn create_project(app: &Router, token: &str, name: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/projects",
        Some(token),
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建项目应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

async fn create_content(app: &Router, token: &str, title: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(token),
        Some(json!({ "content_type": "knowledge_point", "title": title })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建内容应成功");
    envelope.data.unwrap()["item"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_task(app: &Router, token: &str, project_id: &str, title: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        &format!("/api/v1/projects/{project_id}/tasks"),
        Some(token),
        Some(json!({ "title": title })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建任务应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

async fn create_session(app: &Router, token: &str, body: Value) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建学习活动应成功: {envelope:?}");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

/// 一小时区间的最小活动
fn hour_session(starts_at: &str, ends_at: &str) -> Value {
    json!({ "started_at": starts_at, "ended_at": ends_at })
}

#[sqlx::test]
async fn create_session_links_projects_contents_and_tasks(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-create@example.com").await;

    let primary = create_project(&app, &token, "高数").await;
    let related = create_project(&app, &token, "408").await;
    let task_project = create_project(&app, &token, "英语").await;
    let content = create_content(&app, &token, "极限的定义").await;
    let task = create_task(&app, &token, &task_project, "背完 100 词").await;

    let body = json!({
        "title": "  高数第一轮  ",
        "activity_type": "practice",
        "started_at": "2026-11-28T09:00:00Z",
        "ended_at": "2026-11-28T11:00:00Z",
        "duration_minutes": 90,
        "primary_project_id": primary,
        "related_project_ids": [related, task_project],
        "contents": [{
            "content_id": content,
            "contribution_type": "practiced",
            "note": " 第 3 节 "
        }],
        "tasks": [{ "task_id": task, "contribution_minutes": 60 }],
        "summary": "函数与极限",
        "what_done": "做了 12 道题",
        "created_source": "timer"
    });

    let id = create_session(&app, &token, body).await;

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let data = envelope.data.unwrap();
    // 区间 120 分钟，上报 90 → 采信上报值（区间内有暂停）
    assert_eq!(data["duration_minutes"], json!(90));
    assert_eq!(data["title"], "高数第一轮");
    assert_eq!(data["activity_type"], "practice");
    assert_eq!(data["created_source"], "timer");
    assert_eq!(
        data["primary_project_id"].as_str().unwrap(),
        primary.as_str()
    );
    assert_eq!(data["summary"], "函数与极限");
    assert_eq!(data["what_done"], "做了 12 道题");

    let related_ids = data["related_project_ids"].as_array().unwrap();
    assert_eq!(related_ids.len(), 2);
    assert!(related_ids.iter().any(|value| value == &json!(related)));
    assert!(
        related_ids
            .iter()
            .any(|value| value == &json!(task_project))
    );

    assert_eq!(data["contents"][0]["contribution_type"], "practiced");
    assert_eq!(data["contents"][0]["note"], "第 3 节");
    assert_eq!(data["tasks"][0]["contribution_minutes"], json!(60));

    // 列表视图不内联关联，只回主字段
    let (status, list) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-sessions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = list.data.unwrap();
    assert_eq!(list["total"], json!(1));
    assert!(list["items"][0].get("summary").is_none());
    assert_eq!(list["items"][0]["id"].as_str().unwrap(), id);
}

#[sqlx::test]
async fn duration_follows_min_rule_and_source_is_guarded(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-duration@example.com").await;

    // 只给区间 → 时长 = 区间分钟数
    let id = create_session(
        &app,
        &token,
        hour_session("2026-11-28T09:00:00Z", "2026-11-28T10:15:00Z"),
    )
    .await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["duration_minutes"], json!(75));

    // 上报超过区间 → 422，message 带两个数字
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&token),
        Some(json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "duration_minutes": 90
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        envelope.message.contains("90") && envelope.message.contains("60"),
        "错误文案应带上报值与区间值: {}",
        envelope.message
    );

    // 区间反向 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&token),
        Some(hour_session("2026-11-28T10:00:00Z", "2026-11-28T09:00:00Z")),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // imported 只能由迁移脚本写入（ADR-010 §2）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&token),
        Some(json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "created_source": "imported"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(envelope.message.contains("imported"));

    // 跨午夜区间按实际经过时间计算
    let id = create_session(
        &app,
        &token,
        hour_session("2026-11-28T23:30:00Z", "2026-11-29T00:30:00Z"),
    )
    .await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["duration_minutes"], json!(60));
}

#[sqlx::test]
async fn foreign_projects_and_contents_leave_no_partial_rows(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let owner = register_user(&app, "session-owner@example.com").await;
    let other = register_user(&app, "session-other@example.com").await;

    let own_project = create_project(&app, &owner, "自己的项目").await;
    let foreign_project = create_project(&app, &other, "别人的项目").await;
    let foreign_content = create_content(&app, &other, "别人的内容").await;

    // 主要项目是别人的 → 404
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&owner),
        Some(json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "primary_project_id": foreign_project
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 自己的 + 别人的混在一起 → 404，且整体回滚
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&owner),
        Some(json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "primary_project_id": own_project,
            "related_project_ids": [foreign_project]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 别人的内容 → 404
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-sessions",
        Some(&owner),
        Some(json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "contents": [{ "content_id": foreign_content, "contribution_type": "covered" }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 三次失败都不应留下主表行
    let (_, list) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-sessions",
        Some(&owner),
        None,
    )
    .await;
    assert_eq!(list.data.unwrap()["total"], json!(0));

    // 直连数据库再确认一次：连主表行都不存在，证明是整体回滚而不是只回滚关联
    let sessions = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM learning_sessions s
        JOIN users u ON u.id = s.owner_id
        WHERE u.email = $1
        "#,
    )
    .bind("session-owner@example.com")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sessions, 0);
}

#[sqlx::test]
async fn replacements_are_all_or_nothing_and_guarded(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-replace@example.com").await;

    let primary = create_project(&app, &token, "主项目").await;
    let related = create_project(&app, &token, "关联项目").await;
    let unrelated = create_project(&app, &token, "无关项目").await;
    let content_a = create_content(&app, &token, "内容 A").await;
    let content_b = create_content(&app, &token, "内容 B").await;
    let task = create_task(&app, &token, &primary, "主项目的任务").await;
    let related_task = create_task(&app, &token, &related, "关联项目的任务").await;
    let other_task = create_task(&app, &token, &unrelated, "无关项目的任务").await;

    let id = create_session(
        &app,
        &token,
        json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "primary_project_id": primary,
            "related_project_ids": [related],
            "contents": [{ "content_id": content_a, "contribution_type": "covered" }],
            "tasks": [{ "task_id": task, "contribution_minutes": 30 }]
        }),
    )
    .await;

    // 重复 content_id → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/contents"),
        Some(&token),
        Some(json!({ "items": [
            { "content_id": content_a, "contribution_type": "covered" },
            { "content_id": content_a, "contribution_type": "practiced" }
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 关联项目里出现主要项目 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/projects"),
        Some(&token),
        Some(json!({ "project_ids": [primary] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 分摊超过有效时长 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/tasks"),
        Some(&token),
        Some(json!({ "items": [{ "task_id": task, "contribution_minutes": 61 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 任务不属于活动关联的项目 → 422
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/tasks"),
        Some(&token),
        Some(json!({ "items": [
            { "task_id": task, "contribution_minutes": 20 },
            { "task_id": other_task, "contribution_minutes": 20 }
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(envelope.message.contains("任务"));

    // 关联项目的任务可以加入分摊（related 在项目集合内）
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/tasks"),
        Some(&token),
        Some(json!({ "items": [
            { "task_id": task, "contribution_minutes": 20 },
            { "task_id": related_task, "contribution_minutes": 20 }
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 移除关联项目会让它的任务失去归属 → 422（不静默删除分摊行）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/projects"),
        Some(&token),
        Some(json!({ "project_ids": [unrelated] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(envelope.message.contains("任务分摊"));

    // 上述失败都不应改动既有关系
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["related_project_ids"], json!([related]));
    assert_eq!(
        data["contents"][0]["content_id"].as_str().unwrap(),
        content_a
    );
    assert_eq!(data["tasks"].as_array().unwrap().len(), 2);

    // 成功替换：内容换 B、清空任务、再加一个无关项目
    let (status, replaced) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/contents"),
        Some(&token),
        Some(json!({ "items": [{ "content_id": content_b, "contribution_type": "primary" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replaced.data.unwrap()["total"], json!(1));

    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/tasks"),
        Some(&token),
        Some(json!({ "items": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, response) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/projects"),
        Some(&token),
        Some(json!({ "project_ids": [related, unrelated] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        response.data.unwrap()["project_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // 空数组 = 清空（幂等）
    let (status, response) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-sessions/{id}/projects"),
        Some(&token),
        Some(json!({ "project_ids": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response.data.unwrap()["project_ids"], json!([]));
}

#[sqlx::test]
async fn soft_delete_hides_session_but_keeps_associations_and_evidence(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-delete@example.com").await;

    let project = create_project(&app, &token, "项目").await;
    let related = create_project(&app, &token, "关联项目").await;
    let content = create_content(&app, &token, "内容").await;
    let task = create_task(&app, &token, &project, "任务").await;
    let id = create_session(
        &app,
        &token,
        json!({
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T10:00:00Z",
            "primary_project_id": project,
            "related_project_ids": [related],
            "contents": [{ "content_id": content, "contribution_type": "covered" }],
            "tasks": [{ "task_id": task, "contribution_minutes": 15 }]
        }),
    )
    .await;

    // 证据挂在活动上
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({
            "session_id": id,
            "evidence_type": "note",
            "title": "错题摘录",
            "description": "第 3 题算错了收敛半径"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let evidence_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    // 软删
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 详情 404、列表排除
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, list) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-sessions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(list.data.unwrap()["total"], json!(0));

    // 重复删除幂等 200
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 软删是 UPDATE：三类关联行与证据都还在
    let session_uuid = Uuid::parse_str(&id).unwrap();
    for table in ["session_contents", "session_projects", "task_sessions"] {
        let count = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {table} WHERE session_id = $1"
        )))
        .bind(session_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "{table} 的关联行应随软删保留");
    }

    // 证据独立存在，且仍指向已软删的活动
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/evidence/{evidence_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.unwrap()["session_id"].as_str().unwrap(),
        id.as_str()
    );

    // 已软删的活动不能再挂新证据
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({
            "session_id": id,
            "evidence_type": "note",
            "description": "补一条"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn evidence_type_and_payload_guards(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "evidence-guard@example.com").await;
    let other = register_user(&app, "evidence-other@example.com").await;

    // note 缺 description → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({ "evidence_type": "note" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // link 缺 external_url → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({ "evidence_type": "link", "title": "题解" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 非 http(s) 链接 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({ "evidence_type": "link", "external_url": "javascript:alert(1)" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // v1 未开放的类型 → 422（不是 500）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({ "evidence_type": "image", "description": "截图" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(envelope.message.contains("note"));

    // 三类合法证据
    let mut ids = Vec::new();
    for body in [
        json!({ "evidence_type": "note", "description": "函数极限的定义" }),
        json!({ "evidence_type": "test_result", "description": "10 题错 2 题" }),
        json!({ "evidence_type": "link", "external_url": "https://example.com/solution" }),
    ] {
        let (status, envelope) = send_json::<Value>(
            &app,
            Method::POST,
            "/api/v1/evidence",
            Some(&token),
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        ids.push(envelope.data.unwrap()["id"].as_str().unwrap().to_string());
    }

    // 列表按 created_at DESC；别人的证据既看不到也删不掉
    let (_, list) =
        send_json::<Value>(&app, Method::GET, "/api/v1/evidence", Some(&token), None).await;
    assert_eq!(list.data.unwrap()["total"], json!(3));

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/evidence/{}", ids[0]),
        Some(&other),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/evidence/{}", ids[0]),
        Some(&other),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 本人删除成功；物理删除后重复删为 404（无法区分「删过」与「从未存在」）
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/evidence/{}", ids[0]),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/evidence/{}", ids[0]),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn patch_is_three_state_and_revalidates_duration(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-patch@example.com").await;

    let project = create_project(&app, &token, "项目").await;
    let another = create_project(&app, &token, "另一个项目").await;
    let id = create_session(
        &app,
        &token,
        json!({
            "title": "原标题",
            "started_at": "2026-11-28T09:00:00Z",
            "ended_at": "2026-11-28T11:00:00Z",
            "duration_minutes": 90,
            "primary_project_id": project,
            "summary": "原摘要"
        }),
    )
    .await;

    // 三态：显式 null 清空、缺失不变
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({ "summary": null, "title": "新标题" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["title"], "新标题");
    assert!(data["summary"].is_null());

    // 时长超过（新）区间 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({ "ended_at": "2026-11-28T10:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 区间反向 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({ "started_at": "2026-11-28T12:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 同时改区间与时长 → 成功，duration 跟着下界走
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({
            "ended_at": "2026-11-28T10:00:00Z",
            "duration_minutes": 60
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["duration_minutes"], json!(60));

    // 换主要项目
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({ "primary_project_id": another })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.unwrap()["primary_project_id"]
            .as_str()
            .unwrap(),
        another.as_str()
    );

    // 清空主要项目（项目解挂的客户端形态）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&token),
        Some(json!({ "primary_project_id": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(envelope.data.unwrap()["primary_project_id"].is_null());
}

#[sqlx::test]
async fn cross_user_access_returns_not_found(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let owner = register_user(&app, "session-cross-owner@example.com").await;
    let other = register_user(&app, "session-cross-other@example.com").await;

    let id = create_session(
        &app,
        &owner,
        hour_session("2026-11-28T09:00:00Z", "2026-11-28T10:00:00Z"),
    )
    .await;

    for (method, path, body) in [
        (Method::GET, format!("/api/v1/learning-sessions/{id}"), None),
        (
            Method::PATCH,
            format!("/api/v1/learning-sessions/{id}"),
            Some(json!({ "title": "越权改" })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/learning-sessions/{id}"),
            None,
        ),
        (
            Method::PUT,
            format!("/api/v1/learning-sessions/{id}/contents"),
            Some(json!({ "items": [] })),
        ),
        (
            Method::PUT,
            format!("/api/v1/learning-sessions/{id}/projects"),
            Some(json!({ "project_ids": [] })),
        ),
        (
            Method::PUT,
            format!("/api/v1/learning-sessions/{id}/tasks"),
            Some(json!({ "items": [] })),
        ),
    ] {
        let (status, _) = send_json::<Value>(&app, method.clone(), &path, Some(&other), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path} 应 404");
    }

    // 越权操作后记录仍完整可见
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-sessions/{id}"),
        Some(&owner),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn filters_narrow_by_project_content_type_and_range(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "session-filter@example.com").await;

    let project = create_project(&app, &token, "项目").await;
    let content = create_content(&app, &token, "内容").await;

    let morning = create_session(
        &app,
        &token,
        json!({
            "activity_type": "theory",
            "started_at": "2026-11-28T01:00:00Z",
            "ended_at": "2026-11-28T02:00:00Z",
            "primary_project_id": project,
            "contents": [{ "content_id": content, "contribution_type": "covered" }]
        }),
    )
    .await;

    let evening = create_session(
        &app,
        &token,
        json!({
            "activity_type": "review",
            "started_at": "2026-11-28T12:00:00Z",
            "ended_at": "2026-11-28T13:00:00Z"
        }),
    )
    .await;

    let by_project = format!("primary_project_id={project}");
    let by_content = format!("content_id={content}");

    let cases = [
        ("activity_type=theory", 1, morning.as_str()),
        ("activity_type=review", 1, evening.as_str()),
        (by_project.as_str(), 1, morning.as_str()),
        (by_content.as_str(), 1, morning.as_str()),
        ("from=2026-11-28T06:00:00Z", 1, evening.as_str()),
        ("to=2026-11-28T06:00:00Z", 1, morning.as_str()),
    ];

    for (query, expected_total, expected_first) in cases {
        let (status, envelope) = send_json::<Value>(
            &app,
            Method::GET,
            &format!("/api/v1/learning-sessions?{query}"),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{query} 应成功");
        let data = envelope.data.unwrap();
        assert_eq!(
            data["total"],
            json!(expected_total),
            "{query} 的 total 不符"
        );
        assert_eq!(data["items"][0]["id"].as_str().unwrap(), expected_first);
    }

    // 非法 activity_type 走统一信封的 422
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-sessions?activity_type=study",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(envelope.message.contains("activity_type"));
}
