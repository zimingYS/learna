//! M02 项目生命周期集成测试：归属隔离、状态机准入、三态 PATCH、级联删除与 purge。

mod support;

use axum::Router;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use support::{register_user, send_json, send_raw};

/// 创建项目并返回 id
async fn create_project(app: &Router, token: &str, name: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/projects",
        Some(token),
        Some(json!({ "name": name })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "创建项目应成功: {envelope:?}");
    envelope.data.unwrap()["id"]
        .as_str()
        .expect("项目 id 应为字符串")
        .to_string()
}

#[sqlx::test]
async fn create_applies_contract_defaults(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-default@example.com").await;

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({ "name": "  考研数学  " })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.expect("应返回项目");
    // D-19 默认值
    assert_eq!(data["project_type"], "study");
    assert_eq!(data["importance_level"], 3);
    assert_eq!(data["deadline_type"], "none");
    assert_eq!(data["status"], "draft");
    // 名称去空白
    assert_eq!(data["name"], "考研数学");
    assert!(data["deadline_at"].is_null());
}

#[sqlx::test]
async fn create_rejects_empty_name_and_deadline_mismatch(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-invalid@example.com").await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({ "name": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "空名称应 422");

    // deadline_type=none 却给 deadline_at
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({ "name": "错配", "deadline_at": "2027-12-01T00:00:00Z" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "none 带 deadline_at 应 422"
    );

    // deadline_type=soft 却没给 deadline_at
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({ "name": "缺期", "deadline_type": "soft" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "soft 缺 deadline_at 应 422"
    );
}

#[sqlx::test]
async fn lifecycle_actions_follow_the_matrix(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-lifecycle@example.com").await;
    let pid = create_project(&app, &token, "生命周期").await;

    let base = format!("/api/v1/projects/{pid}");

    // draft 不能 pause（未定义迁移）
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/pause"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // activate: draft → active
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/activate"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["status"], "active");

    // pause → resume
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/pause"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["status"], "paused");

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/resume"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["status"], "active");

    // D-01：暂停 → 已完成 开放
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/pause"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/complete"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["status"], "completed");

    // 已完成 → 暂停 拒绝（须先 reopen）
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/pause"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // reopen → active，写 reopened_at
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/reopen"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "active");
    assert!(data["reopened_at"].is_string(), "reopen 应写 reopened_at");

    // archive → archived，写 archived_at
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/archive"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "archived");
    assert!(data["archived_at"].is_string());

    // 已归档 → reopen 清空 archived_at
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/reopen"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "active");
    assert!(data["archived_at"].is_null(), "reopen 应清空 archived_at");
}

#[sqlx::test]
async fn list_excludes_archived_by_default(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-list@example.com").await;

    let active_id = create_project(&app, &token, "活跃项目").await;
    let archived_id = create_project(&app, &token, "归档项目").await;

    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{active_id}/activate"),
        Some(&token),
        None,
    )
    .await;
    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{archived_id}/activate"),
        Some(&token),
        None,
    )
    .await;
    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{archived_id}/archive"),
        Some(&token),
        None,
    )
    .await;

    let (status, envelope) =
        send_json::<Value>(&app, Method::GET, "/api/v1/projects", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1, "默认应排除已归档项目");

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/projects?include_archived=true",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["total"], 2);

    // 显式按 archived 过滤必须查得到：默认排除只作用于「未指定 status」的列表
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/projects?status=archived",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1, "status=archived 应能直接查到归档项目");
    assert_eq!(data["items"][0]["id"], archived_id);

    // 按 status 过滤其他状态时同样只返回该状态
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/projects?status=active",
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1);
    assert_eq!(data["items"][0]["id"], active_id);
}

