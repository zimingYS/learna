//! M02 学习项目的持久化与生命周期编排。
//!
//! 两条贯穿全文件的约定：
//! 1. **归属校验写进 SQL 的 `WHERE`**，不做事先 SELECT 再判断——越权与不存在都得到 404，
//!    调用方无法通过状态码差异探测他人项目是否存在。
//! 2. **状态迁移先判后写**：先用纯函数 `rules::transition` 判定，再带 `status = 当前值`
//!    做条件更新，天然获得乐观并发（并发下第二个请求 `rows_affected = 0` → 409）。

use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use crate::modules::project::dto::{
    CreateProjectRequest, ExtendDeadlineRequest, ProjectItem, ProjectListData, ProjectQuery,
    ProjectRow, PurgeProjectRequest, UpdateProjectRequest,
};
use crate::modules::project::rules::{self, ProjectAction};
use crate::shared::{AppError, AppResult, DeadlineType, ProjectStatus, ProjectType};

/// 项目表可返回的全部列（`RETURNING` / `SELECT` 共用，避免字段漂移）
///
/// 以 `format!` 拼进语句后由 `AssertSqlSafe` 放行：拼接内容只有这个常量，
/// 不含任何用户输入，绑定参数仍走 `$n` 占位。
const PROJECT_COLUMNS: &str = r#"
    id, name, description, project_type, status, importance_level,
    importance_override_until, deadline_type, deadline_at,
    archived_at, reopened_at, created_at, updated_at
"#;

/// 创建项目。初始状态 `draft`，省略字段取 D-19 默认值。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateProjectRequest,
) -> AppResult<ProjectItem> {
    let name = rules::ensure_name(&payload.name)?;
    let importance = payload.importance_level.unwrap_or(3);
    rules::ensure_importance(importance)?;

    let project_type = payload.project_type.unwrap_or(ProjectType::Study);
    let deadline_type = payload.deadline_type.unwrap_or(DeadlineType::None);
    rules::ensure_deadline_pair(deadline_type, payload.deadline_at)?;

    let sql = format!(
        r#"
        INSERT INTO learning_projects (
            owner_id, name, description, project_type, status,
            importance_level, importance_override_until, deadline_type, deadline_at
        ) VALUES ($1, $2, $3, $4, 'draft', $5, $6, $7, $8)
        RETURNING {PROJECT_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, ProjectRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(&name)
        .bind(&payload.description)
        .bind(project_type.as_str())
        .bind(importance)
        .bind(payload.importance_override_until)
        .bind(deadline_type.as_str())
        .bind(payload.deadline_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("insert project: {e}")))?;

    ProjectItem::try_from(row)
}

/// 项目列表。默认排除 `archived`（D-15），排序固定 `updated_at DESC, id DESC`。
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    query: &ProjectQuery,
    limit: i64,
    offset: i64,
) -> AppResult<ProjectListData> {
    let status_filter = query.status.map(|s| s.as_str().to_string());
    let include_archived = query.include_archived.unwrap_or(false);

    let sql = format!(
        r#"
        SELECT {PROJECT_COLUMNS}
        FROM learning_projects
        WHERE owner_id = $1
          AND ($2::text IS NULL OR status = $2)
          -- 「默认排除归档」只在未指定 status 时生效：显式 status=archived 必须查得到
          AND ($2::text IS NOT NULL OR $3::bool OR status <> 'archived')
        ORDER BY updated_at DESC, id DESC
        LIMIT $4 OFFSET $5
        "#
    );

    let rows = sqlx::query_as::<_, ProjectRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(&status_filter)
        .bind(include_archived)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM learning_projects
        WHERE owner_id = $1
          AND ($2::text IS NULL OR status = $2)
          AND ($2::text IS NOT NULL OR $3::bool OR status <> 'archived')
        "#,
    )
    .bind(owner_id)
    .bind(&status_filter)
    .bind(include_archived)
    .fetch_one(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(ProjectItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(ProjectListData { items, total })
}

/// 项目详情
pub async fn detail(pool: &PgPool, owner_id: Uuid, project_id: Uuid) -> AppResult<ProjectItem> {
    let row = fetch_row(pool, owner_id, project_id).await?;
    ProjectItem::try_from(row)
}

async fn fetch_row(pool: &PgPool, owner_id: Uuid, project_id: Uuid) -> AppResult<ProjectRow> {
    let sql = format!(
        r#"
        SELECT {PROJECT_COLUMNS}
        FROM learning_projects
        WHERE id = $1 AND owner_id = $2
        "#
    );

    sqlx::query_as::<_, ProjectRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))
}

