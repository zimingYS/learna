//! 项目内容上下文：关联（幂等）、项目内状态迁移、跨项目复用查询。
//!
//! 状态是**项目级**的：同一份内容在 A 项目 `achieved`、B 项目 `not_started` 是正常状态，
//! 而不是数据不一致（ERD §3.2）。

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::modules::project_content::dto::{
    CreateProjectContentRequest, ProjectContentItem, ProjectContentListData, ProjectContentQuery,
    ProjectContentRow, UpdateProjectContentRequest,
};
use crate::modules::project_content::rules;
use crate::shared::{AppError, AppResult, ProjectContentStatus};

const COLUMNS: &str = r#"
    id, project_id, content_id, current_status, target_mastery_level,
    priority, auto_review_enabled, linked_at, achieved_at
"#;

/// 关联列表：按项目看进度，或按内容看「它在哪些项目里」
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    query: ProjectContentQuery,
) -> AppResult<ProjectContentListData> {
    match (query.project_id, query.content_id) {
        (Some(_), Some(_)) | (None, None) => {
            return Err(AppError::Validation(
                "project_id 与 content_id 必须且只能提供一个".to_string(),
            ));
        }
        _ => {}
    }

    if let Some(project_id) = query.project_id {
        ensure_project(pool, owner_id, project_id).await?;
    }
    if let Some(content_id) = query.content_id {
        ensure_content(pool, owner_id, content_id).await?;
    }

    let (limit, offset) = query.window();
    let status = query.current_status.map(|value| value.as_str().to_string());

    let filter = r#"
        FROM project_contents pc
        JOIN learning_projects p ON p.id = pc.project_id
        WHERE p.owner_id = $1
          AND ($2::uuid IS NULL OR pc.project_id = $2)
          AND ($3::uuid IS NULL OR pc.content_id = $3)
          AND ($4::text IS NULL OR pc.current_status = $4)
    "#;

    let sql = format!(
        r#"
        SELECT pc.id, pc.project_id, pc.content_id, pc.current_status,
               pc.target_mastery_level, pc.priority, pc.auto_review_enabled,
               pc.linked_at, pc.achieved_at
        {filter}
        ORDER BY pc.linked_at DESC, pc.id DESC
        LIMIT $5 OFFSET $6
        "#
    );

    let rows = sqlx::query_as::<_, ProjectContentRow>(sqlx::AssertSqlSafe(sql))
        .bind(owner_id)
        .bind(query.project_id)
        .bind(query.content_id)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let count_sql = format!("SELECT COUNT(*) {filter}");
    let total = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql))
        .bind(owner_id)
        .bind(query.project_id)
        .bind(query.content_id)
        .bind(&status)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(ProjectContentItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(ProjectContentListData { items, total })
}

/// 内容加入项目。
///
/// 归属校验与归档检查都先于写入：项目 / 内容任一不属于当前用户 → 404（三者不可区分），
/// 内容已归档 → 409（归档语义 = 不再使用）。重复关联幂等返回既有行。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    payload: CreateProjectContentRequest,
) -> AppResult<ProjectContentItem> {
    let content_status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT c.status
        FROM learning_projects p, learning_contents c
        WHERE p.id = $1 AND c.id = $2 AND p.owner_id = $3 AND c.owner_id = $3
        "#,
    )
    .bind(payload.project_id)
    .bind(payload.content_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("项目或内容不存在".to_string()))?;

    if content_status == "archived" {
        return Err(AppError::Conflict("已归档的内容不能加入项目".to_string()));
    }

    let mastery = rules::ensure_mastery_level(payload.target_mastery_level.unwrap_or(0))?;
    let priority = rules::ensure_priority(payload.priority.unwrap_or(3))?;
    let auto_review = payload.auto_review_enabled.unwrap_or(false);

    let sql = format!(
        r#"
        INSERT INTO project_contents (
            project_id, content_id, target_mastery_level, current_status, priority,
            auto_review_enabled, linked_at
        ) VALUES ($1, $2, $3, 'not_started', $4, $5, now())
        ON CONFLICT (project_id, content_id) DO NOTHING
        RETURNING {COLUMNS}
        "#
    );

    let inserted = sqlx::query_as::<_, ProjectContentRow>(sqlx::AssertSqlSafe(sql))
        .bind(payload.project_id)
        .bind(payload.content_id)
        .bind(mastery)
        .bind(priority)
        .bind(auto_review)
        .fetch_optional(pool)
        .await?;

    // 已存在时 ON CONFLICT 不返回行，取既有行作为幂等结果
    let row = match inserted {
        Some(row) => row,
        None => sqlx::query_as::<_, ProjectContentRow>(
            r#"
            SELECT pc.id, pc.project_id, pc.content_id, pc.current_status,
                   pc.target_mastery_level, pc.priority, pc.auto_review_enabled,
                   pc.linked_at, pc.achieved_at
            FROM project_contents pc
            JOIN learning_projects p ON p.id = pc.project_id
            WHERE pc.project_id = $1 AND pc.content_id = $2 AND p.owner_id = $3
            "#,
        )
        .bind(payload.project_id)
        .bind(payload.content_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("关联不存在".to_string()))?,
    };

    ProjectContentItem::try_from(row)
}

