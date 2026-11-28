//! 项目目标（ProjectGoal）与目标-阶段关联（GoalStage）的持久化。

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::modules::roadmap::dto::{
    CreateGoalRequest, GoalItem, GoalListData, GoalRow, GoalStageItem, GoalStageLinkOut,
    GoalStageListData, ReplaceGoalStageLinksRequest, UpdateGoalRequest,
};
use crate::modules::roadmap::rules;
use crate::shared::{AppError, AppResult, Contribution, GoalStatus, GoalType};

const GOAL_COLUMNS: &str = r#"
    id, project_id, parent_goal_id, title, description, goal_type, status,
    target_at, created_at, updated_at
"#;

/// 确认项目属于当前用户；返回其是否存在（越权与不存在都得到 404）
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

/// 目标行 + 项目归属校验（一次查询同时完成存在性、归属、父链一致性）
async fn fetch_row(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
) -> AppResult<GoalRow> {
    let sql = format!(
        r#"
        SELECT {GOAL_COLUMNS}
        FROM project_goals g
        WHERE g.id = $1
          AND g.project_id = $2
          AND EXISTS (
              SELECT 1 FROM learning_projects p
              WHERE p.id = g.project_id AND p.owner_id = $3
          )
        "#
    );

    sqlx::query_as::<_, GoalRow>(sqlx::AssertSqlSafe(sql))
        .bind(goal_id)
        .bind(project_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("目标不存在".to_string()))
}

/// 目标列表（按 created_at ASC 稳定排序，D-14）
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<GoalListData> {
    ensure_project(pool, owner_id, project_id).await?;

    let sql = format!(
        r#"
        SELECT {GOAL_COLUMNS}
        FROM project_goals
        WHERE project_id = $1
        ORDER BY created_at ASC, id ASC
        LIMIT $2 OFFSET $3
        "#
    );

    let rows = sqlx::query_as::<_, GoalRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM project_goals WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(pool)
            .await?;

    let items = rows
        .into_iter()
        .map(GoalItem::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(GoalListData { items, total })
}

/// 创建目标。
///
/// 层级约束（D-06）：`parent_goal_id` 为空即主目标，非空必须是同项目的主目标
/// （即父行自身的 `parent_goal_id` 为 NULL），由此天然保证「上限两层」。
///
/// 「每项目最多一个主目标」的**冲突处置**仍是待决策项（M04 §7）：
/// 这里取保守语义——已有主目标时再建主目标返回 409，不静默降级既有数据。
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    payload: CreateGoalRequest,
) -> AppResult<GoalItem> {
    ensure_project(pool, owner_id, project_id).await?;

    let title = rules::ensure_title("title", &payload.title, 200)?;
    let goal_type = payload.goal_type.unwrap_or(GoalType::Outcome);

    match payload.parent_goal_id {
        Some(parent_id) => ensure_parent(pool, project_id, parent_id).await?,
        None => ensure_single_primary(pool, project_id, None).await?,
    }

    let sql = format!(
        r#"
        INSERT INTO project_goals (
            project_id, parent_goal_id, title, description, goal_type, status, target_at
        ) VALUES ($1, $2, $3, $4, $5, 'active', $6)
        RETURNING {GOAL_COLUMNS}
        "#
    );

    let row = sqlx::query_as::<_, GoalRow>(sqlx::AssertSqlSafe(sql))
        .bind(project_id)
        .bind(payload.parent_goal_id)
        .bind(title)
        .bind(payload.description)
        .bind(goal_type.as_str())
        .bind(payload.target_at)
        .fetch_one(pool)
        .await?;

    GoalItem::try_from(row)
}

/// 父目标必须是同项目、且自身为主目标（子目标不得再有子目标）
async fn ensure_parent(pool: &PgPool, project_id: Uuid, parent_id: Uuid) -> AppResult<()> {
    let parent = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
        "SELECT project_id, parent_goal_id FROM project_goals WHERE id = $1",
    )
    .bind(parent_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("parent_goal_id 指向的目标不存在".to_string()))?;

    if parent.0 != project_id {
        return Err(AppError::Validation(
            "parent_goal_id 必须指向同一项目内的目标".to_string(),
        ));
    }

    if parent.1.is_some() {
        return Err(AppError::Validation(
            "目标层级上限为两层：子目标不能再作为父目标".to_string(),
        ));
    }

    Ok(())
}

