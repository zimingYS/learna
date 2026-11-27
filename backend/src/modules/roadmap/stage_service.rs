//! 阶段（Stage）的持久化与四动作执行。
//!
//! 阶段的状态迁移**不走 PATCH**：`pending` 必须先 `start` 才能 `complete`，
//! 回退目标固定为 `active`（ADR-007 §1 的两条例外）。判定全部在 `rules::plan_stage_action`。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::roadmap::dto::{
    CreateStageRequest, StageItem, StageListData, StageRow, UpdateStageRequest,
};
use crate::modules::roadmap::rules::{self, StageAction};
use crate::shared::{AppError, AppResult, RoadmapStatus, StageStatus};

const STAGE_COLUMNS: &str = r#"
    id, roadmap_id, name, description, sequence_no, status, completion_method,
    started_at, completed_at, created_at, updated_at
"#;

/// 路线归属校验：返回路线状态（一次查询同时完成存在性、归属、父链一致性）
async fn ensure_roadmap(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
) -> AppResult<RoadmapStatus> {
    let status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT r.status
        FROM roadmaps r
        JOIN learning_projects p ON p.id = r.project_id
        WHERE r.id = $1 AND r.project_id = $2 AND p.owner_id = $3
        "#,
    )
    .bind(roadmap_id)
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("路线不存在".to_string()))?;

    RoadmapStatus::parse(&status)
}

async fn fetch_row(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    stage_id: Uuid,
) -> AppResult<StageRow> {
    let sql = format!(
        r#"
        SELECT {STAGE_COLUMNS}
        FROM stages s
        WHERE s.id = $1
          AND s.roadmap_id = $2
          AND EXISTS (
              SELECT 1 FROM roadmaps r
              JOIN learning_projects p ON p.id = r.project_id
              WHERE r.id = s.roadmap_id AND r.project_id = $3 AND p.owner_id = $4
          )
        "#
    );

    sqlx::query_as::<_, StageRow>(sqlx::AssertSqlSafe(sql))
        .bind(stage_id)
        .bind(roadmap_id)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("阶段不存在".to_string()))
}

