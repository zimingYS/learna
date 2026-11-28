//! 学习活动持久化：主表 + 三类关联（单事务）、软删、归属校验。
//!
//! 归属校验一律写进 SQL 的 `WHERE … owner_id = $n`：越权与不存在同为 404，
//! 不给调用方区分的机会。软删是 `UPDATE`，三类关联与证据全部保留（ADR-010 §3），
//! 查询层统一带 `deleted_at IS NULL`。

use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use crate::modules::learning_session::dto::{
    CreateSessionRequest, SessionContentInput, SessionContentItem, SessionContentRow,
    SessionDetailItem, SessionDetailRow, SessionItem, SessionListData, SessionQuery, SessionRow,
    SessionTaskInput, SessionTaskItem, SessionTaskRow, UpdateSessionRequest,
};
use crate::modules::learning_session::rules;
use crate::shared::{ActivityType, AppError, AppResult};

const LIST_COLUMNS: &str = r#"
    id, title, activity_type, started_at, ended_at, duration_minutes,
    primary_project_id, created_source, created_at
"#;

const DETAIL_COLUMNS: &str = r#"
    id, title, activity_type, started_at, ended_at, duration_minutes,
    primary_project_id, created_source, created_at,
    summary, what_done, problems, next_steps
"#;

/// 活动列表：默认排除软删，排序固定 `started_at DESC, id DESC`
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    query: SessionQuery,
) -> AppResult<SessionListData> {
    let (limit, offset) = query.window();
    let activity_type = match query.activity_type.as_deref() {
        Some(value) => Some(rules::parse_activity_type(value)?.as_str().to_string()),
        None => None,
    };

    // content_id 过滤走 EXISTS 而不是 JOIN：一条活动关联多个内容时不会重复出行
    let filter = r#"
        FROM learning_sessions
        WHERE owner_id = $1
          AND deleted_at IS NULL
          AND ($2::uuid IS NULL OR primary_project_id = $2)
          AND ($3::text IS NULL OR activity_type = $3)
          AND ($4::timestamptz IS NULL OR started_at >= $4)
          AND ($5::timestamptz IS NULL OR started_at < $5)
          AND ($6::uuid IS NULL OR EXISTS (
                SELECT 1 FROM session_contents sc
                WHERE sc.session_id = learning_sessions.id AND sc.content_id = $6
          ))
    "#;

    let sql = format!(
        "SELECT {LIST_COLUMNS} {filter} ORDER BY started_at DESC, id DESC LIMIT $7 OFFSET $8"
    );

    let rows = sqlx::query_as::<_, SessionRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(query.primary_project_id)
        .bind(activity_type.as_deref())
        .bind(query.from)
        .bind(query.to)
        .bind(query.content_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let count_sql = format!("SELECT COUNT(*) {filter}");
    let total = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql))
        .bind(owner_id)
        .bind(query.primary_project_id)
        .bind(activity_type.as_deref())
        .bind(query.from)
        .bind(query.to)
        .bind(query.content_id)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(SessionItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(SessionListData { items, total })
}

/// 活动详情：内联 `related_project_ids` / `contents` / `tasks`
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
) -> AppResult<SessionDetailItem> {
    let row = fetch_detail_row(pool, owner_id, session_id).await?;
    assemble(pool, row).await
}

