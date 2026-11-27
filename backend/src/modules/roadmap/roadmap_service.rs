//! 路线（Roadmap）的版本化持久化与确认流程。

use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::roadmap::dto::{
    CreateRoadmapRequest, RoadmapItem, RoadmapListData, RoadmapRow, UpdateRoadmapRequest,
};
use crate::modules::roadmap::rules;
use crate::shared::{AppError, AppResult, RoadmapSource, RoadmapStatus};

const ROADMAP_COLUMNS: &str = r#"
    id, project_id, version, status, source, change_summary, confirmed_at,
    created_by, created_at, updated_at
"#;

async fn ensure_project(pool: &PgPool, owner_id: Uuid, project_id: Uuid) -> AppResult<()> {
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
    Ok(())
}

async fn fetch_row(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
) -> AppResult<RoadmapRow> {
    let sql = format!(
        r#"
        SELECT {ROADMAP_COLUMNS}
        FROM roadmaps r
        WHERE r.id = $1
          AND r.project_id = $2
          AND EXISTS (
              SELECT 1 FROM learning_projects p
              WHERE p.id = r.project_id AND p.owner_id = $3
          )
        "#
    );

    sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(sql))
        .bind(roadmap_id)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("路线不存在".to_string()))
}

/// 路线版本历史（含 superseded），按 version DESC
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<RoadmapListData> {
    ensure_project(pool, owner_id, project_id).await?;

    let sql = format!(
        r#"
        SELECT {ROADMAP_COLUMNS}
        FROM roadmaps
        WHERE project_id = $1
        ORDER BY version DESC, id DESC
        LIMIT $2 OFFSET $3
        "#
    );

    let rows = sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roadmaps WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(RoadmapItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(RoadmapListData { items, total })
}

/// 创建路线草稿：`version = MAX(version) + 1`，并发冲突时重试一次。
///
/// `MAX+1` 在同一语句内完成，避免「先查后插」的窗口；两个并发请求仍可能算出同一个
/// `version`，靠 `UNIQUE(project_id, version)` 兜住，捕获 23505 后重试。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: CreateRoadmapRequest,
) -> AppResult<RoadmapItem> {
    ensure_project(pool, owner_id, project_id).await?;

    let source = payload.source.unwrap_or(RoadmapSource::User);
    let sql = format!(
        r#"
        INSERT INTO roadmaps (project_id, version, status, source, change_summary, created_by)
        SELECT $1, COALESCE(MAX(version), 0) + 1, 'draft', $2, $3, $4
        FROM roadmaps
        WHERE project_id = $1
        RETURNING {ROADMAP_COLUMNS}
        "#
    );

    let mut last_error = None;
    for _ in 0..2 {
        match sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(sql.clone()))
            .bind(project_id)
            .bind(source.as_str())
            .bind(&payload.change_summary)
            .bind(owner_id)
            .fetch_one(pool)
            .await
        {
            Ok(row) => return RoadmapItem::try_from(row),
            Err(err) => {
                if is_unique_violation(&err) {
                    last_error = Some(err);
                    continue;
                }
                return Err(AppError::Internal(format!("insert roadmap: {err}")));
            }
        }
    }

    Err(AppError::Conflict(format!(
        "路线版本号分配冲突，请重试：{}",
        last_error.map(|e| e.to_string()).unwrap_or_default()
    )))
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

/// 路线详情
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
) -> AppResult<RoadmapItem> {
    let row = fetch_row(pool, owner_id, project_id, roadmap_id).await?;
    RoadmapItem::try_from(row)
}