/// 阶段列表（按 sequence_no ASC）
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<StageListData> {
    ensure_roadmap(pool, owner_id, project_id, roadmap_id).await?;

    let sql = format!(
        r#"
        SELECT {STAGE_COLUMNS}
        FROM stages
        WHERE roadmap_id = $1
        ORDER BY sequence_no ASC, id ASC
        LIMIT $2 OFFSET $3
        "#
    );

    let rows = sqlx::query_as::<_, StageRow>(sqlx::AssertSqlSafe(sql))
        .bind(roadmap_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM stages WHERE roadmap_id = $1")
        .bind(roadmap_id)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(StageItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(StageListData { items, total })
}

/// 创建阶段：`sequence_no` 省略时追加到末尾。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    payload: CreateStageRequest,
) -> AppResult<StageItem> {
    let status = ensure_roadmap(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_draft_editable(status)?;

    let name = rules::ensure_title("name", &payload.name, 160)?;

    if let Some(seq) = payload.sequence_no {
        rules::ensure_sequence_no(seq)?;
    }

    let sql = format!(
        r#"
        INSERT INTO stages (roadmap_id, name, description, sequence_no, status, completion_method)
        SELECT $1, $2, $3,
               COALESCE($4::int, (SELECT COALESCE(MAX(sequence_no), 0) + 1 FROM stages WHERE roadmap_id = $1)),
               'pending', 'user'
        RETURNING {STAGE_COLUMNS}
        "#
    );

    let mut last_error = None;
    for _ in 0..2 {
        match sqlx::query_as::<_, StageRow>(sqlx::AssertSqlSafe(sql.clone()))
            .bind(roadmap_id)
            .bind(&name)
            .bind(&payload.description)
            .bind(payload.sequence_no)
            .fetch_one(pool)
            .await
        {
            Ok(row) => return StageItem::try_from(row),
            Err(err) => {
                if is_unique_violation(&err) {
                    last_error = Some(err);
                    continue;
                }
                return Err(AppError::Internal(format!("insert stage: {err}")));
            }
        }
    }

    Err(AppError::Conflict(format!(
        "sequence_no 分配冲突，请重试：{}",
        last_error.map(|e| e.to_string()).unwrap_or_default()
    )))
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

/// 阶段详情
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    stage_id: Uuid,
) -> AppResult<StageItem> {
    let row = fetch_row(pool, owner_id, project_id, roadmap_id, stage_id).await?;
    StageItem::try_from(row)
}

/// 更新阶段（状态不可经此修改）。
///
/// 三态语义用 `COALESCE` + 显式 `*_set` 标志表达：缺失＝不变、null＝置空、值＝更新。
/// 不拼动态 SQL——参数编号固定，PG 才能从 `::text` 等标注推断每个占位符的类型。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    stage_id: Uuid,
    payload: UpdateStageRequest,
) -> AppResult<StageItem> {
    let current = fetch_row(pool, owner_id, project_id, roadmap_id, stage_id).await?;
    let roadmap_status = ensure_roadmap(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_draft_editable(roadmap_status)?;

    let name = match payload.name {
        Some(value) => Some(rules::ensure_title("name", &value, 160)?),
        None => None,
    };

    if let Some(sequence_no) = payload.sequence_no {
        rules::ensure_sequence_no(sequence_no)?;
    }

    let description_set = payload.description.is_some();
    let description = payload.description.flatten();

    if name.is_none() && !description_set && payload.sequence_no.is_none() {
        return StageItem::try_from(current);
    }

    let sql = format!(
        r#"
        UPDATE stages
        SET name = COALESCE($3::text, name),
            description = CASE WHEN $4::bool THEN $5::text ELSE description END,
            sequence_no = COALESCE($6::int, sequence_no),
            updated_at = now()
        WHERE id = $1 AND roadmap_id = $2
        RETURNING {STAGE_COLUMNS}
        "#
    );

    let result = sqlx::query_as::<_, StageRow>(sqlx::AssertSqlSafe(sql))
        .bind(stage_id)
        .bind(roadmap_id)
        .bind(name)
        .bind(description_set)
        .bind(description)
        .bind(payload.sequence_no)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => StageItem::try_from(row),
        Err(err) if is_unique_violation(&err) => Err(AppError::Validation(
            "sequence_no 在同一路线内必须唯一".to_string(),
        )),
        Err(err) => Err(AppError::Internal(format!("update stage: {err}"))),
    }
}

/// 删除阶段（仅草稿路线，物理删除）。
///
/// 其下里程碑一并删除；任务只解挂（`stage_id` 与 `milestone_id` 都清）。
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    stage_id: Uuid,
    cascade: bool,
) -> AppResult<()> {
    fetch_row(pool, owner_id, project_id, roadmap_id, stage_id).await?;
    let roadmap_status = ensure_roadmap(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_roadmap_draft(roadmap_status)?;

    let milestone_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM milestones WHERE stage_id = $1")
            .bind(stage_id)
            .fetch_one(pool)
            .await?;

    let task_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM learning_tasks
        WHERE deleted_at IS NULL
          AND (stage_id = $1 OR milestone_id IN (SELECT id FROM milestones WHERE stage_id = $1))
        "#,
    )
    .bind(stage_id)
    .fetch_one(pool)
    .await?;

    if !cascade && milestone_count > 0 {
        return Err(AppError::DeletionConflict {
            impact: serde_json::json!({
                "milestones": milestone_count,
                "tasks": task_count,
            }),
            hint: "阶段下存在里程碑，确认一并删除请追加 ?cascade=true".to_string(),
        });
    }

    let mut tx = pool.begin().await?;

    // 任务解挂保留（含已软删行：外键不看 deleted_at）
    sqlx::query(
        r#"
        UPDATE learning_tasks
        SET stage_id = NULL, milestone_id = NULL, updated_at = now()
        WHERE stage_id = $1
           OR milestone_id IN (SELECT id FROM milestones WHERE stage_id = $1)
        "#,
    )
    .bind(stage_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM milestones WHERE stage_id = $1")
        .bind(stage_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM goal_stages WHERE stage_id = $1")
        .bind(stage_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM stages WHERE id = $1")
        .bind(stage_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 执行阶段动作（start / complete / skip / reactivate）。
///
/// 准入门槛：所属路线必须是 `active`（draft 是编辑区、superseded 只读）。
pub async fn apply_action(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    stage_id: Uuid,
    action: StageAction,
) -> AppResult<StageItem> {
    let current = fetch_row(pool, owner_id, project_id, roadmap_id, stage_id).await?;
    let roadmap_status = ensure_roadmap(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_roadmap_active(roadmap_status)?;

    let current_status = StageStatus::parse(&current.status)?;
    let plan = rules::plan_stage_action(current_status, action)?;

    if plan.noop {
        return StageItem::try_from(current);
    }

    let sql = format!(
        r#"
        UPDATE stages
        SET status = $1,
            started_at = CASE WHEN $2 THEN now() ELSE started_at END,
            completed_at = CASE
                WHEN $3 THEN now()
                WHEN $4 THEN NULL
                ELSE completed_at
            END,
            completion_method = CASE WHEN $3 THEN 'user' ELSE completion_method END,
            updated_at = now()
        WHERE id = $5 AND roadmap_id = $6 AND status = $7
        RETURNING {STAGE_COLUMNS}
        "#
    );

    sqlx::query_as::<_, StageRow>(sqlx::AssertSqlSafe(sql))
        .bind(plan.target.as_str())
        .bind(plan.write_started_at)
        .bind(plan.write_completed_at)
        .bind(plan.clear_completed_at)
        .bind(stage_id)
        .bind(roadmap_id)
        .bind(current_status.as_str())
        .fetch_optional(pool)
        .await?
        .map(StageItem::try_from)
        .ok_or_else(|| AppError::Conflict("阶段状态已被并发修改，请重试".to_string()))?
}
