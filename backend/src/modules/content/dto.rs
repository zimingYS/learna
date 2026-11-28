//! M03 内容域 DTO：学习内容 / 内容关系 / 标签词表。
//!
//! 契约来源：`openapi.yaml` 1.6.1 的 `ContentItem` / `ContentDetailItem` / `RelationItem` /
//! `ContentTagItem` 及其请求体。项目内容上下文见 `modules/project_content/dto.rs`。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{
    AppError, AppResult, BodyFormat, ContentCreatedSource, ContentStatus, ContentType,
    RelationType, double_option,
};

// ---------------------------------------------------------------------------
// 学习内容（LearningContent）
// ---------------------------------------------------------------------------

/// 列表行：**不选 `body`**（ADR-009 §2：100 条 × 100KB 的响应不可接受）
#[derive(Debug, sqlx::FromRow)]
pub struct ContentRow {
    pub id: Uuid,
    pub content_type: String,
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub body_format: String,
    pub source_url: Option<String>,
    pub created_source: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 详情行：多一列 `body`，唯一返回正文的查询
#[derive(Debug, sqlx::FromRow)]
pub struct ContentDetailRow {
    pub id: Uuid,
    pub content_type: String,
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub body: Option<String>,
    pub body_format: String,
    pub source_url: Option<String>,
    pub created_source: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 内容列表项（`ContentItem`）
#[derive(Debug, Serialize)]
pub struct ContentItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub content_type: ContentType,
    pub parent_id: Option<Uuid>,
    pub source_url: Option<String>,
    pub status: ContentStatus,
    pub body_format: BodyFormat,
    pub created_source: ContentCreatedSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ContentRow> for ContentItem {
    type Error = AppError;

    fn try_from(row: ContentRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            title: row.title,
            description: row.description,
            content_type: ContentType::parse(&row.content_type)?,
            parent_id: row.parent_id,
            source_url: row.source_url,
            status: ContentStatus::parse(&row.status)?,
            body_format: parse_body_format(&row.body_format),
            created_source: ContentCreatedSource::parse(&row.created_source)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 内容详情（`ContentDetailItem`）：列表视图 + `body` + `depth`
#[derive(Debug, Serialize)]
pub struct ContentDetailItem {
    #[serde(flatten)]
    pub item: ContentItem,
    pub body: Option<String>,
    /// 树深度，根节点为 0
    pub depth: i32,
}

impl ContentDetailItem {
    pub fn new(row: ContentDetailRow, depth: i32) -> AppResult<Self> {
        let body = row.body.clone();
        let item = ContentItem::try_from(ContentRow {
            id: row.id,
            content_type: row.content_type,
            title: row.title,
            description: row.description,
            parent_id: row.parent_id,
            body_format: row.body_format,
            source_url: row.source_url,
            created_source: row.created_source,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })?;
        Ok(Self { item, body, depth })
    }
}

/// 未知 `body_format` 按 `plain_text` 解释（ADR-009 §6：降级方向必须选更安全的一侧）
fn parse_body_format(value: &str) -> BodyFormat {
    BodyFormat::parse(value).unwrap_or(BodyFormat::PlainText)
}

/// 相似内容提示（`ContentHint`，D-24）
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ContentHint {
    pub id: Uuid,
    pub title: String,
    pub similarity: f32,
}

/// 内容列表数据（`ContentListData`）
#[derive(Debug, Serialize)]
pub struct ContentListData {
    pub items: Vec<ContentItem>,
    pub total: i64,
}

/// 创建结果（`ContentCreateData`）：`similar_hints` 只提示，不影响创建成功
#[derive(Debug, Serialize)]
pub struct ContentCreateData {
    pub item: ContentDetailItem,
    pub similar_hints: Vec<ContentHint>,
}

/// 创建内容请求
#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub title: String,
    pub content_type: ContentType,
    pub description: Option<String>,
    pub body: Option<String>,
    pub body_format: Option<BodyFormat>,
    pub source_url: Option<String>,
    pub parent_id: Option<Uuid>,
    /// ADR-009 §3：v1 不读写。用它接住「字段出现过」才能回 422，
    /// 而不是让 serde 静默丢弃（那会让违反契约的请求看起来成功）。
    #[serde(default, deserialize_with = "double_option")]
    pub metadata: Option<Option<serde_json::Value>>,
}

/// 更新内容请求（三态：缺失＝不变 / `null`＝置空 / 值＝更新）
///
/// `status` / `created_source` / `metadata` 不在可更新字段内；三者用存在性字段接住，
/// 出现即 422（ADR-009 §3、契约 §2.1），而不是被 serde 静默忽略。
#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub body: Option<Option<String>>,
    pub body_format: Option<BodyFormat>,
    #[serde(default, deserialize_with = "double_option")]
    pub source_url: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub parent_id: Option<Option<Uuid>>,
    pub content_type: Option<ContentType>,
    /// 以下三个字段 v1 不可写，仅用于「出现即 422」的存在性判断（契约 §2.1、ADR-009 §3）
    #[serde(default, deserialize_with = "double_option")]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(default, deserialize_with = "double_option")]
    pub status: Option<Option<serde_json::Value>>,
    #[serde(default, deserialize_with = "double_option")]
    pub created_source: Option<Option<serde_json::Value>>,
}

/// 内容列表的 `status` 过滤：比 DB 枚举多一个 `all`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatusFilter {
    Active,
    Archived,
    All,
}

/// 内容列表查询参数
#[derive(Debug, Deserialize)]
pub struct ContentQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub content_type: Option<ContentType>,
    pub status: Option<ContentStatusFilter>,
    pub parent_id: Option<Uuid>,
    pub root_only: Option<bool>,
    pub q: Option<String>,
}