/// 降为子目标前的守卫：自身有子目标时不允许降级，否则会出现第三层（D-06）
async fn ensure_no_children(pool: &PgPool, goal_id: Uuid) -> AppResult<()> {
    let children = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_goals WHERE parent_goal_id = $1",
    )
    .bind(goal_id)
    .fetch_one(pool)
    .await?;

    if children > 0 {
        return Err(AppError::Validation(
            "该目标下已有子目标，不能再降为子目标（层级上限两层）".to_string(),
        ));
    }

    Ok(())
}

/// 「每项目最多一个主目标」校验；`exclude` 用于更新场景排除自身
async fn ensure_single_primary(
    pool: &PgPool,
    project_id: Uuid,
    exclude: Option<Uuid>,
) -> AppResult<()> {
    let existing = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM project_goals
        WHERE project_id = $1
          AND parent_goal_id IS NULL
          AND ($2::uuid IS NULL OR id <> $2)
        "#,
    )
    .bind(project_id)
    .bind(exclude)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict(
            "该项目已有主目标；如需添加目标请指定 parent_goal_id，或先把原主目标改为子目标"
                .to_string(),
        ));
    }

    Ok(())
}

/// 目标详情
pub async fn detail(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
) -> AppResult<GoalItem> {
    let row = fetch_row(pool, owner_id, project_id, goal_id).await?;
    GoalItem::try_from(row)
}

/// 目标状态迁移（D-07b）：`active ↔ achieved / cancelled`；`achieved → cancelled` 拒绝。
fn evaluate_goal_status(current: GoalStatus, target: GoalStatus) -> AppResult<Option<GoalStatus>> {
    if current == target {
        return Ok(None);
    }

    match (current, target) {
        (GoalStatus::Active, GoalStatus::Achieved)
        | (GoalStatus::Active, GoalStatus::Cancelled)
        | (GoalStatus::Achieved, GoalStatus::Active)
        | (GoalStatus::Cancelled, GoalStatus::Active) => Ok(Some(target)),
        _ => Err(AppError::Conflict(format!(
            "目标不允许从 {} 直接转到 {}，请先回到 active",
            current.as_str(),
            target.as_str()
        ))),
    }
}

/// 部分更新目标
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
    payload: UpdateGoalRequest,
) -> AppResult<GoalItem> {
    let current = fetch_row(pool, owner_id, project_id, goal_id).await?;
    let current_status = GoalStatus::parse(&current.status)?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE project_goals SET ");
    let mut separated = builder.separated(", ");
    let mut touched = false;

    if let Some(title) = payload.title {
        let title = rules::ensure_title("title", &title, 200)?;
        separated.push("title = ").push_bind_unseparated(title);
        touched = true;
    }

    if let Some(description) = payload.description {
        separated
            .push("description = ")
            .push_bind_unseparated(description);
        touched = true;
    }

    if let Some(parent) = payload.parent_goal_id {
        match parent {
            Some(parent_id) => {
                if parent_id == goal_id {
                    return Err(AppError::Validation("目标不能以自身为父目标".to_string()));
                }
                ensure_parent(pool, project_id, parent_id).await?;
                ensure_no_children(pool, goal_id).await?;
            }
            None => {
                // 提升为主目标：须保证项目内没有第二个主目标
                ensure_single_primary(pool, project_id, Some(goal_id)).await?;
            }
        }
        separated
            .push("parent_goal_id = ")
            .push_bind_unseparated(parent);
        touched = true;
    }

    if let Some(goal_type) = payload.goal_type {
        separated
            .push("goal_type = ")
            .push_bind_unseparated(goal_type.as_str().to_string());
        touched = true;
    }

    if let Some(target_at) = payload.target_at {
        separated
            .push("target_at = ")
            .push_bind_unseparated(target_at);
        touched = true;
    }

    if let Some(target_status) = payload.status
        && let Some(next) = evaluate_goal_status(current_status, target_status)?
    {
        separated
            .push("status = ")
            .push_bind_unseparated(next.as_str().to_string());
        touched = true;
    }

    // 全部字段都是同态或未提供：按幂等返回当前行，不刷新 updated_at
    if !touched {
        return GoalItem::try_from(current);
    }

    separated.push_unseparated(", updated_at = now()");

    builder
        .push(" WHERE id = ")
        .push_bind(goal_id)
        .push(" AND project_id = ")
        .push_bind(project_id)
        .push(format!(" RETURNING {GOAL_COLUMNS}"));

    let row = builder
        .build_query_as::<GoalRow>()
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("目标不存在".to_string()))?;

    GoalItem::try_from(row)
}