/// 创建活动：主表 + 三类关联单事务，任一校验失败整体回滚
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateSessionRequest,
) -> AppResult<SessionDetailItem> {
    rules::ensure_interval(payload.started_at, payload.ended_at)?;
    let duration = rules::derive_duration(
        payload.started_at,
        payload.ended_at,
        payload.duration_minutes,
    )?;
    let source = rules::parse_api_source(payload.created_source.as_deref())?;
    let activity_type = match payload.activity_type.as_deref() {
        Some(value) => rules::parse_activity_type(value)?,
        None => ActivityType::Mixed,
    };
    let title = rules::normalize_title(payload.title)?;

    let related = rules::dedupe_ids(&payload.related_project_ids);
    rules::ensure_related_excludes_primary(payload.primary_project_id, &related)?;
    rules::ensure_unique_contents(&payload.contents)?;
    rules::ensure_unique_tasks(&payload.tasks)?;
    rules::ensure_allocation_within_duration(&payload.tasks, duration)?;

    // 贡献类型先整体解析：写入前就把拼错的值挡掉
    for item in &payload.contents {
        rules::parse_contribution(&item.contribution_type)?;
    }

    let mut projects = related.clone();
    if let Some(primary) = payload.primary_project_id {
        projects.push(primary);
    }

    let mut tx = pool.begin().await?;

    ensure_projects_owned(&mut tx, owner_id, &projects).await?;
    ensure_contents_owned(&mut tx, owner_id, &payload.contents).await?;
    ensure_tasks_attributable(&mut tx, owner_id, &projects, &payload.tasks).await?;

    let session_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO learning_sessions (
            owner_id, primary_project_id, title, activity_type,
            started_at, ended_at, duration_minutes,
            summary, what_done, problems, next_steps, created_source
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .bind(payload.primary_project_id)
    .bind(title)
    .bind(activity_type.as_str())
    .bind(payload.started_at)
    .bind(payload.ended_at)
    .bind(duration)
    .bind(rules::normalize_text(payload.summary))
    .bind(rules::normalize_text(payload.what_done))
    .bind(rules::normalize_text(payload.problems))
    .bind(rules::normalize_text(payload.next_steps))
    .bind(source.as_str())
    .fetch_one(&mut *tx)
    .await?;

    insert_related_projects(&mut tx, session_id, &related).await?;
    insert_contents(&mut tx, session_id, &payload.contents).await?;
    insert_tasks(&mut tx, session_id, &payload.tasks).await?;

    tx.commit().await?;

    detail(pool, owner_id, session_id).await
}

/// 部分更新（ADR-002 三态）。
///
/// 改时间或时长走同一套校验：新区间必须合法，且既有时长仍不得超过区间（ADR-010 §1）。
/// 关联不在此端点修改——三个 `PUT` 子资源负责。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
    payload: UpdateSessionRequest,
) -> AppResult<SessionDetailItem> {
    let mut tx = pool.begin().await?;
    let current = fetch_detail_row_locked(&mut tx, owner_id, session_id).await?;

    let started_at = payload.started_at.unwrap_or(current.started_at);
    let ended_at = payload.ended_at.unwrap_or(current.ended_at);
    rules::ensure_interval(started_at, ended_at)?;
    let duration = payload.duration_minutes.unwrap_or(current.duration_minutes);
    rules::ensure_duration_fits(duration, started_at, ended_at)?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE learning_sessions SET ");
    let mut separated = builder.separated(", ");
    let mut touched = false;

    if let Some(title) = payload.title {
        let title = rules::normalize_title(title)?;
        if title != current.title {
            separated.push("title = ").push_bind_unseparated(title);
            touched = true;
        }
    }

    if let Some(value) = payload.activity_type.as_deref() {
        let activity_type = rules::parse_activity_type(value)?;
        if activity_type.as_str() != current.activity_type {
            separated
                .push("activity_type = ")
                .push_bind_unseparated(activity_type.as_str().to_string());
            touched = true;
        }
    }

    if started_at != current.started_at {
        separated
            .push("started_at = ")
            .push_bind_unseparated(started_at);
        touched = true;
    }

    if ended_at != current.ended_at {
        separated
            .push("ended_at = ")
            .push_bind_unseparated(ended_at);
        touched = true;
    }

    if duration != current.duration_minutes {
        separated
            .push("duration_minutes = ")
            .push_bind_unseparated(duration);
        touched = true;
    }

    if let Some(primary) = payload.primary_project_id
        && primary != current.primary_project_id
    {
        if let Some(project_id) = primary {
            ensure_projects_owned(&mut tx, owner_id, &[project_id]).await?;
            // 主要项目与关联项目是同一事实的两种表达，重复会让归因分叉
            let listed = sqlx::query_scalar::<_, i32>(
                "SELECT 1 FROM session_projects WHERE session_id = $1 AND project_id = $2",
            )
            .bind(session_id)
            .bind(project_id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();

            if listed {
                return Err(AppError::Validation(
                    "该项目已作为关联项目存在；请先从关联项目列表移除，再设为主要项目".to_string(),
                ));
            }
        }

        // 换主要项目可能让既有任务分摊失去归属
        let mut allowed = sqlx::query_scalar::<_, Uuid>(
            "SELECT project_id FROM session_projects WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_all(&mut *tx)
        .await?;

        if let Some(project_id) = primary {
            allowed.push(project_id);
        }
        ensure_existing_tasks_attributable(&mut tx, session_id, &allowed).await?;

        separated
            .push("primary_project_id = ")
            .push_bind_unseparated(primary);
        touched = true;
    }

    if let Some(summary) = payload.summary {
        let summary = rules::normalize_text(summary);
        if summary != current.summary {
            separated.push("summary = ").push_bind_unseparated(summary);
            touched = true;
        }
    }

    if let Some(what_done) = payload.what_done {
        let what_done = rules::normalize_text(what_done);
        if what_done != current.what_done {
            separated
                .push("what_done = ")
                .push_bind_unseparated(what_done);
            touched = true;
        }
    }

    if let Some(problems) = payload.problems {
        let problems = rules::normalize_text(problems);
        if problems != current.problems {
            separated
                .push("problems = ")
                .push_bind_unseparated(problems);
            touched = true;
        }
    }

    if let Some(next_steps) = payload.next_steps {
        let next_steps = rules::normalize_text(next_steps);
        if next_steps != current.next_steps {
            separated
                .push("next_steps = ")
                .push_bind_unseparated(next_steps);
            touched = true;
        }
    }

    if !touched {
        tx.commit().await?;
        return detail(pool, owner_id, session_id).await;
    }

    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(session_id)
        .push(" AND owner_id = ")
        .push_bind(owner_id)
        .push(" AND deleted_at IS NULL");

    builder.build().execute(&mut *tx).await?;
    tx.commit().await?;

    detail(pool, owner_id, session_id).await
}

/// 软删：`deleted_at` 打标记，关联行与证据全部保留（ADR-010 决策 3）。
///
/// 重复删除返回 200（行还在，能区分「已删」与「不存在」）。
pub async fn delete(pool: &PgPool, owner_id: Uuid, session_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE learning_sessions
        SET deleted_at = now(), updated_at = now()
        WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(session_id)
    .bind(owner_id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected > 0 {
        return Ok(());
    }

    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_sessions WHERE id = $1 AND owner_id = $2",
    )
    .bind(session_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if exists {
        return Ok(());
    }

    Err(AppError::NotFound("学习活动不存在".to_string()))
}

/// 取详情行，归属与软删过滤都写进 `WHERE`
async fn fetch_detail_row(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
) -> AppResult<SessionDetailRow> {
    let sql = format!(
        r#"
        SELECT {DETAIL_COLUMNS}
        FROM learning_sessions
        WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
        "#
    );

    sqlx::query_as::<_, SessionDetailRow>(sqlx::AssertSqlSafe(sql))
        .bind(session_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("学习活动不存在".to_string()))
}

/// 同 [`fetch_detail_row`]，但锁行以固定「读当前态 → 校验 → 写入」的并发窗口
async fn fetch_detail_row_locked(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: Uuid,
    session_id: Uuid,
) -> AppResult<SessionDetailRow> {
    let sql = format!(
        r#"
        SELECT {DETAIL_COLUMNS}
        FROM learning_sessions
        WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
        FOR UPDATE
        "#
    );

    sqlx::query_as::<_, SessionDetailRow>(sqlx::AssertSqlSafe(sql))
        .bind(session_id)
        .bind(owner_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("学习活动不存在".to_string()))
}

/// 主行 + 三组关联拼成详情
async fn assemble(pool: &PgPool, row: SessionDetailRow) -> AppResult<SessionDetailItem> {
    let session_id = row.id;

    let related_project_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT project_id FROM session_projects WHERE session_id = $1 ORDER BY project_id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let contents = sqlx::query_as::<_, SessionContentRow>(
        r#"
        SELECT content_id, contribution_type, note
        FROM session_contents
        WHERE session_id = $1
        ORDER BY content_id
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(SessionContentItem::try_from)
    .collect::<AppResult<Vec<_>>>()?;

    let tasks = sqlx::query_as::<_, SessionTaskRow>(
        "SELECT task_id, contribution_minutes, note FROM task_sessions WHERE session_id = $1 ORDER BY task_id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(SessionTaskItem::from)
    .collect();

    SessionDetailItem::new(row, related_project_ids, contents, tasks)
}

/// 项目全部属于当前用户；数量不符即 404（不区分「不存在」与「不是你的」）
pub(super) async fn ensure_projects_owned(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: Uuid,
    project_ids: &[Uuid],
) -> AppResult<()> {
    if project_ids.is_empty() {
        return Ok(());
    }

    let found = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM learning_projects WHERE id = ANY($1) AND owner_id = $2",
    )
    .bind(project_ids)
    .bind(owner_id)
    .fetch_one(&mut **tx)
    .await?;

    if found != project_ids.len() as i64 {
        return Err(AppError::NotFound("项目不存在".to_string()));
    }
    Ok(())
}

/// 内容全部属于当前用户
pub(super) async fn ensure_contents_owned(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: Uuid,
    items: &[SessionContentInput],
) -> AppResult<()> {
    if items.is_empty() {
        return Ok(());
    }

    let ids: Vec<Uuid> = items.iter().map(|item| item.content_id).collect();
    let found = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM learning_contents WHERE id = ANY($1) AND owner_id = $2",
    )
    .bind(&ids)
    .bind(owner_id)
    .fetch_one(&mut **tx)
    .await?;

    if found != ids.len() as i64 {
        return Err(AppError::NotFound("内容不存在".to_string()));
    }
    Ok(())
}

/// 任务必须属于本次活动的项目集合（主要 + 关联）且未软删。
///
/// 归因不成立属组合校验，统一 422——不区分「任务不存在」「别人的任务」「别的项目的任务」。
pub(super) async fn ensure_tasks_attributable(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: Uuid,
    project_ids: &[Uuid],
    items: &[SessionTaskInput],
) -> AppResult<()> {
    if items.is_empty() {
        return Ok(());
    }

    let ids: Vec<Uuid> = items.iter().map(|item| item.task_id).collect();
    let found = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM learning_tasks t
        JOIN learning_projects p ON p.id = t.project_id
        WHERE t.id = ANY($1) AND t.deleted_at IS NULL
          AND p.owner_id = $2 AND t.project_id = ANY($3)
        "#,
    )
    .bind(&ids)
    .bind(owner_id)
    .bind(project_ids)
    .fetch_one(&mut **tx)
    .await?;

    if found != ids.len() as i64 {
        return Err(AppError::Validation(
            "任务不属于本次活动的关联项目".to_string(),
        ));
    }
    Ok(())
}

/// 已有任务分摊必须仍能归因到活动项目集合。
///
/// 改项目集合或主要项目都可能让分摊行失去归属；这里不静默删除分摊行，
/// 而是要求先调整分摊——与「不静默覆盖用户数据」一致。
pub(super) async fn ensure_existing_tasks_attributable(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    project_ids: &[Uuid],
) -> AppResult<()> {
    let orphaned = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM task_sessions ts
        JOIN learning_tasks t ON t.id = ts.task_id
        WHERE ts.session_id = $1 AND t.project_id <> ALL($2)
        "#,
    )
    .bind(session_id)
    .bind(project_ids)
    .fetch_one(&mut **tx)
    .await?;

    if orphaned > 0 {
        return Err(AppError::Validation(
            "已有任务分摊不属于新的项目集合；请先调整任务分摊".to_string(),
        ));
    }
    Ok(())
}

/// `session_projects` 只存 related（主要项目在主字段），`relation_role` 恒为 related
pub(super) async fn insert_related_projects(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    project_ids: &[Uuid],
) -> AppResult<()> {
    for project_id in project_ids {
        sqlx::query(
            "INSERT INTO session_projects (session_id, project_id, relation_role) VALUES ($1, $2, 'related')",
        )
        .bind(session_id)
        .bind(project_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(super) async fn insert_contents(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    items: &[SessionContentInput],
) -> AppResult<()> {
    for item in items {
        sqlx::query(
            "INSERT INTO session_contents (session_id, content_id, contribution_type, note) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(item.content_id)
        .bind(&item.contribution_type)
        .bind(rules::normalize_text(item.note.clone()))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(super) async fn insert_tasks(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    items: &[SessionTaskInput],
) -> AppResult<()> {
    for item in items {
        sqlx::query(
            "INSERT INTO task_sessions (task_id, session_id, contribution_minutes, note) VALUES ($1, $2, $3, $4)",
        )
        .bind(item.task_id)
        .bind(session_id)
        .bind(item.contribution_minutes)
        .bind(rules::normalize_text(item.note.clone()))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// 校验活动存在、属于当前用户且未软删；返回值供关联替换使用
pub(super) async fn lock_session(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: Uuid,
    session_id: Uuid,
) -> AppResult<LockedSession> {
    let row = sqlx::query_as::<_, (Option<Uuid>, i32)>(
        r#"
        SELECT primary_project_id, duration_minutes
        FROM learning_sessions
        WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(session_id)
    .bind(owner_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("学习活动不存在".to_string()))?;

    Ok(LockedSession {
        primary_project_id: row.0,
        duration_minutes: row.1,
    })
}

/// 关联替换需要的活动上下文
pub(super) struct LockedSession {
    pub primary_project_id: Option<Uuid>,
    pub duration_minutes: i32,
}
