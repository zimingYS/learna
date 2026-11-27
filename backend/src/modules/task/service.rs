//! 任务（LearningTask）与依赖（TaskDependency）的持久化。
//!
//! 三条贯穿全文件的规则：
//! 1. **软删行不参与任何查询与计数**（D-17），每个 SELECT 都带 `deleted_at IS NULL`；
//! 2. **任务不随上层结构消失**：删阶段 / 里程碑 / 路线只解挂引用，行保留（L3）；
//! 3. **依赖成环检测在事务内加行锁后执行**，否则两个并发请求各自检测通过、组合成环。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::modules::task::dto::{
    CreateTaskDependencyRequest, CreateTaskRequest, TaskDependencyItem, TaskDependencyListData,
    TaskDependencyRow, TaskItem, TaskListData, TaskQuery, TaskRow, UpdateTaskRequest,
};
use crate::modules::task::graph;
use crate::modules::task::rules::{self, TransitionOutcome};
use crate::shared::{AppError, AppResult, CreatedSource, DependencyType, TaskStatus};

const TASK_COLUMNS: &str = r#"
    id, project_id, milestone_id, stage_id, title, description, status, priority,
    due_at, estimated_minutes, completed_at, created_source, created_at, updated_at
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
    task_id: Uuid,
) -> AppResult<TaskRow> {
    let sql = format!(
        r#"
        SELECT {TASK_COLUMNS}
        FROM learning_tasks t
        WHERE t.id = $1
          AND t.project_id = $2
          AND t.deleted_at IS NULL
          AND EXISTS (
              SELECT 1 FROM learning_projects p
              WHERE p.id = t.project_id AND p.owner_id = $3
          )
        "#
    );

    sqlx::query_as::<_, TaskRow>(sqlx::AssertSqlSafe(sql))
        .bind(task_id)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("任务不存在".to_string()))
}

/// 校验阶段归属，返回其存在性
async fn ensure_stage_in_project(pool: &PgPool, project_id: Uuid, stage_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1 FROM stages s
        JOIN roadmaps r ON r.id = s.roadmap_id
        WHERE s.id = $1 AND r.project_id = $2
        "#,
    )
    .bind(stage_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::Validation("stage_id 不属于该项目".to_string()));
    }
    Ok(())
}

/// 校验里程碑归属并返回其所属阶段
async fn fetch_milestone_stage(
    pool: &PgPool,
    project_id: Uuid,
    milestone_id: Uuid,
) -> AppResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT m.stage_id FROM milestones m
        JOIN stages s ON s.id = m.stage_id
        JOIN roadmaps r ON r.id = s.roadmap_id
        WHERE m.id = $1 AND r.project_id = $2
        "#,
    )
    .bind(milestone_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("milestone_id 不属于该项目".to_string()))
}

/// 校验 `stage_id` 与 `milestone_id` 的组合一致（D-05）：
/// 两者同时存在时，阶段须与里程碑所属阶段一致。
async fn ensure_stage_milestone_consistent(
    pool: &PgPool,
    project_id: Uuid,
    stage_id: Option<Uuid>,
    milestone_id: Option<Uuid>,
) -> AppResult<()> {
    if let Some(stage_id) = stage_id {
        ensure_stage_in_project(pool, project_id, stage_id).await?;
    }

    if let Some(milestone_id) = milestone_id {
        let milestone_stage = fetch_milestone_stage(pool, project_id, milestone_id).await?;
        if let Some(stage_id) = stage_id
            && stage_id != milestone_stage
        {
            return Err(AppError::Validation(
                "stage_id 与 milestone_id 所属阶段不一致".to_string(),
            ));
        }
    }

    Ok(())
}