/// 关联详情
pub async fn detail(pool: &PgPool, owner_id: Uuid, id: Uuid) -> AppResult<ProjectContentItem> {
    let row = fetch_row(pool, owner_id, id).await?;
    ProjectContentItem::try_from(row)
}

/// 更新项目内状态与目标。
///
/// `achieved` 只能由 M06 评价流程写入（D-27），用户提交即 409；同态提交幂等，
/// 且**不刷新** `updated_at`——否则前端每次保存都会把「最后更新」往前推。
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    id: Uuid,
    payload: UpdateProjectContentRequest,
) -> AppResult<ProjectContentItem> {
    let mut tx = pool.begin().await?;

    let current = fetch_row_locked(&mut tx, owner_id, id).await?;
    let current_status = ProjectContentStatus::parse(&current.current_status)?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE project_contents SET ");
    let mut separated = builder.separated(", ");
    let mut touched = false;

    if let Some(target_status) = payload.current_status {
        rules::ensure_user_writable_status(target_status)?;
        if target_status != current_status {
            separated
                .push("current_status = ")
                .push_bind_unseparated(target_status.as_str().to_string());
            touched = true;
        }
    }

    if let Some(level) = payload.target_mastery_level {
        let level = rules::ensure_mastery_level(level)?;
        if level != current.target_mastery_level {
            separated
                .push("target_mastery_level = ")
                .push_bind_unseparated(level);
            touched = true;
        }
    }

    if let Some(priority) = payload.priority {
        let priority = rules::ensure_priority(priority)?;
        if priority != current.priority {
            separated
                .push("priority = ")
                .push_bind_unseparated(priority);
            touched = true;
        }
    }

    if let Some(auto_review) = payload.auto_review_enabled
        && auto_review != current.auto_review_enabled
    {
        separated
            .push("auto_review_enabled = ")
            .push_bind_unseparated(auto_review);
        touched = true;
    }

    if !touched {
        tx.commit().await?;
        return ProjectContentItem::try_from(current);
    }

    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(id)
        .push(format!(" RETURNING {COLUMNS}"));

    let row = builder
        .build_query_as::<ProjectContentRow>()
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("关联不存在".to_string()))?;

    tx.commit().await?;
    ProjectContentItem::try_from(row)
}

/// 解除关联：物理删关联行，**不删内容本体**，也不动 `content_relations`
pub async fn delete(pool: &PgPool, owner_id: Uuid, id: Uuid) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        DELETE FROM project_contents pc
        USING learning_projects p
        WHERE pc.id = $1 AND p.id = pc.project_id AND p.owner_id = $2
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("关联不存在".to_string()));
    }
    Ok(())
}

/// 取关联行，归属通过所属项目的 `owner_id` 判定
async fn fetch_row(pool: &PgPool, owner_id: Uuid, id: Uuid) -> AppResult<ProjectContentRow> {
    let sql = r#"
        SELECT pc.id, pc.project_id, pc.content_id, pc.current_status,
               pc.target_mastery_level, pc.priority, pc.auto_review_enabled,
               pc.linked_at, pc.achieved_at
        FROM project_contents pc
        JOIN learning_projects p ON p.id = pc.project_id
        WHERE pc.id = $1 AND p.owner_id = $2
    "#;

    sqlx::query_as::<_, ProjectContentRow>(sql)
        .bind(id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("关联不存在".to_string()))
}

/// 同 [`fetch_row`]，但锁行以固定当前态与写入之间的并发窗口
async fn fetch_row_locked(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    owner_id: Uuid,
    id: Uuid,
) -> AppResult<ProjectContentRow> {
    let sql = r#"
        SELECT pc.id, pc.project_id, pc.content_id, pc.current_status,
               pc.target_mastery_level, pc.priority, pc.auto_review_enabled,
               pc.linked_at, pc.achieved_at
        FROM project_contents pc
        JOIN learning_projects p ON p.id = pc.project_id
        WHERE pc.id = $1 AND p.owner_id = $2
        FOR UPDATE OF pc
    "#;

    sqlx::query_as::<_, ProjectContentRow>(sql)
        .bind(id)
        .bind(owner_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("关联不存在".to_string()))
}

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

async fn ensure_content(pool: &PgPool, owner_id: Uuid, content_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM learning_contents WHERE id = $1 AND owner_id = $2",
    )
    .bind(content_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    if !exists {
        return Err(AppError::NotFound("内容不存在".to_string()));
    }
    Ok(())
}
