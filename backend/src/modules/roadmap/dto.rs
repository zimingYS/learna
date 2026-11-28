//! M04 规划域 DTO：目标 / 路线 / 阶段 / 里程碑。
//!
//! 契约来源：`openapi.yaml` 1.4.1 的 `GoalItem` / `RoadmapItem` / `StageItem` /
//! `MilestoneItem` 及其请求体。任务相关 DTO 在 `modules/task/dto.rs`。

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{
    AppError, AppResult, CompletionMethod, Contribution, GoalStatus, GoalType, MilestoneStatus,
    RoadmapSource, RoadmapStatus, StageStatus, double_option,
};

// ---------------------------------------------------------------------------
// 目标（ProjectGoal）
// ---------------------------------------------------------------------------

/// 目标行
#[derive(Debug, sqlx::FromRow)]
pub struct GoalRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_goal_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub status: String,
    pub target_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 目标响应（`GoalItem`）
#[derive(Debug, Serialize)]
pub struct GoalItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_goal_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub goal_type: GoalType,
    pub status: GoalStatus,
    pub target_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<GoalRow> for GoalItem {
    type Error = AppError;

    fn try_from(row: GoalRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            project_id: row.project_id,
            parent_goal_id: row.parent_goal_id,
            title: row.title,
            description: row.description,
            goal_type: GoalType::parse(&row.goal_type)?,
            status: GoalStatus::parse(&row.status)?,
            target_at: row.target_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 目标列表数据（`GoalListData`）
#[derive(Debug, Serialize)]
pub struct GoalListData {
    pub items: Vec<GoalItem>,
    pub total: i64,
}

/// 创建目标请求
#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub title: String,
    pub description: Option<String>,
    pub parent_goal_id: Option<Uuid>,
    pub goal_type: Option<GoalType>,
    pub target_at: Option<DateTime<Utc>>,
}

/// 更新目标请求（三态语义）
#[derive(Debug, Deserialize)]
pub struct UpdateGoalRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub parent_goal_id: Option<Option<Uuid>>,
    pub goal_type: Option<GoalType>,
    #[serde(default, deserialize_with = "double_option")]
    pub target_at: Option<Option<DateTime<Utc>>>,
    pub status: Option<GoalStatus>,
}

/// 目标-阶段关联行
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GoalStageItem {
    pub goal_id: Uuid,
    pub stage_id: Uuid,
    pub contribution: String,
}

/// 目标-阶段关联列表（`GoalStageListEnvelope` 的 data）
#[derive(Debug, Serialize)]
pub struct GoalStageListData {
    pub items: Vec<GoalStageLinkOut>,
}

/// 对外关联行；`contribution` 已解析为枚举
#[derive(Debug, Serialize)]
pub struct GoalStageLinkOut {
    pub goal_id: Uuid,
    pub stage_id: Uuid,
    pub contribution: Contribution,
}

impl TryFrom<GoalStageItem> for GoalStageLinkOut {
    type Error = AppError;

    fn try_from(row: GoalStageItem) -> AppResult<Self> {
        Ok(Self {
            goal_id: row.goal_id,
            stage_id: row.stage_id,
            contribution: parse_contribution(&row.contribution)?,
        })
    }
}

fn parse_contribution(value: &str) -> AppResult<Contribution> {
    match value {
        "primary" => Ok(Contribution::Primary),
        "supporting" => Ok(Contribution::Supporting),
        other => Err(AppError::Internal(format!(
            "unknown contribution value in database: {other}"
        ))),
    }
}

/// 单条关联（请求体）
#[derive(Debug, Deserialize)]
pub struct GoalStageLink {
    pub stage_id: Uuid,
    pub contribution: Option<Contribution>,
}

/// 整体替换请求（D-13）
#[derive(Debug, Deserialize)]
pub struct ReplaceGoalStageLinksRequest {
    pub links: Vec<GoalStageLink>,
}

// ---------------------------------------------------------------------------
// 路线（Roadmap）
// ---------------------------------------------------------------------------

/// 路线行
#[derive(Debug, sqlx::FromRow)]
pub struct RoadmapRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub version: i32,
    pub status: String,
    pub source: String,
    pub change_summary: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 路线响应（`RoadmapItem`）