/// 当前状态（用于迁移前判定）
async fn fetch_status(pool: &PgPool, owner_id: Uuid, project_id: Uuid) -> AppResult<ProjectStatus> {
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM learning_projects WHERE id = $1 AND owner_id = $2",
    )
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))?;

    ProjectStatus::parse(&status)
}

/// 部分更新（ADR-002 三态）。状态与 `owner_id` 不在可更新字段内。
///
/// `deadline_type = fixed` 的项目不允许经此端点改期，须走 `extend_deadline`（D-04）。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: UpdateProjectRequest,
) -> AppResult<ProjectItem> {
    let current = fetch_row(pool, owner_id, project_id).await?;
    let current_deadline_type = DeadlineType::parse(&current.deadline_type)?;

    // 合并后的最终值，用于校验 deadline 组合而不只看单个字段
    let next_deadline_type = payload.deadline_type.unwrap_or(current_deadline_type);
    let next_deadline_at = match payload.deadline_at {
        Some(value) => value,
        None => current.deadline_at,
    };

    if current_deadline_type == DeadlineType::Fixed
        && (payload.deadline_at.is_some() || payload.deadline_type.is_some())
    {
        return Err(AppError::Validation(
            "fixed 项目的截止日期只能通过 POST /projects/{id}/extend-deadline 修改".to_string(),
        ));
    }

    // 只在两字段之一被触及，或类型变化时校验组合
    if payload.deadline_type.is_some() || payload.deadline_at.is_some() {
        rules::ensure_deadline_pair(next_deadline_type, next_deadline_at)?;
    }

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE learning_projects SET ");
    let mut separated = builder.separated(", ");
    let mut touched = false;

    if let Some(name) = payload.name {
        let name = rules::ensure_name(&name)?;
        separated.push("name = ").push_bind_unseparated(name);
        touched = true;
    }

    if let Some(description) = payload.description {
        separated
            .push("description = ")
            .push_bind_unseparated(description);
        touched = true;
    }

    if let Some(project_type) = payload.project_type {
        separated
            .push("project_type = ")
            .push_bind_unseparated(project_type.as_str().to_string());
        touched = true;
    }

    if let Some(importance) = payload.importance_level {
        rules::ensure_importance(importance)?;
        separated
            .push("importance_level = ")
            .push_bind_unseparated(importance);
        touched = true;
    }

    if let Some(override_until) = payload.importance_override_until {
        separated
            .push("importance_override_until = ")
            .push_bind_unseparated(override_until);
        touched = true;
    }

    if let Some(deadline_type) = payload.deadline_type {
        separated
            .push("deadline_type = ")
            .push_bind_unseparated(deadline_type.as_str().to_string());
        touched = true;
    }

    if let Some(deadline_at) = payload.deadline_at {
        separated
            .push("deadline_at = ")
            .push_bind_unseparated(deadline_at);
        touched = true;
    }

    if !touched {
        return Err(AppError::Validation("没有可更新的字段".to_string()));
    }

    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(project_id)
        .push(" AND owner_id = ")
        .push_bind(owner_id)
        .push(format!(" RETURNING {PROJECT_COLUMNS}"));

    let row = builder
        .build_query_as::<ProjectRow>()
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))?;

    ProjectItem::try_from(row)
}

