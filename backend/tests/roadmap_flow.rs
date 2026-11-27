//! M04 规划域集成测试：目标层级、GoalStage 整体替换、路线版本与确认、阶段四动作。

mod support;

use axum::Router;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use support::{register_user, send_json};

async fn create_project(app: &Router, token: &str, name: &str) -> String {
    let (_, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/projects",
        Some(token),
        Some(json!({ "name": name })),
    )
    .await;
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

async fn create_goal(app: &Router, token: &str, pid: &str, body: Value) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/goals"),
        Some(token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建目标应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

/// 创建路线并返回 (id, version)
async fn create_roadmap(app: &Router, token: &str, pid: &str) -> (String, i64) {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps"),
        Some(token),
        Some(json!({ "source": "user", "change_summary": "首版" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建路线应成功");
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "draft", "新路线初始状态应为 draft");
    (
        data["id"].as_str().unwrap().to_string(),
        data["version"].as_i64().unwrap(),
    )
}

async fn create_stage(app: &Router, token: &str, pid: &str, rid: &str, name: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages"),
        Some(token),
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建阶段应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn goal_hierarchy_limits_to_two_levels(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "goal-level@example.com").await;
    let pid = create_project(&app, &token, "目标层级").await;

    let primary = create_goal(&app, &token, &pid, json!({ "title": "主目标" })).await;
    let child = create_goal(
        &app,
        &token,
        &pid,
        json!({ "title": "子目标", "parent_goal_id": primary }),
    )
    .await;

    // 子目标不能再作为父目标（两层上限）
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/goals"),
        Some(&token),
        Some(json!({ "title": "孙目标", "parent_goal_id": child })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 已有主目标时再建主目标 → 409（保守语义，冲突处置方式待决策）
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/goals"),
        Some(&token),
        Some(json!({ "title": "第二个主目标" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn goal_delete_cascades_child_goals_with_confirmation(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "goal-cascade@example.com").await;
    let pid = create_project(&app, &token, "目标级联").await;

    let primary = create_goal(&app, &token, &pid, json!({ "title": "主目标" })).await;
    let child = create_goal(
        &app,
        &token,
        &pid,
        json!({ "title": "子目标", "parent_goal_id": primary }),
    )
    .await;

    let base = format!("/api/v1/projects/{pid}/goals/{primary}");

    // 未带 cascade → 409 且带影响面
    let (status, body) = send_json::<Value>(&app, Method::DELETE, &base, Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body.data.unwrap()["impact"]["goals"],
        1,
        "影响面应给出子目标数"
    );

    // 带 cascade → 子目标一并删除
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("{base}?cascade=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/goals/{child}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "子目标应随主目标删除");
}

#[sqlx::test]
async fn goal_stage_links_are_replaced_atomically(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "goal-links@example.com").await;
    let pid = create_project(&app, &token, "目标关联").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let s1 = create_stage(&app, &token, &pid, &rid, "阶段一").await;
    let s2 = create_stage(&app, &token, &pid, &rid, "阶段二").await;
    let gid = create_goal(&app, &token, &pid, json!({ "title": "目标" })).await;

    let url = format!("/api/v1/projects/{pid}/goals/{gid}/stages");

    // 初次替换：两条关联
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &url,
        Some(&token),
        Some(json!({ "links": [
            { "stage_id": s1, "contribution": "primary" },
            { "stage_id": s2, "contribution": "supporting" }
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["items"].as_array().unwrap().len(), 2);

    // 重复 stage → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &url,
        Some(&token),
        Some(json!({ "links": [
            { "stage_id": s1, "contribution": "primary" },
            { "stage_id": s1, "contribution": "supporting" }
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 跨项目 stage → 422
    let other_pid = create_project(&app, &token, "另一个项目").await;
    let (other_rid, _) = create_roadmap(&app, &token, &other_pid).await;
    let foreign_stage = create_stage(&app, &token, &other_pid, &other_rid, "外来阶段").await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &url,
        Some(&token),
        Some(json!({ "links": [{ "stage_id": foreign_stage, "contribution": "primary" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 空数组清空
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &url,
        Some(&token),
        Some(json!({ "links": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        envelope.data.unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn confirming_roadmap_supersedes_previous_active(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "roadmap-confirm@example.com").await;
    let pid = create_project(&app, &token, "路线确认").await;

    let (r1, v1) = create_roadmap(&app, &token, &pid).await;
    assert_eq!(v1, 1, "首版版本号应为 1");
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{r1}/confirm"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "active");
    assert!(data["confirmed_at"].is_string());

    // 已 active 的路线再次 confirm → 409
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{r1}/confirm"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // 第二版确认 → 旧版 superseded
    let (r2, v2) = create_roadmap(&app, &token, &pid).await;
    assert_eq!(v2, 2, "版本号应按 MAX(version)+1 递增");
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{r2}/confirm"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/roadmaps/{r1}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["status"], "superseded");

    // superseded 只读：PATCH 409
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/projects/{pid}/roadmaps/{r1}"),
        Some(&token),
        Some(json!({ "change_summary": "改历史" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn stage_actions_require_active_roadmap_and_fixed_edges(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "stage-actions@example.com").await;
    let pid = create_project(&app, &token, "阶段动作").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "阶段一").await;

    let action_url =
        |action: &str| format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages/{sid}/{action}");

    // draft 路线上做动作 → 409（准入要求 active）
    let (status, _) =
        send_json::<Value>(&app, Method::POST, &action_url("start"), Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/confirm"),
        Some(&token),
        None,
    )
    .await;

    // pending 不能直接 complete
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &action_url("complete"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "pending 不可直达 completed");

    // pending 上 reactivate → 409（源态集不含 pending）
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &action_url("reactivate"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // start → active
    let (status, envelope) =
        send_json::<Value>(&app, Method::POST, &action_url("start"), Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["status"], "active");
    assert!(data["started_at"].is_string());

    // active 上重复 start → 200 幂等，started_at 不被刷新
    let (status, envelope) =
        send_json::<Value>(&app, Method::POST, &action_url("start"), Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["started_at"], data["started_at"]);

    // complete → completed
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &action_url("complete"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let completed = envelope.data.unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["completion_method"], "user");
    assert!(completed["completed_at"].is_string());

    // completed 上重复 complete → 200 幂等，completed_at 不刷新
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &action_url("complete"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        envelope.data.unwrap()["completed_at"],
        completed["completed_at"]
    );

    // completed 上 skip → 409（终态之间不直转）
    let (status, _) =
        send_json::<Value>(&app, Method::POST, &action_url("skip"), Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // reactivate → active，清空 completed_at，started_at 保持
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &action_url("reactivate"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let reactivated = envelope.data.unwrap();
    assert_eq!(reactivated["status"], "active");
    assert!(
        reactivated["completed_at"].is_null(),
        "离开 completed 应清空"
    );
    assert_eq!(
        reactivated["started_at"], data["started_at"],
        "started_at 不变"
    );
}

#[sqlx::test]
async fn stage_delete_requires_draft_roadmap_and_detaches_tasks(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "stage-delete@example.com").await;
    let pid = create_project(&app, &token, "阶段删除").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "待删阶段").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/milestones"),
        Some(&token),
        Some(json!({ "stage_id": sid, "title": "里程碑" })),
    )
    .await;
    assert!(envelope.data.is_some());

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "挂在阶段上的任务", "stage_id": sid })),
    )
    .await;
    let task_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    let url = format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages/{sid}");

    // 未带 cascade → 409 带影响面
    let (status, body) = send_json::<Value>(&app, Method::DELETE, &url, Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let impact = body.data.unwrap();
    assert_eq!(impact["impact"]["milestones"], 1);
    assert_eq!(impact["impact"]["tasks"], 1);

    // 带 cascade → 删除，任务解挂但保留
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("{url}?cascade=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks/{task_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "任务不随上层结构消失");
    let task = envelope.data.unwrap();
    assert!(task["stage_id"].is_null(), "stage_id 应被解挂");
    assert!(task["milestone_id"].is_null(), "milestone_id 应被解挂");
}

#[sqlx::test]
async fn stage_delete_on_active_roadmap_is_rejected(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "stage-active-del@example.com").await;
    let pid = create_project(&app, &token, "已确认路线").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "承诺阶段").await;

    send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/confirm"),
        Some(&token),
        None,
    )
    .await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages/{sid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "已确认结构不提供删除");
}

#[sqlx::test]
async fn milestone_delete_only_clears_milestone_id(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "milestone-del@example.com").await;
    let pid = create_project(&app, &token, "里程碑删除").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "阶段").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/milestones"),
        Some(&token),
        Some(json!({ "stage_id": sid, "title": "里程碑" })),
    )
    .await;
    let mid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "任务", "stage_id": sid, "milestone_id": mid })),
    )
    .await;
    let task_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/projects/{pid}/milestones/{mid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks/{task_id}"),
        Some(&token),
        None,
    )
    .await;
    let task = envelope.data.unwrap();
    assert!(task["milestone_id"].is_null());
    assert_eq!(task["stage_id"], sid.as_str(), "stage_id 应保留");
}

#[sqlx::test]
async fn milestone_status_follows_unified_rules(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "milestone-status@example.com").await;
    let pid = create_project(&app, &token, "里程碑状态").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "阶段").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/milestones"),
        Some(&token),
        Some(json!({ "stage_id": sid, "title": "里程碑" })),
    )
    .await;
    let mid = envelope.data.unwrap()["id"].as_str().unwrap().to_string();
    let url = format!("/api/v1/projects/{pid}/milestones/{mid}");

    // pending → completed 允许跳过中间态
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
    assert_eq!(completed["status"], "completed");
    assert!(completed["completed_at"].is_string());

    // 同态提交幂等 200，completed_at 不刷新
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

    // completed → skipped 拒绝（终态之间不直转）
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "skipped" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // completed → pending 回退，清空 completed_at
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &url,
        Some(&token),
        Some(json!({ "status": "pending" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let back = envelope.data.unwrap();
    assert_eq!(back["status"], "pending");
    assert!(back["completed_at"].is_null());
}

#[sqlx::test]
async fn roadmap_delete_detaches_tasks_and_removes_children(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "roadmap-del@example.com").await;
    let pid = create_project(&app, &token, "路线删除").await;
    let (rid, _) = create_roadmap(&app, &token, &pid).await;
    let sid = create_stage(&app, &token, &pid, &rid, "阶段").await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/projects/{pid}/tasks"),
        Some(&token),
        Some(json!({ "title": "任务", "stage_id": sid })),
    )
    .await;
    let task_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    let url = format!("/api/v1/projects/{pid}/roadmaps/{rid}");

    // 未带 cascade → 409
    let (status, _) = send_json::<Value>(&app, Method::DELETE, &url, Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("{url}?cascade=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 阶段随之消失
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/roadmaps/{rid}/stages/{sid}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 任务保留并解挂
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/projects/{pid}/tasks/{task_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(envelope.data.unwrap()["stage_id"].is_null());
}
