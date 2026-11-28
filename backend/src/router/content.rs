//! M03 内容域路由：内容本体、标签词表、项目内容关联。
//!
//! 三组资源各有独立前缀（内容跨项目复用，不挂在项目下），因此由调用方分别 `nest`。

use axum::Router;
use axum::routing::{delete, get, patch};

use crate::app_state::AppState;
use crate::modules::content::handler;
use crate::modules::project_content::handler as project_content_handler;

/// `/learning-contents`：内容本体 + 关系 + 打标
pub fn contents_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handler::list_contents).post(handler::create_content),
        )
        .route(
            "/{content_id}",
            get(handler::detail_content)
                .patch(handler::update_content)
                .delete(handler::delete_content),
        )
        .route(
            "/{content_id}/relations",
            get(handler::list_relations).post(handler::create_relation),
        )
        .route(
            "/{content_id}/relations/{relation_id}",
            delete(handler::delete_relation),
        )
        .route(
            "/{content_id}/tags",
            get(handler::list_content_tags).put(handler::set_content_tags),
        )
}

/// `/content-tags`：用户私有标签词表
pub fn tags_router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list_tags).post(handler::create_tag))
        .route(
            "/{tag_id}",
            patch(handler::update_tag).delete(handler::delete_tag),
        )
}

/// `/project-contents`：内容在项目中的上下文
pub fn project_contents_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(project_content_handler::list_project_contents)
                .post(project_content_handler::create_project_content),
        )
        .route(
            "/{project_content_id}",
            get(project_content_handler::detail_project_content)
                .patch(project_content_handler::update_project_content)
                .delete(project_content_handler::delete_project_content),
        )
}
