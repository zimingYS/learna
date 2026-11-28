//! 三类关联的整体替换（`PUT`）：删旧 → 校验新 → 插入，全程单事务。
//!
//! 逐条增删会让前端维护差集，失败时还容易留下半成品；一次活动关联的内容通常
//! 一次选完，所以只提供整体替换（与 GoalStage 同构）。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::modules::learning_session::dto::{
    SessionContentItem, SessionContentListData, SessionContentRow, SessionProjectsData,
    SessionTaskItem, SessionTaskListData, SessionTaskRow, SetSessionContentsRequest,
    SetSessionProjectsRequest, SetSessionTasksRequest,
};
use crate::modules::learning_session::rules;
use crate::modules::learning_session::service::{
    ensure_contents_owned, ensure_existing_tasks_attributable, ensure_projects_owned,
    ensure_tasks_attributable, insert_contents, insert_related_projects, insert_tasks,
    lock_session,
};
use crate::shared::AppResult;

/// 整体替换关联内容：空数组表示清空（幂等）
pub async fn set_contents(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
    payload: SetSessionContentsRequest,
) -> AppResult<SessionContentListData> {
    rules::ensure_unique_contents(&payload.items)?;
    for item in &payload.items {
        rules::parse_contribution(&item.contribution_type)?;
    }

    let mut tx = pool.begin().await?;

    lock_session(&mut tx, owner_id, session_id).await?;
    ensure_contents_owned(&mut tx, owner_id, &payload.items).await?;

    sqlx::query("DELETE FROM session_contents WHERE session_id = $1")
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

    insert_contents(&mut tx, session_id, &payload.items).await?;

    let items = read_contents(&mut tx, session_id).await?;
    tx.commit().await?;

    Ok(SessionContentListData {
        total: items.len() as i64,
        items,
    })
}

/// 整体替换 related 项目：主要项目由 `primary_project_id` 唯一表达，出现在这里即 422
pub async fn set_projects(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
    payload: SetSessionProjectsRequest,
) -> AppResult<SessionProjectsData> {
    let mut tx = pool.begin().await?;

    let current = lock_session(&mut tx, owner_id, session_id).await?;
    let project_ids = rules::dedupe_ids(&payload.project_ids);
    rules::ensure_related_excludes_primary(current.primary_project_id, &project_ids)?;
    ensure_projects_owned(&mut tx, owner_id, &project_ids).await?;

    // 换成新项目集合后，既有分摊可能失去归属——先挡住，不静默删除分摊行
    let mut allowed = project_ids.clone();
    if let Some(primary) = current.primary_project_id {
        allowed.push(primary);
    }
    ensure_existing_tasks_attributable(&mut tx, session_id, &allowed).await?;

    sqlx::query("DELETE FROM session_projects WHERE session_id = $1")
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

    insert_related_projects(&mut tx, session_id, &project_ids).await?;
    tx.commit().await?;

    Ok(SessionProjectsData { project_ids })
}

/// 整体替换任务分摊：合计 ≤ 有效时长，任务须属本次活动的项目集合
pub async fn set_tasks(
    pool: &PgPool,
    owner_id: Uuid,
    session_id: Uuid,
    payload: SetSessionTasksRequest,
) -> AppResult<SessionTaskListData> {
    rules::ensure_unique_tasks(&payload.items)?;

    let mut tx = pool.begin().await?;

    let current = lock_session(&mut tx, owner_id, session_id).await?;
    rules::ensure_allocation_within_duration(&payload.items, current.duration_minutes)?;

    // 允许的项目集合 = 主要项目 + 当前关联项目（关联替换与分摊替换互不改写对方）
    let mut project_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT project_id FROM session_projects WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_all(&mut *tx)
    .await?;

    if let Some(primary) = current.primary_project_id {
        project_ids.push(primary);
    }

    ensure_tasks_attributable(&mut tx, owner_id, &project_ids, &payload.items).await?;

    sqlx::query("DELETE FROM task_sessions WHERE session_id = $1")
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

    insert_tasks(&mut tx, session_id, &payload.items).await?;

    let items = read_tasks(&mut tx, session_id).await?;
    tx.commit().await?;

    Ok(SessionTaskListData {
        total: items.len() as i64,
        items,
    })
}

async fn read_contents(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
) -> AppResult<Vec<SessionContentItem>> {
    sqlx::query_as::<_, SessionContentRow>(
        r#"
        SELECT content_id, contribution_type, note
        FROM session_contents
        WHERE session_id = $1
        ORDER BY content_id
        "#,
    )
    .bind(session_id)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(SessionContentItem::try_from)
    .collect()
}

async fn read_tasks(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
) -> AppResult<Vec<SessionTaskItem>> {
    Ok(sqlx::query_as::<_, SessionTaskRow>(
        "SELECT task_id, contribution_minutes, note FROM task_sessions WHERE session_id = $1 ORDER BY task_id",
    )
    .bind(session_id)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(SessionTaskItem::from)
    .collect())
}