#[derive(Debug, Serialize)]
pub struct RoadmapItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub version: i32,
    pub status: RoadmapStatus,
    pub source: RoadmapSource,
    pub change_summary: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<RoadmapRow> for RoadmapItem {
    type Error = AppError;

    fn try_from(row: RoadmapRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            project_id: row.project_id,
            version: row.version,
            status: RoadmapStatus::parse(&row.status)?,
            source: RoadmapSource::parse(&row.source)?,
            change_summary: row.change_summary,
            confirmed_at: row.confirmed_at,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 路线列表数据
#[derive(Debug, Serialize)]
pub struct RoadmapListData {
    pub items: Vec<RoadmapItem>,
    pub total: i64,
}

/// 创建路线请求；`version` 与 `status` 由服务端决定
#[derive(Debug, Deserialize)]
pub struct CreateRoadmapRequest {
    pub source: Option<RoadmapSource>,
    pub change_summary: Option<String>,
}

/// 更新路线请求（仅 draft）
#[derive(Debug, Deserialize)]
pub struct UpdateRoadmapRequest {
    #[serde(default, deserialize_with = "double_option")]
    pub change_summary: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// 阶段（Stage）
// ---------------------------------------------------------------------------

/// 阶段行
#[derive(Debug, sqlx::FromRow)]
pub struct StageRow {
    pub id: Uuid,
    pub roadmap_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sequence_no: i32,
    pub status: String,
    pub completion_method: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 阶段响应（`StageItem`）
#[derive(Debug, Serialize)]
pub struct StageItem {
    pub id: Uuid,
    pub roadmap_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sequence_no: i32,
    pub status: StageStatus,
    pub completion_method: CompletionMethod,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<StageRow> for StageItem {
    type Error = AppError;

    fn try_from(row: StageRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            roadmap_id: row.roadmap_id,
            name: row.name,
            description: row.description,
            sequence_no: row.sequence_no,
            status: StageStatus::parse(&row.status)?,
            completion_method: CompletionMethod::parse(&row.completion_method)?,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 阶段列表数据
#[derive(Debug, Serialize)]
pub struct StageListData {
    pub items: Vec<StageItem>,
    pub total: i64,
}

/// 创建阶段请求
#[derive(Debug, Deserialize)]
pub struct CreateStageRequest {
    pub name: String,
    pub description: Option<String>,
    pub sequence_no: Option<i32>,
}

/// 更新阶段请求（状态必须走动作端点）
#[derive(Debug, Deserialize)]
pub struct UpdateStageRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    pub sequence_no: Option<i32>,
}

// ---------------------------------------------------------------------------
// 里程碑（Milestone）
// ---------------------------------------------------------------------------

/// 里程碑行
#[derive(Debug, sqlx::FromRow)]
pub struct MilestoneRow {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 里程碑响应（`MilestoneItem`）
#[derive(Debug, Serialize)]
pub struct MilestoneItem {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: MilestoneStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<MilestoneRow> for MilestoneItem {
    type Error = AppError;

    fn try_from(row: MilestoneRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            stage_id: row.stage_id,
            title: row.title,
            description: row.description,
            status: MilestoneStatus::parse(&row.status)?,
            completed_at: row.completed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// 里程碑列表数据
#[derive(Debug, Serialize)]
pub struct MilestoneListData {
    pub items: Vec<MilestoneItem>,
    pub total: i64,
}

/// 创建里程碑请求
#[derive(Debug, Deserialize)]
pub struct CreateMilestoneRequest {
    pub stage_id: Uuid,
    pub title: String,
    pub description: Option<String>,
}

/// 更新里程碑请求（含状态，按统一规则校验）
#[derive(Debug, Deserialize)]
pub struct UpdateMilestoneRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    pub status: Option<MilestoneStatus>,
}

// ---------------------------------------------------------------------------
// 通用查询
// ---------------------------------------------------------------------------

/// 纯分页查询
#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl PageQuery {
    /// 返回 (limit, offset)，与全局分页约定一致（1 ≤ page_size ≤ 100）
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}

/// 里程碑列表查询：额外支持按阶段过滤
#[derive(Debug, Deserialize)]
pub struct MilestoneQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub stage_id: Option<Uuid>,
}

impl MilestoneQuery {
    /// 返回 (limit, offset)
    pub fn window(&self) -> (i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        (page_size, (page - 1) * page_size)
    }
}
