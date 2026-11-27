-- 学习项目（M02）与规划域（M04）：目标 / 路线 / 阶段 / 里程碑 / 任务 / 依赖 / 目标-阶段关联
-- 依据：ERD V1.2 §5.1、§8.2；ADR-005 / ADR-006 / ADR-007
-- 外键 ON DELETE 不是实现细节，逐条按 ERD §8.2 的判据落地：
--   归属根（NOT NULL）→ CASCADE ／ 可选归属 → SET NULL ／ 纯关联行 → CASCADE

CREATE TABLE learning_projects
(
    id                        UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    owner_id                  UUID        NOT NULL,
    name                      VARCHAR(160) NOT NULL,
    description               TEXT,
    theory_enabled            BOOLEAN     NOT NULL DEFAULT TRUE,
    practice_enabled          BOOLEAN     NOT NULL DEFAULT TRUE,
    project_type              VARCHAR(30) NOT NULL DEFAULT 'study',
    status                    VARCHAR(20) NOT NULL DEFAULT 'draft',
    importance_level          SMALLINT    NOT NULL DEFAULT 3,
    importance_override_until TIMESTAMPTZ,
    deadline_type             VARCHAR(20) NOT NULL DEFAULT 'none',
    deadline_at               TIMESTAMPTZ,
    archived_at               TIMESTAMPTZ,
    reopened_at               TIMESTAMPTZ,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_learning_projects_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT ck_learning_projects_name
        CHECK (length(btrim(name)) BETWEEN 1 AND 160),
    CONSTRAINT ck_learning_projects_type
        CHECK (project_type IN ('study', 'skill_development', 'challenge', 'deliverable')),
    -- deleting 是 purge 事务内部的并发写保护标记，值合法但只允许内部写入（ADR-006 §2）
    CONSTRAINT ck_learning_projects_status
        CHECK (status IN ('draft', 'active', 'paused', 'completed', 'archived', 'deleting')),
    CONSTRAINT ck_learning_projects_importance
        CHECK (importance_level BETWEEN 1 AND 5),
    CONSTRAINT ck_learning_projects_deadline_type
        CHECK (deadline_type IN ('none', 'soft', 'fixed')),
    -- ERD §5.1：none 时 deadline_at 必须为空；soft / fixed 时必填
    CONSTRAINT ck_learning_projects_deadline_pair
        CHECK ((deadline_type = 'none' AND deadline_at IS NULL)
            OR (deadline_type <> 'none' AND deadline_at IS NOT NULL))
);

CREATE INDEX idx_learning_projects_owner_status_updated
    ON learning_projects (owner_id, status, updated_at DESC, id DESC);