/// 删除目标（D-07a）。
///
/// `achieved` 是成果事实，拒绝删除。删主目标时其下子目标**一并删除**——
/// 不能把子目标 `parent_goal_id` 置 NULL，那会造出第二个主目标（D-06）。
pub async fn delete(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
    cascade: bool,
) -> AppResult<()> {
    let current = fetch_row(pool, owner_id, project_id, goal_id).await?;
    let status = GoalStatus::parse(&current.status)?;

    if status == GoalStatus::Achieved {
        return Err(AppError::Conflict(
            "已达成（achieved）的目标是成果事实，不可删除；如需清理展示请改状态".to_string(),
        ));
    }

    let child_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_goals WHERE parent_goal_id = $1",
    )
    .bind(goal_id)
    .fetch_one(pool)
    .await?;

    if child_count > 0 && !cascade {
        return Err(AppError::DeletionConflict {
            impact: serde_json::json!({ "goals": child_count }),
            hint: "目标下存在子目标，确认一并删除请追加 ?cascade=true".to_string(),
        });
    }

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM goal_stages WHERE goal_id = $1")
        .bind(goal_id)
        .execute(&mut *tx)
        .await?;

    // 子目标随父目标删除（外键亦为 CASCADE，此处显式执行以便计数与顺序可控）
    sqlx::query("DELETE FROM project_goals WHERE parent_goal_id = $1")
        .bind(goal_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM project_goals WHERE id = $1")
        .bind(goal_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 目标关联的阶段列表
pub async fn list_goal_stages(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
) -> AppResult<GoalStageListData> {
    fetch_row(pool, owner_id, project_id, goal_id).await?;

    let rows = sqlx::query_as::<_, GoalStageItem>(
        "SELECT goal_id, stage_id, contribution FROM goal_stages WHERE goal_id = $1 ORDER BY stage_id",
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(GoalStageLinkOut::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(GoalStageListData { items })
}

/// 整体替换目标-阶段关联（D-13）。
///
/// 天然幂等：同一集合重复提交得到同一结果，传空数组即清空。
pub async fn replace_goal_stages(
    pool: &PgPool,
    owner_id: Uuid,
    project_id: Uuid,
    goal_id: Uuid,
    payload: ReplaceGoalStageLinksRequest,
) -> AppResult<GoalStageListData> {
    fetch_row(pool, owner_id, project_id, goal_id).await?;

    let mut stage_ids: Vec<Uuid> = payload.links.iter().map(|link| link.stage_id).collect();
    let requested = stage_ids.len();

    stage_ids.sort_unstable();
    stage_ids.dedup();
    if stage_ids.len() != requested {
        return Err(AppError::Validation(
            "links 中存在重复的 stage_id".to_string(),
        ));
    }

    if requested > 0 {
        let owned = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM stages s
            JOIN roadmaps r ON r.id = s.roadmap_id
            WHERE s.id = ANY($1) AND r.project_id = $2
            "#,
        )
        .bind(&stage_ids)
        .bind(project_id)
        .fetch_one(pool)
        .await?;

        if owned != requested as i64 {
            return Err(AppError::Validation(
                "links 中存在不属于该项目的阶段".to_string(),
            ));
        }
    }

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM goal_stages WHERE goal_id = $1")
        .bind(goal_id)
        .execute(&mut *tx)
        .await?;

    for link in &payload.links {
        let contribution = link.contribution.unwrap_or(Contribution::Supporting);
        sqlx::query(
            "INSERT INTO goal_stages (goal_id, stage_id, contribution) VALUES ($1, $2, $3)",
        )
        .bind(goal_id)
        .bind(link.stage_id)
        .bind(contribution.as_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    list_goal_stages(pool, owner_id, project_id, goal_id).await
}