impl ContentQuery {
    /// 返回 (limit, offset)，与全局分页约定一致（1 ≤ page_size ≤ 100）
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}

/// 删除内容查询参数
#[derive(Debug, Deserialize)]
pub struct PurgeQuery {
    #[serde(default)]
    pub purge: bool,
}

// ---------------------------------------------------------------------------
// 内容关系（ContentRelation）
// ---------------------------------------------------------------------------

/// 关系行
#[derive(Debug, sqlx::FromRow)]
pub struct RelationRow {
    pub id: Uuid,
    pub source_content_id: Uuid,
    pub target_content_id: Uuid,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
}

/// 关系方向：相对查询锚点而言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    Outbound,
    Inbound,
}

impl RelationDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Outbound => "outbound",
            Self::Inbound => "inbound",
        }
    }
}

/// 关系响应（`RelationItem`）
#[derive(Debug, Serialize)]
pub struct RelationItem {
    pub id: Uuid,
    pub source_content_id: Uuid,
    pub target_content_id: Uuid,
    pub relation_type: RelationType,
    pub direction: RelationDirection,
}

impl RelationItem {
    pub fn from_row(row: RelationRow, anchor: Uuid) -> AppResult<Self> {
        let direction = if row.source_content_id == anchor {
            RelationDirection::Outbound
        } else {
            RelationDirection::Inbound
        };
        Ok(Self {
            id: row.id,
            source_content_id: row.source_content_id,
            target_content_id: row.target_content_id,
            relation_type: RelationType::parse(&row.relation_type)?,
            direction,
        })
    }
}

/// 关系列表数据（`RelationListData`）
#[derive(Debug, Serialize)]
pub struct RelationListData {
    pub items: Vec<RelationItem>,
    pub total: i64,
}

/// 创建关系请求；源内容取自路径
#[derive(Debug, Deserialize)]
pub struct CreateRelationRequest {
    pub target_content_id: Uuid,
    pub relation_type: RelationType,
}

/// 关系列表查询参数
#[derive(Debug, Deserialize)]
pub struct RelationQuery {
    pub relation_type: Option<RelationType>,
    /// `outbound` / `inbound` / `all`（默认）
    pub direction: Option<RelationDirectionFilter>,
}

/// 关系查询的方向过滤：比 `RelationDirection` 多一个 `all`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirectionFilter {
    Outbound,
    Inbound,
    All,
}

// ---------------------------------------------------------------------------
// 标签（ContentTag）
// ---------------------------------------------------------------------------

/// 标签行
#[derive(Debug, sqlx::FromRow)]
pub struct ContentTagRow {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 标签响应（`ContentTagItem`）
#[derive(Debug, Serialize)]
pub struct ContentTagItem {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ContentTagRow> for ContentTagItem {
    fn from(row: ContentTagRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            color: row.color,
            created_at: row.created_at,
        }
    }
}

/// 标签列表数据（`ContentTagListData`）
#[derive(Debug, Serialize)]
pub struct ContentTagListData {
    pub items: Vec<ContentTagItem>,
    pub total: i64,
}

/// 创建标签请求
#[derive(Debug, Deserialize)]
pub struct CreateContentTagRequest {
    pub name: String,
    pub color: Option<String>,
}

/// 更新标签请求（三态；`name` 缺失即不改名）
#[derive(Debug, Deserialize)]
pub struct UpdateContentTagRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub color: Option<Option<String>>,
}

/// 整体替换内容标签（D-28：空数组表示清空）
#[derive(Debug, Deserialize)]
pub struct SetContentTagsRequest {
    pub tag_ids: Vec<Uuid>,
}

/// 标签列表查询参数
#[derive(Debug, Deserialize)]
pub struct ContentTagQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub q: Option<String>,
}

impl ContentTagQuery {
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}