/// 更新路线（仅 draft）
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    payload: UpdateRoadmapRequest,
) -> AppResult<RoadmapItem> {
    let current = fetch_row(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_draft_editable(RoadmapStatus::parse(&current.status)?)?;

    let Some(summary) = payload.change_summary else {
        // 无字段可改：按幂等返回当前行
        return RoadmapItem::try_from(current);
    };

    let sql = format!(
        r#"
        UPDATE roadmaps
        SET change_summary = $1, updated_at = now()
        WHERE id = $2
        RETURNING {ROADMAP_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(sql))
        .bind(summary)
        .bind(roadmap_id)
        .fetch_one(pool)
        .await?;

    RoadmapItem::try_from(row)
}

/// 确认路线：`draft → active`，同项目旧 `active` 降级为 `superseded`（单事务）。
///
/// 加锁顺序固定为「同项目全部路线行按 id 排序」——若改为锁「当前 active 行」，
/// 两个并发 confirm 可能各锁不同版本而互相等待。
pub async fn confirm(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
) -> AppResult<RoadmapItem> {
    ensure_project(pool, owner_id, project_id).await?;

    let mut tx = pool.begin().await?;

    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, status FROM roadmaps WHERE project_id = $1 ORDER BY id FOR UPDATE",
    )
    .bind(project_id)
    .fetch_all(&mut *tx)
    .await?;

    let target = rows
        .iter()
        .find(|(id, _)| *id == roadmap_id)
        .ok_or_else(|| AppError::NotFound("路线不存在".to_string()))?;

    rules::ensure_confirmable(RoadmapStatus::parse(&target.1)?)?;

    sqlx::query(
        r#"
        UPDATE roadmaps
        SET status = 'superseded', updated_at = now()
        WHERE project_id = $1 AND status = 'active' AND id <> $2
        "#,
    )
    .bind(project_id)
    .bind(roadmap_id)
    .execute(&mut *tx)
    .await?;

    let sql = format!(
        r#"
        UPDATE roadmaps
        SET status = 'active', confirmed_at = now(), updated_at = now()
        WHERE id = $1
        RETURNING {ROADMAP_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(sql))
        .bind(roadmap_id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    RoadmapItem::try_from(row)
}

/// 删除草稿路线（D-12）：物理删除 stage / milestone、清 goal_stage、任务解挂保留。
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    roadmap_id: Uuid,
    cascade: bool,
) -> AppResult<()> {
    let current = fetch_row(pool, owner_id, project_id, roadmap_id).await?;
    rules::ensure_roadmap_draft(RoadmapStatus::parse(&current.status)?)?;

    let stage_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM stages WHERE roadmap_id = $1")
            .bind(roadmap_id)
            .fetch_one(pool)
            .await?;

    let milestone_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM milestones m
        JOIN stages s ON s.id = m.stage_id
        WHERE s.roadmap_id = $1
        "#,
    )
    .bind(roadmap_id)
    .fetch_one(pool)
    .await?;

    let task_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM learning_tasks t
        WHERE t.deleted_at IS NULL
          AND (t.stage_id IN (SELECT id FROM stages WHERE roadmap_id = $1)
            OR t.milestone_id IN (
                SELECT m.id FROM milestones m
                JOIN stages s ON s.id = m.stage_id
                WHERE s.roadmap_id = $1
            ))
        "#,
    )
    .bind(roadmap_id)
    .fetch_one(pool)
    .await?;

    if !cascade && (stage_count > 0 || milestone_count > 0) {
        return Err(AppError::DeletionConflict {
            impact: serde_json::json!({
                "stages": stage_count,
                "milestones": milestone_count,
                "tasks": task_count,
            }),
            hint: "路线下存在阶段或里程碑，确认一并删除请追加 ?cascade=true".to_string(),
        });
    }

    let mut tx = pool.begin().await?;

    // 任务解挂保留（L3）：显式清空两列，DB 的 ON DELETE SET NULL 作为兜底
    // 含已软删行——外键不看 deleted_at，留着引用只会在删阶段时违约
    sqlx::query(
        r#"
        UPDATE learning_tasks
        SET stage_id = NULL, milestone_id = NULL, updated_at = now()
        WHERE stage_id IN (SELECT id FROM stages WHERE roadmap_id = $1)
           OR milestone_id IN (
                SELECT m.id FROM milestones m
                JOIN stages s ON s.id = m.stage_id
                WHERE s.roadmap_id = $1
           )
        "#,
    )
    .bind(roadmap_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM milestones m
        USING stages s
        WHERE m.stage_id = s.id AND s.roadmap_id = $1
        "#,
    )
    .bind(roadmap_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM goal_stages gs
        USING stages s
        WHERE gs.stage_id = s.id AND s.roadmap_id = $1
        "#,
    )
    .bind(roadmap_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM stages WHERE roadmap_id = $1")
        .bind(roadmap_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM roadmaps WHERE id = $1")
        .bind(roadmap_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
