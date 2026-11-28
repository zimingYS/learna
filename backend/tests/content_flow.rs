//! M03 内容域集成测试：内容本体与正文边界、树与图保护、标签词表、项目内容隔离。

mod support;

use axum::Router;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use support::{register_user, send_json};

async fn create_content(app: &Router, token: &str, body: Value) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建内容应成功: {envelope:?}");
    envelope.data.unwrap()["item"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

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

async fn create_tag(app: &Router, token: &str, name: &str) -> String {
    let (status, envelope) = send_json::<Value>(
        app,
        Method::POST,
        "/api/v1/content-tags",
        Some(token),
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建标签应成功");
    envelope.data.unwrap()["id"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn content_crud_archive_and_body_boundary(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-crud@example.com").await;

    let id = create_content(
        &app,
        &token,
        json!({
            "title": "极限的定义",
            "content_type": "knowledge_point",
            "description": "数列极限的 ε-N 定义",
            "body": "# 定义\n对任意 ε>0 ...",
            "body_format": "markdown",
            "source_url": "https://example.com/limits"
        }),
    )
    .await;

    // 详情是唯一返回 body 的端点，同时给出 depth
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["body"], "# 定义\n对任意 ε>0 ...");
    assert_eq!(data["body_format"], "markdown");
    assert_eq!(data["depth"], 0, "根节点深度为 0");
    assert_eq!(data["status"], "active");
    assert_eq!(data["created_source"], "user");

    // 列表不返回 body（ADR-009 §2）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-contents",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = envelope.data.unwrap();
    assert_eq!(list["total"], 1);
    assert!(
        list["items"][0].get("body").is_none(),
        "列表响应不得包含 body"
    );

    // 三态 PATCH：body 置空 → NULL
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-contents/{id}"),
        Some(&token),
        Some(json!({ "body": null, "title": "极限的定义（修订）" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["body"], Value::Null);
    assert_eq!(data["title"], "极限的定义（修订）");

    // 归档幂等：连续两次都 200
    for _ in 0..2 {
        let (status, _) = send_json::<Value>(
            &app,
            Method::DELETE,
            &format!("/api/v1/learning-contents/{id}"),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "归档应幂等成功");
    }

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["status"], "archived");
}

#[sqlx::test]
async fn metadata_field_is_rejected(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-metadata@example.com").await;

    // ADR-009 §3：metadata 在 v1 不可写，出现即 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(&token),
        Some(json!({
            "title": "带元数据的内容",
            "content_type": "knowledge_point",
            "metadata": { "part_of_speech": "noun" }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 非法出处协议同样拒绝
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(&token),
        Some(json!({
            "title": "非法出处",
            "content_type": "knowledge_point",
            "source_url": "ftp://example.com/a"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn similar_titles_coexist_and_hint_without_blocking(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-similar@example.com").await;

    create_content(
        &app,
        &token,
        json!({ "title": "极限的定义", "content_type": "knowledge_point" }),
    )
    .await;

    // 相似度超过 0.4 时带提示，但创建必定成功（ERD §3.1）
    // 注：pg_trgm 对中文按三元组切分，「极限定义」与「极限的定义」只有 0.375，
    // 因此用确实过阈值的标题验证提示通路（阈值本身是 D-24 的冻结值）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(&token),
        Some(json!({ "title": "极限的定义(修订)", "content_type": "knowledge_point" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "相似内容不阻断创建");

    let data = envelope.data.unwrap();
    let hints = data["similar_hints"].as_array().unwrap();
    assert!(!hints.is_empty(), "应返回相似内容提示");
    assert_eq!(hints[0]["title"], "极限的定义");
    assert!(hints[0]["similarity"].as_f64().unwrap() > 0.4);

    // 同类型才提示：换成技能点后不再命中
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/learning-contents",
        Some(&token),
        Some(json!({ "title": "极限的定义(修订)", "content_type": "skill_point" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        envelope.data.unwrap()["similar_hints"]
            .as_array()
            .unwrap()
            .is_empty(),
        "不同类型不参与相似提示"
    );
}

#[sqlx::test]
async fn content_tree_guards_cycle_depth_and_ownership(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-tree@example.com").await;
    let other_token = register_user(&app, "content-tree-other@example.com").await;

    let root = create_content(
        &app,
        &token,
        json!({ "title": "高等数学", "content_type": "knowledge_point" }),
    )
    .await;

    let child = create_content(
        &app,
        &token,
        json!({
            "title": "第一章 极限",
            "content_type": "knowledge_point",
            "parent_id": root
        }),
    )
    .await;

    let grandchild = create_content(
        &app,
        &token,
        json!({
            "title": "1.1 数列的极限",
            "content_type": "knowledge_point",
            "parent_id": child
        }),
    )
    .await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{grandchild}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["depth"], 2, "三层树的叶节点深度为 2");

    // 把根挂到自己的孙节点下 → 成环 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-contents/{root}"),
        Some(&token),
        Some(json!({ "parent_id": grandchild })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 自引用同样拒绝
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-contents/{root}"),
        Some(&token),
        Some(json!({ "parent_id": root })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 他人内容作为父节点 → 404（不区分「不存在」与「不是你的」）
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/learning-contents",
        Some(&other_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["total"], 0, "他人看不到这些内容");

    let foreign = create_content(
        &app,
        &other_token,
        json!({ "title": "别人的内容", "content_type": "knowledge_point" }),
    )
    .await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/learning-contents/{root}"),
        Some(&token),
        Some(json!({ "parent_id": foreign })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // root_only / parent_id 互斥
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents?root_only=true&parent_id={root}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn relations_are_single_row_bidirectional_and_cycle_safe(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-relation@example.com").await;

    let basic = create_content(
        &app,
        &token,
        json!({ "title": "函数极限", "content_type": "knowledge_point" }),
    )
    .await;
    let advanced = create_content(
        &app,
        &token,
        json!({ "title": "导数", "content_type": "knowledge_point" }),
    )
    .await;

    // 前置关系：函数极限 → 导数
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/learning-contents/{basic}/relations"),
        Some(&token),
        Some(json!({ "target_content_id": advanced, "relation_type": "prerequisite" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let relation_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    // 重复提交幂等：不新增行
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/learning-contents/{basic}/relations"),
        Some(&token),
        Some(json!({ "target_content_id": advanced, "relation_type": "prerequisite" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{basic}/relations"),
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1, "重复关系不新增行");
    assert_eq!(data["items"][0]["direction"], "outbound");

    // 反向查询：目标侧看到入边
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{advanced}/relations?direction=inbound"),
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1);
    assert_eq!(data["items"][0]["direction"], "inbound");

    // 反向建前置 → 成环 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/learning-contents/{advanced}/relations"),
        Some(&token),
        Some(json!({ "target_content_id": basic, "relation_type": "prerequisite" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 自引用 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/learning-contents/{basic}/relations"),
        Some(&token),
        Some(json!({ "target_content_id": basic, "relation_type": "related" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 删除关系；再从另一端删同一 id 也应 404（已不存在）
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{basic}/relations/{relation_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{basic}/relations/{relation_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn relations_reject_cross_user_targets(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "relation-owner@example.com").await;
    let other_token = register_user(&app, "relation-other@example.com").await;

    let mine = create_content(
        &app,
        &token,
        json!({ "title": "我的内容", "content_type": "knowledge_point" }),
    )
    .await;
    let foreign = create_content(
        &app,
        &other_token,
        json!({ "title": "他人内容", "content_type": "knowledge_point" }),
    )
    .await;

    // 两端都查 owner，跨用户不可达 → 404
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        &format!("/api/v1/learning-contents/{mine}/relations"),
        Some(&token),
        Some(json!({ "target_content_id": foreign, "relation_type": "related" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn tags_vocabulary_and_replacement(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "content-tags@example.com").await;
    let other_token = register_user(&app, "content-tags-other@example.com").await;

    let content = create_content(
        &app,
        &token,
        json!({ "title": "打标对象", "content_type": "knowledge_point" }),
    )
    .await;

    let tag_a = create_tag(&app, &token, "重点").await;
    let tag_b = create_tag(&app, &token, "易错").await;

    // 重名 409，不静默返回既有标签
    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/content-tags",
        Some(&token),
        Some(json!({ "name": "重点" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // 改名撞已有名 → 409
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/content-tags/{tag_b}"),
        Some(&token),
        Some(json!({ "name": "重点" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // 整体替换
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-contents/{content}/tags"),
        Some(&token),
        Some(json!({ "tag_ids": [tag_a, tag_b] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(envelope.data.unwrap()["total"], 2);

    // 幂等：同一集合再提交一次结果一致
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-contents/{content}/tags"),
        Some(&token),
        Some(json!({ "tag_ids": [tag_b] })),
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 1);
    assert_eq!(data["items"][0]["name"], "易错");

    // 重复 id → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-contents/{content}/tags"),
        Some(&token),
        Some(json!({ "tag_ids": [tag_a, tag_a] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 他人标签 → 404
    let foreign_tag = create_tag(&app, &other_token, "别人的标签").await;
    let (status, _) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-contents/{content}/tags"),
        Some(&token),
        Some(json!({ "tag_ids": [foreign_tag] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 清空
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::PUT,
        &format!("/api/v1/learning-contents/{content}/tags"),
        Some(&token),
        Some(json!({ "tag_ids": [] })),
    )
    .await;
    assert_eq!(envelope.data.unwrap()["total"], 0);
}

#[sqlx::test]
async fn project_content_attach_is_idempotent_and_isolated(pool: PgPool) {
    let app = support::build_app(pool.clone());
    let token = register_user(&app, "project-content@example.com").await;

    let project_a = create_project(&app, &token, "项目 A").await;
    let project_b = create_project(&app, &token, "项目 B").await;
    let content = create_content(
        &app,
        &token,
        json!({ "title": "共享内容", "content_type": "knowledge_point" }),
    )
    .await;

    let attach = |pid: String, cid: String| {
        let token = token.clone();
        let app = app.clone();
        async move {
            let (status, envelope) = send_json::<Value>(
                &app,
                Method::POST,
                "/api/v1/project-contents",
                Some(&token),
                Some(json!({ "project_id": pid, "content_id": cid })),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            envelope.data.unwrap()
        }
    };

    let link_a = attach(project_a.clone(), content.clone()).await;
    assert_eq!(link_a["current_status"], "not_started");
    assert_eq!(link_a["target_mastery_level"], 0);
    assert_eq!(link_a["priority"], 3);

    // 重复关联幂等：返回同一行
    let again = attach(project_a.clone(), content.clone()).await;
    assert_eq!(again["id"], link_a["id"]);

    let link_b = attach(project_b.clone(), content.clone()).await;

    // 项目 A 置为 achieved（模拟 M06 评价结果写入）
    sqlx::query(
        "UPDATE project_contents SET current_status = 'achieved', achieved_at = now() WHERE id = $1",
    )
    .bind(uuid::Uuid::parse_str(link_a["id"].as_str().unwrap()).unwrap())
    .execute(&pool)
    .await
    .expect("直接写入 achieved 用于验证项目级隔离");

    // 同一内容在两个项目状态互不干扰（本域最重要的不变量）
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/project-contents?content_id={content}"),
        Some(&token),
        None,
    )
    .await;
    let data = envelope.data.unwrap();
    assert_eq!(data["total"], 2);
    let statuses: Vec<&str> = data["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["current_status"].as_str().unwrap())
        .collect();
    assert!(statuses.contains(&"achieved"));
    assert!(statuses.contains(&"not_started"));
    assert_ne!(
        link_a["id"], link_b["id"],
        "两个项目各自持有关联行，不共享状态"
    );

    // 按项目过滤只看本项目
    let (_, envelope) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/project-contents?project_id={project_b}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(envelope.data.unwrap()["total"], 1);

    // 两个过滤参数都不给 → 422
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        "/api/v1/project-contents",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn project_content_status_guard_and_purge_preconditions(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "project-content-status@example.com").await;

    let project = create_project(&app, &token, "状态守卫").await;
    let content = create_content(
        &app,
        &token,
        json!({ "title": "状态内容", "content_type": "skill_point" }),
    )
    .await;

    let (_, envelope) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/project-contents",
        Some(&token),
        Some(json!({ "project_id": project, "content_id": content })),
    )
    .await;
    let link_id = envelope.data.unwrap()["id"].as_str().unwrap().to_string();

    // achieved 只能由 M06 写入（D-27）
    let (status, _) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/project-contents/{link_id}"),
        Some(&token),
        Some(json!({ "current_status": "achieved" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // 其余状态可写
    let (status, envelope) = send_json::<Value>(
        &app,
        Method::PATCH,
        &format!("/api/v1/project-contents/{link_id}"),
        Some(&token),
        Some(json!({ "current_status": "learning", "priority": 5 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = envelope.data.unwrap();
    assert_eq!(data["current_status"], "learning");
    assert_eq!(data["priority"], 5);

    // 未归档直接 purge → 409
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{content}?purge=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // 归档后仍有项目关联 → 409 且带影响面
    send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{content}"),
        Some(&token),
        None,
    )
    .await;

    let (status, envelope) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{content}?purge=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let data = envelope.data.expect("冲突应携带影响面");
    assert_eq!(data["impact"]["project_contents"], 1);

    // 解除关联后可彻底删除
    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/project-contents/{link_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{content}?purge=true"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/learning-contents/{content}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "彻底删除后不可见");
}

#[sqlx::test]
async fn archived_content_cannot_join_project(pool: PgPool) {
    let app = support::build_app(pool);
    let token = register_user(&app, "archived-attach@example.com").await;

    let project = create_project(&app, &token, "归档校验").await;
    let content = create_content(
        &app,
        &token,
        json!({ "title": "将归档的内容", "content_type": "language_item" }),
    )
    .await;

    send_json::<Value>(
        &app,
        Method::DELETE,
        &format!("/api/v1/learning-contents/{content}"),
        Some(&token),
        None,
    )
    .await;

    let (status, _) = send_json::<Value>(
        &app,
        Method::POST,
        "/api/v1/project-contents",
        Some(&token),
        Some(json!({ "project_id": project, "content_id": content })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "归档内容不再接受新关联");
}

#[sqlx::test]
async fn cross_user_access_returns_not_found(pool: PgPool) {
    let app = support::build_app(pool);
    let owner = register_user(&app, "owner@example.com").await;
    let intruder = register_user(&app, "intruder@example.com").await;

    let content = create_content(
        &app,
        &owner,
        json!({ "title": "私密内容", "content_type": "knowledge_point" }),
    )
    .await;
    let project = create_project(&app, &owner, "私密项目").await;

    for (method, uri) in [
        (Method::GET, format!("/api/v1/learning-contents/{content}")),
        (
            Method::PATCH,
            format!("/api/v1/learning-contents/{content}"),
        ),
        (
            Method::DELETE,
            format!("/api/v1/learning-contents/{content}"),
        ),
        (
            Method::GET,
            format!("/api/v1/learning-contents/{content}/relations"),
        ),
        (
            Method::GET,
            format!("/api/v1/learning-contents/{content}/tags"),
        ),
    ] {
        let body = if method == Method::PATCH {
            Some(json!({ "title": "改别人的" }))
        } else {
            None
        };
        let (status, _) =
            send_json::<Value>(&app, method.clone(), &uri, Some(&intruder), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {uri} 应返回 404");
    }

    // 他人项目下的关联列表同样 404
    let (status, _) = send_json::<Value>(
        &app,
        Method::GET,
        &format!("/api/v1/project-contents?project_id={project}"),
        Some(&intruder),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
