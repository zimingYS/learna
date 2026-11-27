//! M04 任务集成测试：软删除、状态迁移、依赖环检测与阶段一致性。

mod support;

use axum::Router;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use support::{register_user, send_json};

async fn create_project(app: &Router, token: &str) -> String {
    let (_, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/projects",
        Some(token),
        Some(json!({ "name": "任务项目" })),
    )
    .await;
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

async fn create_task(app: &Router, token: &str, pid: &str, body: Value) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建任务应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn create_applies_defaults_and_validates_priority(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "task-create@example.com").await;
    let pid = create_project(&app, &token).await;

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "  写测试  " })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["title"], "写测试");
    assert_eq!(data["status"], "todo");
    assert_eq!(data["priority"], 3);
    assert_eq!(data["created_source"], "user");
    assert!(data["milestone_id"].is_null());
    assert!(data["stage_id"].is_null());

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "越界优先级", "priority": 9 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn task_status_follows_unified_rules(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "task-status@example.com").await;
    let pid = create_project(&app, &token).await;
    let tid = create_task(&app, &token, &pid, json!({ "title": "任务" })).await;

    let url = format!("/api/v1/projects/{pid}/tasks/{tid}");

    // todo → completed 允许跳过中间态（一次打勾）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "completed" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let completed = envelope.data.unwrap();
    assert!(completed["completed_at"].is_string());

    // 同态幂等 200，completed_at 不刷新
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "completed" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.unwrap()["completed_at"],
        completed["completed_at"]
    );

    // completed → cancelled 拒绝
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "cancelled" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // completed → todo 回退，清空 completed_at
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "todo" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(envelope.data.unwrap()["completed_at"].is_null());
}

#[sqlx::test]
async fn soft_deleted_task_disappears_from_list_and_detail(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "task-softdel@example.com").await;
    let pid = create_project(&app, &token).await;
    let tid = create_task(&app, &token, &pid, json!({ "title": "将删除" })).await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/tasks/{tid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks/{tid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["total"], 0);

    // 软删行仍在库中（数据留存），只是不参与查询
    let remaining = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM learning_tasks WHERE project_id = $1::uuid AND deleted_at IS NOT NULL",
    )
    .bind(&pid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(remaining, 1, "软删应保留行");
}

#[sqlx::test]
async fn soft_delete_removes_dependency_rows(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "task-dep-clean@example.com").await;
    let pid = create_project(&app, &token).await;
    let a = create_task(&app, &token, &pid, json!({ "title": "前置" })).await;
    let b = create_task(&app, &token, &pid, json!({ "title": "后继" })).await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks/{b}/dependencies"),
        Some(&token),
        Some(json!({ "depends_on_task_id": a })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 软删前置任务：依赖行必须被物理清理（软删不触发 ON DELETE CASCADE）
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks/{a}/dependencies"),
        Some(&token),
        Some(json!({ "depends_on_task_id": b })),
    )
    .await;
    assert_eq!(
        envelope.code, "VALIDATION_ERROR",
        "b 依赖 a 已存在时，a 依赖 b 会成环"
    );

    send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/tasks/{a}"),
        Some(&token),
        None,
    )
    .await;

    let dangling = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM task_dependencies WHERE task_id = $1::uuid OR depends_on_task_id = $1::uuid",
    )
    .bind(&a)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(dangling, 0, "软删任务时依赖行应被物理删除");
}

#[sqlx::test]
async fn dependency_rejects_self_cross_project_and_cycles(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "task-dep@example.com").await;
    let pid = create_project(&app, &token).await;
    let other_pid = create_project(&app, &token).await;

    let a = create_task(&app, &token, &pid, json!({ "title": "A" })).await;
    let b = create_task(&app, &token, &pid, json!({ "title": "B" })).await;
    let c = create_task(&app, &token, &pid, json!({ "title": "C" })).await;
    let foreign = create_task(&app, &token, &other_pid, json!({ "title": "别的项目" })).await;

    let dep_url = |task: &str| format!("/api/v1/projects/{pid}/tasks/{task}/dependencies");

    // 自依赖 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &dep_url(&a),
        Some(&token),
        Some(json!({ "depends_on_task_id": a })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 跨项目 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &dep_url(&a),
        Some(&token),
        Some(json!({ "depends_on_task_id": foreign })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A 依赖 B，B 依赖 C
    for (task, dep) in [(&a, &b), (&b, &c)] {
        let (status, _) = send_json::<Value>(
            &app,
            Method::POST,
            &dep_url(task),
            Some(&token),
            Some(json!({ "depends_on_task_id": dep })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "A→B→C 应可建立");
    }

    // C 依赖 A → 成环 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &dep_url(&c),
        Some(&token),
        Some(json!({ "depends_on_task_id": a })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "长链成环应被拒绝");

    // 重复添加同一依赖 → 幂等 200
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &dep_url(&a),
        Some(&token),
        Some(json!({ "depends_on_task_id": b })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "重复依赖应幂等");

    // 删除依赖
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/tasks/{a}/dependencies/{b}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, envelope) =
        send_json::<Value>(&app, Method::GET, &dep_url(&a), Some(&token), None).await;
    assert!(
        envelope.data.unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn stage_and_milestone_must_belong_to_same_stage(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "task-consistency@example.com").await;
    let pid = create_project(&app, &token).await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps"),
        Some(&token),
        Some(json!({})),
    )
    .await;
    let rid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    let mut stages = Vec::new();
    for name in ["阶段一", "阶段二"] {
        let (_, envelope) = send_json::<Value>(
            &app,
            Method::POST,
            &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages"),
            Some(&token),
            Some(json!({ "name": name })),
        )
        .await;
        stages.push(envelope.data.unwrap()["id"].as_str().unwrap().to_string());
    }

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/milestones"),
        Some(&token),
        Some(json!({ "stage_id": stages[1], "title": "阶段二的里程碑" })),
    )
    .await;
    let mid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    // stage_id 指向阶段一、milestone 属阶段二 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "错配", "stage_id": stages[0], "milestone_id": mid })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 一致时成功
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "正确", "stage_id": stages[1], "milestone_id": mid })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn task_list_filters_and_orders_by_priority(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "task-list@example.com").await;
    let pid = create_project(&app, &token).await;

    create_task(&app, &token, &pid, json!({ "title": "低", "priority": 1 })).await;
    create_task(&app, &token, &pid, json!({ "title": "高", "priority": 5 })).await;
    let done = create_task(
        &app,
        &token,
        &pid,
        json!({ "title": "已完成", "priority": 3 }),
    )
    .await;

    send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}/tasks/{done}"),
        Some(&token),
        Some(json!({ "status": "completed" })),
    )
    .await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    let items = data["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["priority"], 5, "应按 priority DESC 排序");

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks?status=completed"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["total"], 1);
}