CREATE TABLE project_goals
(
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id         UUID         NOT NULL,
    parent_goal_id     UUID,
    title              VARCHAR(200) NOT NULL,
    description        TEXT,
    goal_type          VARCHAR(30)  NOT NULL DEFAULT 'outcome',
    target_definition  JSONB,
    status             VARCHAR(20)  NOT NULL DEFAULT 'active',
    target_at          TIMESTAMPTZ,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_project_goals_project
        FOREIGN KEY (project_id) REFERENCES learning_projects (id) ON DELETE CASCADE,
    -- 不能用 SET NULL：那会把子目标升格为第二个主目标，破坏「每项目最多一个主目标」（D-06）
    CONSTRAINT fk_project_goals_parent
        FOREIGN KEY (parent_goal_id) REFERENCES project_goals (id) ON DELETE CASCADE,
    CONSTRAINT ck_project_goals_not_self_parent
        CHECK (parent_goal_id IS NULL OR parent_goal_id <> id),
    CONSTRAINT ck_project_goals_title
        CHECK (length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT ck_project_goals_type
        CHECK (goal_type IN ('outcome', 'performance', 'completion')),
    CONSTRAINT ck_project_goals_status
        CHECK (status IN ('active', 'achieved', 'cancelled'))
);

CREATE INDEX idx_project_goals_project_created
    ON project_goals (project_id, created_at ASC, id ASC);
CREATE INDEX idx_project_goals_parent
    ON project_goals (parent_goal_id) WHERE parent_goal_id IS NOT NULL;

CREATE TABLE roadmaps
(
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id     UUID        NOT NULL,
    version        INTEGER     NOT NULL,
    status         VARCHAR(20) NOT NULL DEFAULT 'draft',
    source         VARCHAR(20) NOT NULL DEFAULT 'user',
    change_summary TEXT,
    confirmed_at   TIMESTAMPTZ,
    created_by     UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_roadmaps_project
        FOREIGN KEY (project_id) REFERENCES learning_projects (id) ON DELETE CASCADE,
    -- created_by 可空：系统 / AI 生成的路线没有人类操作者（ERD §5.1）
    CONSTRAINT fk_roadmaps_created_by
        FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE SET NULL,
    CONSTRAINT uq_roadmaps_project_version UNIQUE (project_id, version),
    CONSTRAINT ck_roadmaps_version CHECK (version >= 1),
    CONSTRAINT ck_roadmaps_status CHECK (status IN ('draft', 'active', 'superseded')),
    CONSTRAINT ck_roadmaps_source CHECK (source IN ('user', 'ai', 'template'))
);

CREATE INDEX idx_roadmaps_project_version ON roadmaps (project_id, version DESC);
-- 部分唯一索引兜底「一个项目最多一个 active 路线」（ADR-004）
CREATE UNIQUE INDEX uq_roadmaps_project_active
    ON roadmaps (project_id) WHERE status = 'active';

CREATE TABLE stages
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    roadmap_id        UUID         NOT NULL,
    name              VARCHAR(160) NOT NULL,
    description       TEXT,
    sequence_no       INTEGER      NOT NULL,
    status            VARCHAR(20)  NOT NULL DEFAULT 'pending',
    completion_rule   JSONB,
    completion_method VARCHAR(20)  NOT NULL DEFAULT 'user',
    started_at        TIMESTAMPTZ,
    completed_at      TIMESTAMPTZ,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_stages_roadmap
        FOREIGN KEY (roadmap_id) REFERENCES roadmaps (id) ON DELETE CASCADE,
    CONSTRAINT uq_stages_roadmap_sequence UNIQUE (roadmap_id, sequence_no),
    CONSTRAINT ck_stages_name
        CHECK (length(btrim(name)) BETWEEN 1 AND 160),
    CONSTRAINT ck_stages_status
        CHECK (status IN ('pending', 'active', 'completed', 'skipped')),
    CONSTRAINT ck_stages_completion_method
        CHECK (completion_method IN ('user', 'system_confirmed', 'ai_suggested'))
);

CREATE INDEX idx_stages_roadmap_sequence ON stages (roadmap_id, sequence_no ASC, id ASC);