/// 任务列表（不含软删行；排序 priority DESC, created_at DESC, id DESC）
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    query: &TaskQuery,
    limit: i64,
    offset: i64,
) -> AppResult<TaskListData> {
    ensure_project(pool, owner_id, project_id).await?;

    let status_filter = query.status.map(|s| s.as_str().to_string());

    let sql = format!(
        r#"
        SELECT {TASK_COLUMNS}
        FROM learning_tasks
        WHERE project_id = $1
          AND deleted_at IS NULL
          AND ($2::text IS NULL OR status = $2)
          AND ($3::uuid IS NULL OR milestone_id = $3)
          AND ($4::uuid IS NULL OR stage_id = $4)
        ORDER BY priority DESC, created_at DESC, id DESC
        LIMIT $5 OFFSET $6
        "#
    );

    let rows = sqlx::query_as::<_, TaskRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(&status_filter)
        .bind(query.milestone_id)
        .bind(query.stage_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM learning_tasks
        WHERE project_id = $1
          AND deleted_at IS NULL
          AND ($2::text IS NULL OR status = $2)
          AND ($3::uuid IS NULL OR milestone_id = $3)
          AND ($4::uuid IS NULL OR stage_id = $4)
        "#,
    )
    .bind(project_id)
    .bind(&status_filter)
    .bind(query.milestone_id)
    .bind(query.stage_id)
    .fetch_one(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(TaskItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(TaskListData { items, total })
}

/// 创建任务（`created_source=user`，`status=todo`，`priority` 默认 3）
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: CreateTaskRequest,
) -> AppResult<TaskItem> {
    ensure_project(pool, owner_id, project_id).await?;

    let title = crate::modules::roadmap::rules::ensure_title("title", &payload.title, 200)?;
    let priority = payload.priority.unwrap_or(3);
    rules::ensure_priority(priority)?;

    if let Some(minutes) = payload.estimated_minutes {
        rules::ensure_estimated_minutes(minutes)?;
    }

    ensure_stage_milestone_consistent(pool, project_id, payload.stage_id, payload.milestone_id)
        .await?;

    let sql = format!(
        r#"
        INSERT INTO learning_tasks (
            project_id, milestone_id, stage_id, title, description, status,
            priority, due_at, estimated_minutes, created_source
        ) VALUES ($1, $2, $3, $4, $5, 'todo', $6, $7, $8, $9)
        RETURNING {TASK_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, TaskRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(payload.milestone_id)
        .bind(payload.stage_id)
        .bind(title)
        .bind(payload.description)
        .bind(priority)
        .bind(payload.due_at)
        .bind(payload.estimated_minutes)
        .bind(CreatedSource::User.as_str())
        .fetch_one(pool)
        .await?;

    TaskItem::try_from(row)
}

/// 任务详情
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
) -> AppResult<TaskItem> {
    let row = fetch_row(pool, owner_id, project_id, task_id).await?;
    TaskItem::try_from(row)
}

/// 更新任务（含状态迁移）。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
    payload: UpdateTaskRequest,
) -> AppResult<TaskItem> {
    let current = fetch_row(pool, owner_id, project_id, task_id).await?;

    let current_status = TaskStatus::parse(&current.status)?;
    let target_status = payload.status.unwrap_or(current_status);
    let outcome = rules::evaluate_task(current_status, target_status)?;

    let title = match payload.title {
        Some(value) => Some(crate::modules::roadmap::rules::ensure_title(
            "title", &value, 200,
        )?),
        None => None,
    };

    if let Some(priority) = payload.priority {
        rules::ensure_priority(priority)?;
    }

    if let Some(Some(minutes)) = payload.estimated_minutes {
        rules::ensure_estimated_minutes(minutes)?;
    }

    // 归属变更需按合并后的结果校验一致性
    let next_stage_id = match payload.stage_id {
        Some(value) => value,
        None => current.stage_id,
    };
    let next_milestone_id = match payload.milestone_id {
        Some(value) => value,
        None => current.milestone_id,
    };

    if payload.stage_id.is_some() || payload.milestone_id.is_some() {
        ensure_stage_milestone_consistent(pool, project_id, next_stage_id, next_milestone_id)
            .await?;
    }

    let description_set = payload.description.is_some();
    let description = payload.description.flatten();
    let due_at_set = payload.due_at.is_some();
    let due_at = payload.due_at.flatten();
    let estimated_set = payload.estimated_minutes.is_some();
    let estimated = payload.estimated_minutes.flatten();

    let nothing_to_do = outcome == TransitionOutcome::Noop
        && title.is_none()
        && !description_set
        && payload.stage_id.is_none()
        && payload.milestone_id.is_none()
        && payload.priority.is_none()
        && !due_at_set
        && !estimated_set;

    if nothing_to_do {
        return TaskItem::try_from(current);
    }

    let sql = format!(
        r#"
        UPDATE learning_tasks
        SET title = COALESCE($2::text, title),
            description = CASE WHEN $3::bool THEN $4::text ELSE description END,
            milestone_id = CASE WHEN $5::bool THEN $6::uuid ELSE milestone_id END,
            stage_id = CASE WHEN $7::bool THEN $8::uuid ELSE stage_id END,
            priority = COALESCE($9::smallint, priority),
            due_at = CASE WHEN $10::bool THEN $11::timestamptz ELSE due_at END,
            estimated_minutes = CASE WHEN $12::bool THEN $13::int ELSE estimated_minutes END,
            status = $14,
            completed_at = CASE
                WHEN $15 THEN now()
                WHEN $16 THEN NULL
                ELSE completed_at
            END,
            updated_at = now()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING {TASK_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, TaskRow>(sqlx::AssertSqlSafe(sql))
        .bind(task_id)
        .bind(title)
        .bind(description_set)
        .bind(description)
        .bind(payload.milestone_id.is_some())
        .bind(payload.milestone_id.flatten())
        .bind(payload.stage_id.is_some())
        .bind(payload.stage_id.flatten())
        .bind(payload.priority)
        .bind(due_at_set)
        .bind(due_at)
        .bind(estimated_set)
        .bind(estimated)
        .bind(target_status.as_str())
        .bind(outcome == TransitionOutcome::EnterCompleted)
        .bind(outcome == TransitionOutcome::LeaveCompleted)
        .fetch_one(pool)
        .await?;

    TaskItem::try_from(row)
}

/// 软删除任务（L3）+ 显式物理删依赖行。
///
/// 软删是 `UPDATE`，**不会触发** `ON DELETE CASCADE`，所以依赖行必须在这里删干净，
/// 否则依赖环路检测会读到悬空边。
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
) -> AppResult<()> {
    fetch_row(pool, owner_id, project_id, task_id).await?;

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        DELETE FROM task_dependencies
        WHERE task_id = $1 OR depends_on_task_id = $1
        "#,
    )
    .bind(task_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE learning_tasks SET deleted_at = now(), updated_at = now() WHERE id = $1")
        .bind(task_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 前置依赖列表
pub async fn list_dependencies(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
) -> AppResult<TaskDependencyListData> {
    fetch_row(pool, owner_id, project_id, task_id).await?;

    let rows = sqlx::query_as::<_, TaskDependencyRow>(
        r#"
        SELECT task_id, depends_on_task_id, dependency_type
        FROM task_dependencies
        WHERE task_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(TaskDependencyItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(TaskDependencyListData { items })
}

/// 新增依赖：同项目、禁自依赖、禁成环。整段在事务内对两个任务行加锁。
pub async fn create_dependency(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
    payload: CreateTaskDependencyRequest,
) -> AppResult<TaskDependencyItem> {
    fetch_row(pool, owner_id, project_id, task_id).await?;

    let depends_on = payload.depends_on_task_id;

    if depends_on == task_id {
        return Err(AppError::Validation("任务不能依赖自身".to_string()));
    }

    let mut tx = pool.begin().await?;

    // 固定顺序加锁（按 id 排序），避免两个并发请求以相反顺序互等
    let mut lock_ids = vec![task_id, depends_on];
    lock_ids.sort_unstable();
    let found = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id FROM learning_tasks
        WHERE id = ANY($1) AND project_id = $2 AND deleted_at IS NULL
        ORDER BY id
        FOR UPDATE
        "#,
    )
    .bind(&lock_ids)
    .bind(project_id)
    .fetch_all(&mut *tx)
    .await?;

    if found.len() != 2 {
        return Err(AppError::Validation(
            "depends_on_task_id 必须是同项目内未被删除的任务".to_string(),
        ));
    }

    if edge_exists(&mut tx, task_id, depends_on).await? {
        // 重复添加同一依赖：按幂等返回已存在行，不报冲突
        let row = fetch_dependency(&mut tx, task_id, depends_on).await?;
        tx.commit().await?;
        return TaskDependencyItem::try_from(row);
    }

    let edges = load_edges(&mut tx, project_id).await?;
    let graph = graph::build_graph(&edges);

    if graph::would_cycle(&graph, task_id, depends_on) {
        return Err(AppError::Validation(
            "该依赖会形成环路，首版不允许循环依赖".to_string(),
        ));
    }

    let dependency_type = payload
        .dependency_type
        .unwrap_or(DependencyType::FinishToStart);

    let row = sqlx::query_as::<_, TaskDependencyRow>(
        r#"
        INSERT INTO task_dependencies (task_id, depends_on_task_id, dependency_type)
        VALUES ($1, $2, $3)
        RETURNING task_id, depends_on_task_id, dependency_type
        "#,
    )
    .bind(task_id)
    .bind(depends_on)
    .bind(dependency_type.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    TaskDependencyItem::try_from(row)
}

async fn edge_exists(
    tx: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
    depends_on: Uuid,
) -> AppResult<bool> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM task_dependencies WHERE task_id = $1 AND depends_on_task_id = $2",
    )
    .bind(task_id)
    .bind(depends_on)
    .fetch_optional(&mut **tx)
    .await?
    .is_some();

    Ok(exists)
}

async fn fetch_dependency(
    tx: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
    depends_on: Uuid,
) -> AppResult<TaskDependencyRow> {
    sqlx::query_as::<_, TaskDependencyRow>(
        r#"
        SELECT task_id, depends_on_task_id, dependency_type
        FROM task_dependencies
        WHERE task_id = $1 AND depends_on_task_id = $2
        "#,
    )
    .bind(task_id)
    .bind(depends_on)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)
}

/// 加载项目内（两端都未软删）的全部依赖边
async fn load_edges(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> AppResult<Vec<(Uuid, Uuid)>> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT d.task_id, d.depends_on_task_id
        FROM task_dependencies d
        JOIN learning_tasks t ON t.id = d.task_id
        JOIN learning_tasks p ON p.id = d.depends_on_task_id
        WHERE t.project_id = $1
          AND t.deleted_at IS NULL
          AND p.deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows)
}

/// 删除依赖边
pub async fn delete_dependency(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
    depends_on_task_id: Uuid,
) -> AppResult<()> {
    fetch_row(pool, owner_id, project_id, task_id).await?;

    let result =
        sqlx::query("DELETE FROM task_dependencies WHERE task_id = $1 AND depends_on_task_id = $2")
            .bind(task_id)
            .bind(depends_on_task_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("依赖不存在".to_string()));
    }

    Ok(())
}
