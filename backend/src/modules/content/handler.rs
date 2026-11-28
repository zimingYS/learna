//! M03 内容域 HTTP 层：内容本体 / 关系 / 标签词表。
//!
//! 扁平资源用顶层路径（内容跨项目复用，不能挂在 `/projects/{id}` 下）；
//! 关系与打标以内容为锚点，目标内容只出现在请求体。

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::modules::content::dto::{
    ContentCreateData, ContentDetailItem, ContentListData, ContentQuery, ContentTagItem,
    ContentTagListData, ContentTagQuery, CreateContentRequest, CreateContentTagRequest,
    CreateRelationRequest, PurgeQuery, RelationItem, RelationListData, RelationQuery,
    SetContentTagsRequest, UpdateContentRequest, UpdateContentTagRequest,
};
use crate::modules::content::{relation, service, tag};
use crate::shared::types::UserId;
use crate::shared::{ApiReply, RequestId};

// ---------------------------------------------------------------------------
// 内容本体
// ---------------------------------------------------------------------------

pub async fn list_contents(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<ContentQuery>,
) -> ApiReply<ContentListData> {
    ApiReply(service::list(&state.db, user_id.0, query).await, req_id)
}

pub async fn create_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateContentRequest>,
) -> ApiReply<ContentCreateData> {
    ApiReply(service::create(&state.db, user_id.0, payload).await, req_id)
}

pub async fn detail_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
) -> ApiReply<ContentDetailItem> {
    ApiReply(
        service::detail(&state.db, user_id.0, content_id).await,
        req_id,
    )
}

pub async fn update_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
    Json(payload): Json<UpdateContentRequest>,
) -> ApiReply<ContentDetailItem> {
    ApiReply(
        service::update(&state.db, user_id.0, content_id, payload).await,
        req_id,
    )
}

pub async fn delete_content(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
    Query(query): Query<PurgeQuery>,
) -> ApiReply<()> {
    ApiReply(
        service::delete(&state.db, user_id.0, content_id, query.purge).await,
        req_id,
    )
}

// ---------------------------------------------------------------------------
// 内容关系
// ---------------------------------------------------------------------------

pub async fn list_relations(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
    Query(query): Query<RelationQuery>,
) -> ApiReply<RelationListData> {
    ApiReply(
        relation::list(&state.db, user_id.0, content_id, query).await,
        req_id,
    )
}

pub async fn create_relation(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
    Json(payload): Json<CreateRelationRequest>,
) -> ApiReply<RelationItem> {
    ApiReply(
        relation::create(&state.db, user_id.0, content_id, payload).await,
        req_id,
    )
}

pub async fn delete_relation(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path((content_id, relation_id)): Path<(Uuid, Uuid)>,
) -> ApiReply<()> {
    ApiReply(
        relation::delete(&state.db, user_id.0, content_id, relation_id).await,
        req_id,
    )
}

// ---------------------------------------------------------------------------
// 标签
// ---------------------------------------------------------------------------

pub async fn list_content_tags(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
) -> ApiReply<ContentTagListData> {
    ApiReply(
        tag::list_content_tags(&state.db, user_id.0, content_id).await,
        req_id,
    )
}

pub async fn set_content_tags(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(content_id): Path<Uuid>,
    Json(payload): Json<SetContentTagsRequest>,
) -> ApiReply<ContentTagListData> {
    ApiReply(
        tag::set_content_tags(&state.db, user_id.0, content_id, payload).await,
        req_id,
    )
}

pub async fn list_tags(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Query(query): Query<ContentTagQuery>,
) -> ApiReply<ContentTagListData> {
    ApiReply(tag::list_tags(&state.db, user_id.0, query).await, req_id)
}

pub async fn create_tag(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<CreateContentTagRequest>,
) -> ApiReply<ContentTagItem> {
    ApiReply(tag::create_tag(&state.db, user_id.0, payload).await, req_id)
}

pub async fn update_tag(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(tag_id): Path<Uuid>,
    Json(payload): Json<UpdateContentTagRequest>,
) -> ApiReply<ContentTagItem> {
    ApiReply(
        tag::update_tag(&state.db, user_id.0, tag_id, payload).await,
        req_id,
    )
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Extension(req_id): Extension<RequestId>,
    Path(tag_id): Path<Uuid>,
) -> ApiReply<()> {
    ApiReply(tag::delete_tag(&state.db, user_id.0, tag_id).await, req_id)
}