CREATE TABLE milestones
(
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_id        UUID         NOT NULL,
    title           VARCHAR(200) NOT NULL,
    description     TEXT,
    status          VARCHAR(20)  NOT NULL DEFAULT 'pending',
    completion_rule JSONB,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_milestones_stage
        FOREIGN KEY (stage_id) REFERENCES stages (id) ON DELETE CASCADE,
    CONSTRAINT ck_milestones_title
        CHECK (length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT ck_milestones_status
        CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped'))
);

CREATE INDEX idx_milestones_stage_created ON milestones (stage_id, created_at ASC, id ASC);

CREATE TABLE learning_tasks
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    project_id        UUID         NOT NULL,
    milestone_id      UUID,
    stage_id          UUID,
    title             VARCHAR(200) NOT NULL,
    description       TEXT,
    status            VARCHAR(20)  NOT NULL DEFAULT 'todo',
    priority          SMALLINT     NOT NULL DEFAULT 3,
    due_at            TIMESTAMPTZ,
    estimated_minutes INTEGER,
    completed_at      TIMESTAMPTZ,
    created_source    VARCHAR(20)  NOT NULL DEFAULT 'user',
    deleted_at        TIMESTAMPTZ,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),

    -- 唯一绕过软删的路径：项目是任务的归属根（NOT NULL），项目物理删则任务连同软删行一并消失
    CONSTRAINT fk_learning_tasks_project
        FOREIGN KEY (project_id) REFERENCES learning_projects (id) ON DELETE CASCADE,
    -- 任务不随上层结构消失（L3），只能解挂；软删行也会被 SET NULL（外键不看 deleted_at）
    CONSTRAINT fk_learning_tasks_milestone
        FOREIGN KEY (milestone_id) REFERENCES milestones (id) ON DELETE SET NULL,
    CONSTRAINT fk_learning_tasks_stage
        FOREIGN KEY (stage_id) REFERENCES stages (id) ON DELETE SET NULL,
    CONSTRAINT ck_learning_tasks_title
        CHECK (length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT ck_learning_tasks_status
        CHECK (status IN ('todo', 'in_progress', 'blocked', 'completed', 'cancelled')),
    CONSTRAINT ck_learning_tasks_priority
        CHECK (priority BETWEEN 1 AND 5),
    CONSTRAINT ck_learning_tasks_estimated_minutes
        CHECK (estimated_minutes IS NULL OR estimated_minutes >= 0),
    CONSTRAINT ck_learning_tasks_source
        CHECK (created_source IN ('user', 'ai', 'template'))
);

-- 软删行不参与任何查询，索引一律带部分条件（ERD §7 V1.1 补充）
CREATE INDEX idx_learning_tasks_project_status_priority
    ON learning_tasks (project_id, status, priority DESC, created_at DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_learning_tasks_project_due
    ON learning_tasks (project_id, due_at)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_learning_tasks_project_stage
    ON learning_tasks (project_id, stage_id)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_learning_tasks_project_milestone
    ON learning_tasks (project_id, milestone_id)
    WHERE deleted_at IS NULL;

CREATE TABLE task_dependencies
(
    task_id            UUID        NOT NULL,
    depends_on_task_id UUID        NOT NULL,
    dependency_type    VARCHAR(20) NOT NULL DEFAULT 'finish_to_start',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT pk_task_dependencies PRIMARY KEY (task_id, depends_on_task_id),
    CONSTRAINT fk_task_dependencies_task
        FOREIGN KEY (task_id) REFERENCES learning_tasks (id) ON DELETE CASCADE,
    CONSTRAINT fk_task_dependencies_depends_on
        FOREIGN KEY (depends_on_task_id) REFERENCES learning_tasks (id) ON DELETE CASCADE,
    CONSTRAINT ck_task_dependencies_not_self CHECK (task_id <> depends_on_task_id),
    CONSTRAINT ck_task_dependencies_type CHECK (dependency_type IN ('finish_to_start'))
);

CREATE INDEX idx_task_dependencies_depends_on ON task_dependencies (depends_on_task_id);

CREATE TABLE goal_stages
(
    goal_id      UUID        NOT NULL,
    stage_id     UUID        NOT NULL,
    contribution VARCHAR(20) NOT NULL DEFAULT 'supporting',

    CONSTRAINT pk_goal_stages PRIMARY KEY (goal_id, stage_id),
    CONSTRAINT fk_goal_stages_goal
        FOREIGN KEY (goal_id) REFERENCES project_goals (id) ON DELETE CASCADE,
    CONSTRAINT fk_goal_stages_stage
        FOREIGN KEY (stage_id) REFERENCES stages (id) ON DELETE CASCADE,
    CONSTRAINT ck_goal_stages_contribution CHECK (contribution IN ('primary', 'supporting'))
);

CREATE INDEX idx_goal_stages_stage ON goal_stages (stage_id);