/// 生命周期动作的统一入口。
pub async fn apply_action(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    action: ProjectAction,
) -> AppResult<ProjectItem> {
    let current = fetch_status(pool, owner_id, project_id).await?;

    let target = rules::transition(current, action).ok_or_else(|| {
        AppError::Conflict(format!(
            "项目当前状态 {} 不支持 {} 操作",
            current.as_str(),
            action.as_str()
        ))
    })?;

    let write_archived_at = rules::writes_archived_at(action);
    let reopen = rules::writes_reopened_at(action);

    let sql = format!(
        r#"
        UPDATE learning_projects
        SET status = $1,
            archived_at = CASE WHEN $2 THEN now() WHEN $3 THEN NULL ELSE archived_at END,
            reopened_at = CASE WHEN $3 THEN now() ELSE reopened_at END,
            updated_at = now()
        WHERE id = $4 AND owner_id = $5 AND status = $6
        RETURNING {PROJECT_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, ProjectRow>(sqlx::AssertSqlSafe(sql))
        .bind(target.as_str())
        .bind(write_archived_at)
        .bind(reopen)
        .bind(project_id)
        .bind(owner_id)
        .bind(current.as_str())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Conflict("项目状态已被并发修改，请重试".to_string()))?;

    ProjectItem::try_from(row)
}

/// 修改 `fixed` 截止日期（D-04）。
///
/// v1 无审计表，`reason` 以结构化 WARN 日志落盘（ERD §8.1 step 9 的降级形态）。
pub async fn extend_deadline(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: ExtendDeadlineRequest,
) -> AppResult<ProjectItem> {
    let current = fetch_row(pool, owner_id, project_id).await?;

    if DeadlineType::parse(&current.deadline_type)? != DeadlineType::Fixed {
        return Err(AppError::Validation(
            "仅 deadline_type=fixed 的项目可走改期流程；其余类型请用 PATCH".to_string(),
        ));
    }

    if payload.reason.trim().is_empty() {
        return Err(AppError::Validation("reason 不能为空".to_string()));
    }

    if let Some(old) = current.deadline_at
        && payload.new_deadline_at <= old
    {
        return Err(AppError::Validation(
            "new_deadline_at 必须晚于当前截止日期".to_string(),
        ));
    }

    let sql = format!(
        r#"
        UPDATE learning_projects
        SET deadline_at = $1, updated_at = now()
        WHERE id = $2 AND owner_id = $3
        RETURNING {PROJECT_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, ProjectRow>(sqlx::AssertSqlSafe(sql))
        .bind(payload.new_deadline_at)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))?;

    tracing::warn!(
        project_id = %project_id,
        owner_id = %owner_id,
        old_deadline_at = ?current.deadline_at,
        new_deadline_at = ?payload.new_deadline_at,
        reason = payload.reason.as_str(),
        "fixed 截止日期改期"
    );

    ProjectItem::try_from(row)
}

/// 规划域各子对象计数（删除前统计，用于日志与影响面）
#[derive(Debug, sqlx::FromRow)]
struct PlanningCounts {
    goals: i64,
    roadmaps: i64,
    stages: i64,
    milestones: i64,
    tasks: i64,
}

async fn count_planning(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> AppResult<PlanningCounts> {
    let counts = sqlx::query_as::<_, PlanningCounts>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM project_goals WHERE project_id = $1) AS goals,
            (SELECT COUNT(*) FROM roadmaps WHERE project_id = $1) AS roadmaps,
            (SELECT COUNT(*) FROM stages s
                JOIN roadmaps r ON r.id = s.roadmap_id
                WHERE r.project_id = $1) AS stages,
            (SELECT COUNT(*) FROM milestones m
                JOIN stages s ON s.id = m.stage_id
                JOIN roadmaps r ON r.id = s.roadmap_id
                WHERE r.project_id = $1) AS milestones,
            (SELECT COUNT(*) FROM learning_tasks WHERE project_id = $1) AS tasks
        "#,
    )
    .bind(project_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(counts)
}

/// 按依赖顺序物理删除项目的全部规划域数据。
///
/// DB 的 `ON DELETE CASCADE` 能兜住同样的结果，但显式删除让顺序与计数可控——
/// 将来加审计快照时，需要「删之前」的数字，级联不提供。
async fn cascade_delete_planning(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> AppResult<PlanningCounts> {
    let counts = count_planning(tx, project_id).await?;

    // 依赖行最先：两个外键都指向 learning_tasks
    sqlx::query(
        r#"
        DELETE FROM task_dependencies d
        USING learning_tasks t
        WHERE (d.task_id = t.id OR d.depends_on_task_id = t.id)
          AND t.project_id = $1
        "#,
    )
    .bind(project_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM learning_tasks WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        DELETE FROM goal_stages gs
        USING project_goals g
        WHERE gs.goal_id = g.id AND g.project_id = $1
        "#,
    )
    .bind(project_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM milestones m
        USING stages s, roadmaps r
        WHERE m.stage_id = s.id AND s.roadmap_id = r.id AND r.project_id = $1
        "#,
    )
    .bind(project_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM stages s
        USING roadmaps r
        WHERE s.roadmap_id = r.id AND r.project_id = $1
        "#,
    )
    .bind(project_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM roadmaps WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM project_goals WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut **tx)
        .await?;

    Ok(counts)
}

/// 删除草稿项目（D-02a）。仅 `status=draft`，级联物理删除，单事务。
///
/// 草稿不因存在子数据而拒绝（D-03）：草稿内子数据均为编辑中空壳，故无 `cascade` 参数。
pub async fn delete_draft(pool: &PgPool, owner_id: Uuid, project_id: Uuid) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM learning_projects WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))?;

    if ProjectStatus::parse(&status)? != ProjectStatus::Draft {
        return Err(AppError::Conflict(
            "仅草稿项目可直接删除；已实施项目请归档，或调用 purge 强制删除".to_string(),
        ));
    }

    let counts = cascade_delete_planning(&mut tx, project_id).await?;

    sqlx::query("DELETE FROM learning_projects WHERE id = $1 AND owner_id = $2")
        .bind(project_id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    tracing::info!(
        project_id = %project_id,
        goals = counts.goals,
        roadmaps = counts.roadmaps,
        stages = counts.stages,
        milestones = counts.milestones,
        tasks = counts.tasks,
        "删除草稿项目"
    );

    Ok(())
}