#[sqlx::test]
async fn patch_supports_three_state_semantics(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-patch@example.com").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({ "name": "三态", "description": "初始描述" })),
    )
    .await;
    let pid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();
    let base = format!("/api/v1/projects/{pid}");

    // 字段缺失 → 不变
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &base,
        Some(&token),
        Some(json!({ "name": "改名" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["description"], "初始描述");

    // 显式 null → 置空
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &base,
        Some(&token),
        Some(json!({ "description": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        envelope.data.unwrap()["description"].is_null(),
        "null 应置空"
    );

    // 空对象 → 422（没有可更新的字段）
    let (status, _) =
        send_json::<Value>(&app, Method::PATCH, &base, Some(&token), Some(json!({}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn fixed_deadline_only_changes_through_extend_deadline(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-deadline@example.com").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/projects",
        Some(&token),
        Some(json!({
            "name": "固定期限",
            "deadline_type": "fixed",
            "deadline_at": "2027-12-20T00:00:00Z"
        })),
    )
    .await;
    let pid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();
    let base = format!("/api/v1/projects/{pid}");

    // PATCH 改期 → 422（D-04）
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &base,
        Some(&token),
        Some(json!({ "deadline_at": "2027-12-25T00:00:00Z" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // extend-deadline 缺 reason → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/extend-deadline"),
        Some(&token),
        Some(json!({ "new_deadline_at": "2027-12-25T00:00:00Z", "reason": "  " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 早于当前 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/extend-deadline"),
        Some(&token),
        Some(json!({ "new_deadline_at": "2027-12-01T00:00:00Z", "reason": "提前" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 正常改期 → 200
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("{base}/extend-deadline"),
        Some(&token),
        Some(json!({ "new_deadline_at": "2027-12-25T00:00:00Z", "reason": "疫情延期" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.unwrap()["deadline_at"],
        "2027-12-25T00:00:00Z"
    );
}

#[sqlx::test]
async fn project_isolation_hides_other_users_projects(pool: PgPool) {
    let app = support::build_app(pool);
    let owner = register_user(&app, "proj-owner@example.com").await;
    let stranger = register_user(&app, "proj-stranger@example.com").await;

    let pid = create_project(&app, &owner, "私有项目").await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}"),
        Some(&stranger),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "越权应表现为 404");

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/activate"),
        Some(&stranger),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn draft_delete_cascades_but_non_draft_requires_purge(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-delete@example.com").await;

    let pid = create_project(&app, &token, "草稿删除").await;
    // 草稿下挂一条任务，验证级联
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "草稿任务" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "草稿删除应为物理删除");

    // 非草稿：DELETE 409，purge 需重输名称
    let pid2 = create_project(&app, &token, "已实施").await;
    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid2}/activate"),
        Some(&token),
        None,
    )
    .await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid2}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "非草稿 DELETE 应 409");

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid2}/purge"),
        Some(&token),
        Some(json!({ "confirm_name": "写错了" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "名称不符应 422");

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid2}/purge"),
        Some(&token),
        Some(json!({ "confirm_name": "已实施" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid2}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn deleting_achieved_goal_returns_conflict(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "goal-achieved@example.com").await;
    let pid = create_project(&app, &token, "目标删除").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/goals"),
        Some(&token),
        Some(json!({ "title": "主目标" })),
    )
    .await;
    let gid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    // active → achieved
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}/goals/{gid}"),
        Some(&token),
        Some(json!({ "status": "achieved" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // achieved 删除 → 409
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/goals/{gid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "成果目标禁止删除");

    // achieved → cancelled 拒绝
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}/goals/{gid}"),
        Some(&token),
        Some(json!({ "status": "cancelled" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "终态之间不直转");

    // achieved → active 允许
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}/goals/{gid}"),
        Some(&token),
        Some(json!({ "status": "active" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn archive_keeps_planning_children(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-archive@example.com").await;
    let pid = create_project(&app, &token, "归档保留子数据").await;

    // 铺满规划域：目标 / 路线 / 阶段 / 任务
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/goals"),
        Some(&token),
        Some(json!({ "title": "主目标" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, roadmap) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps"),
        Some(&token),
        Some(json!({ "change_summary": "初版" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rid = roadmap.data.unwrap()["id"].as_str().unwrap().to_string();

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages"),
        Some(&token),
        Some(json!({ "name": "第一阶段" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "任务一" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/archive"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.expect("归档应返回项目");
    assert_eq!(data["status"], "archived");
    assert!(!data["archived_at"].is_null(), "归档应写 archived_at");

    // 归档 ≠ 删除：子数据必须全部还在
    for path in ["goals", "roadmaps", "tasks"] {
        let (status, envelope) = send_json::<Value>(
            &app,
            Method::GET,
            &format!("/api/v1/projects/{pid}/{path}"),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "归档后 {path} 应可读");
        assert_eq!(
            envelope.data.expect("列表应有数据")["total"],
            1,
            "归档后 {path} 应保留 1 条"
        );
    }

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "归档后阶段应可读");
    assert_eq!(envelope.data.expect("列表应有数据")["total"], 1);
}

#[sqlx::test]
async fn purge_only_removes_the_target_project(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-purge-scope@example.com").await;

    let doomed = create_project(&app, &token, "待销毁").await;
    let kept = create_project(&app, &token, "应保留").await;

    for pid in [&doomed, &kept] {
        let (status, _) = send_json::<Value>(
            &app,
            Method::POST,
            &format!("/api/v1/projects/{pid}/goals"),
            Some(&token),
            Some(json!({ "title": "项目目标" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{doomed}/activate"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{doomed}/purge"),
        Some(&token),
        Some(json!({ "confirm_name": "待销毁" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{doomed}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 同用户的另一个项目及其子数据不受影响（级联范围不得越界）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{kept}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.expect("应返回项目")["name"], "应保留");

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{kept}/goals"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.expect("目标应保留")["total"],
        1,
        "purge 不得删除其他项目的目标"
    );
}

#[sqlx::test]
async fn patch_rejects_empty_body_and_out_of_range_importance(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-patch-invalid@example.com").await;
    let pid = create_project(&app, &token, "校验").await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}"),
        Some(&token),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "空 PATCH 应 422");

    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}"),
        Some(&token),
        Some(json!({ "importance_level": 9 })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "importance_level 越界应 422 而非 DB CHECK 的 500"
    );
}

#[sqlx::test]
async fn unknown_project_returns_404(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "proj-404@example.com").await;

    let (status, body) = send_raw(
        &app,
        Method::GET,
        "/api/v1/projects/00000000-0000-0000-0000-000000000000",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "NOT_FOUND");
    assert!(body["request_id"].is_string(), "失败响应也要带 request_id");
}
