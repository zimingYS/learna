//! 里程碑（Milestone）的持久化与状态迁移。
//!
//! 里程碑走 `PATCH status`，迁移规则由 `rules::evaluate_milestone` 统一判定
//! （允许跳过中间态、允许回退、终态之间不直转、同态幂等 200）。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::roadmap::dto::{
    CreateMilestoneRequest, MilestoneItem, MilestoneListData, MilestoneQuery, MilestoneRow,
    UpdateMilestoneRequest,
};
use crate::modules::roadmap::rules::{self, TransitionOutcome};
use crate::shared::{AppError, AppResult, MilestoneStatus, RoadmapStatus};

const MILESTONE_COLUMNS: &str = r#"
    id, stage_id, title, description, status, completed_at, created_at, updated_at
"#;

/// 阶段归属校验：返回所属路线状态
async fn ensure_stage(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    stage_id: Uuid,
) -> AppResult<RoadmapStatus> {
    let status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT r.status
        FROM stages s
        JOIN roadmaps r ON r.id = s.roadmap_id
        JOIN learning_projects p ON p.id = r.project_id
        WHERE s.id = $1 AND r.project_id = $2 AND p.owner_id = $3
        "#,
    )
    .bind(stage_id)
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("阶段不存在".to_string()))?;

    RoadmapStatus::parse(&status)
}

async fn fetch_row(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    milestone_id: Uuid,
) -> AppResult<MilestoneRow> {
    let sql = format!(
        r#"
        SELECT {MILESTONE_COLUMNS}
        FROM milestones m
        WHERE m.id = $1
          AND EXISTS (
              SELECT 1 FROM stages s
              JOIN roadmaps r ON r.id = s.roadmap_id
              JOIN learning_projects p ON p.id = r.project_id
              WHERE s.id = m.stage_id AND r.project_id = $2 AND p.owner_id = $3
          )
        "#
    );

    sqlx::query_as::<_, MilestoneRow>(sqlx::AssertSqlSafe(sql))
        .bind(milestone_id)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("里程碑不存在".to_string()))
}

/// 里程碑列表（可按阶段过滤，按 created_at ASC 稳定排序）
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    query: &MilestoneQuery,
    limit: i64,
    offset: i64,
) -> AppResult<MilestoneListData> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_projects WHERE id = $1 AND owner_id = $2",
    )
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::NotFound("项目不存在".to_string()));
    }

    let sql = format!(
        r#"
        SELECT {MILESTONE_COLUMNS}
        FROM milestones m
        WHERE EXISTS (
            SELECT 1 FROM stages s
            JOIN roadmaps r ON r.id = s.roadmap_id
            WHERE s.id = m.stage_id AND r.project_id = $1
        )
          AND ($2::uuid IS NULL OR m.stage_id = $2)
        ORDER BY m.created_at ASC, m.id ASC
        LIMIT $3 OFFSET $4
        "#
    );

    let rows = sqlx::query_as::<_, MilestoneRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(query.stage_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM milestones m
        WHERE EXISTS (
            SELECT 1 FROM stages s
            JOIN roadmaps r ON r.id = s.roadmap_id
            WHERE s.id = m.stage_id AND r.project_id = $1
        )
          AND ($2::uuid IS NULL OR m.stage_id = $2)
        "#,
    )
    .bind(project_id)
    .bind(query.stage_id)
    .fetch_one(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(MilestoneItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(MilestoneListData { items, total })
}

/// 创建里程碑（`stage_id` 必填；仅 superseded 路线拒绝）
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: CreateMilestoneRequest,
) -> AppResult<MilestoneItem> {
    let roadmap_status = ensure_stage(pool, owner_id, project_id, payload.stage_id).await?;
    rules::ensure_roadmap_writable(roadmap_status)?;

    let title = rules::ensure_title("title", &payload.title, 200)?;

    let sql = format!(
        r#"
        INSERT INTO milestones (stage_id, title, description, status)
        VALUES ($1, $2, $3, 'pending')
        RETURNING {MILESTONE_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, MilestoneRow>(sqlx::AssertSqlSafe(sql))
        .bind(payload.stage_id)
        .bind(title)
        .bind(payload.description)
        .fetch_one(pool)
        .await?;

    MilestoneItem::try_from(row)
}

/// 里程碑详情
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    milestone_id: Uuid,
) -> AppResult<MilestoneItem> {
    let row = fetch_row(pool, owner_id, project_id, milestone_id).await?;
    MilestoneItem::try_from(row)
}

/// 更新里程碑（含状态迁移）
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    milestone_id: Uuid,
    payload: UpdateMilestoneRequest,
) -> AppResult<MilestoneItem> {
    let current = fetch_row(pool, owner_id, project_id, milestone_id).await?;
    let roadmap_status = ensure_stage(pool, owner_id, project_id, current.stage_id).await?;
    rules::ensure_roadmap_writable(roadmap_status)?;

    let current_status = MilestoneStatus::parse(&current.status)?;
    let target_status = payload.status.unwrap_or(current_status);
    let outcome = rules::evaluate_milestone(current_status, target_status)?;

    let title = match payload.title {
        Some(value) => Some(rules::ensure_title("title", &value, 200)?),
        None => None,
    };
    let description_set = payload.description.is_some();
    let description = payload.description.flatten();

    // 全部字段都是同态或未提供：幂等返回，不刷新 updated_at
    if outcome == TransitionOutcome::Noop && title.is_none() && !description_set {
        return MilestoneItem::try_from(current);
    }

    let sql = format!(
        r#"
        UPDATE milestones
        SET title = COALESCE($2::text, title),
            description = CASE WHEN $3::bool THEN $4::text ELSE description END,
            status = $5,
            completed_at = CASE
                WHEN $6 THEN now()
                WHEN $7 THEN NULL
                ELSE completed_at
            END,
            updated_at = now()
        WHERE id = $1
        RETURNING {MILESTONE_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, MilestoneRow>(sqlx::AssertSqlSafe(sql))
        .bind(milestone_id)
        .bind(title)
        .bind(description_set)
        .bind(description)
        .bind(target_status.as_str())
        .bind(outcome == TransitionOutcome::EnterCompleted)
        .bind(outcome == TransitionOutcome::LeaveCompleted)
        .fetch_one(pool)
        .await?;

    MilestoneItem::try_from(row)
}

/// 删除里程碑（仅草稿路线，物理删除；任务只解挂 `milestone_id`，保留 `stage_id`）
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    milestone_id: Uuid,
) -> AppResult<()> {
    let current = fetch_row(pool, owner_id, project_id, milestone_id).await?;
    let roadmap_status = ensure_stage(pool, owner_id, project_id, current.stage_id).await?;
    rules::ensure_roadmap_draft(roadmap_status)?;

    let mut tx = pool.begin().await?;

    // D-10：只清 milestone_id，保留 stage_id（任务不依赖里程碑存活）
    sqlx::query(
        "UPDATE learning_tasks SET milestone_id = NULL, updated_at = now() WHERE milestone_id = $1",
    )
    .bind(milestone_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM milestones WHERE id = $1")
        .bind(milestone_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