/// 强制删除项目（D-02b）。任意状态，须重输项目名，单事务。
pub async fn purge(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: PurgeProjectRequest,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT name, status FROM learning_projects WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(project_id)
    .bind(owner_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("项目不存在".to_string()))?;

    let (name, _status) = row;

    if payload.confirm_name != name {
        return Err(AppError::Validation(
            "confirm_name 与项目名称不一致".to_string(),
        ));
    }

    // deleting 只作行锁标记，随事务结束消失；常规写入一律拒绝该值（ADR-006 §2）
    sqlx::query("UPDATE learning_projects SET status = 'deleting' WHERE id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await?;

    let counts = cascade_delete_planning(&mut tx, project_id).await?;

    sqlx::query("DELETE FROM learning_projects WHERE id = $1 AND owner_id = $2")
        .bind(project_id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // v1 无审计表：purge 的结构化快照落 WARN 日志（ERD §8.1 step 9 的降级形态）
    tracing::warn!(
        project_id = %project_id,
        name = name.as_str(),
        goals = counts.goals,
        roadmaps = counts.roadmaps,
        stages = counts.stages,
        milestones = counts.milestones,
        tasks = counts.tasks,
        "purge 项目（含规划域全部数据）"
    );

    Ok(())
}

/// 供其他模块复用的时间戳类型别名，避免各处重复引入 chrono
pub type Timestamp = DateTime<Utc>;
