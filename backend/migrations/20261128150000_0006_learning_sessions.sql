-- M05 学习活动与证据：正式 LearningSession + 三类关联 + 学习证据（ADR-010）
--
-- 本迁移只建表：过渡表 learning_records 的数据迁移在 0007，旧表下线在 0008
-- （ADR-010 §6：新建表 + 一次性迁移 + 不双写）。

CREATE TABLE learning_sessions
(
    id                 UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    owner_id           UUID         NOT NULL,
    primary_project_id UUID,
    title              VARCHAR(200),
    activity_type      VARCHAR(30)  NOT NULL DEFAULT 'mixed',
    started_at         TIMESTAMPTZ  NOT NULL,
    ended_at           TIMESTAMPTZ  NOT NULL,
    -- 有效时长 = min(用户上报, 区间分钟数)，由服务层生成（ADR-010 §1）
    duration_minutes   INTEGER      NOT NULL DEFAULT 0,
    summary            TEXT,
    what_done          TEXT,
    problems           TEXT,
    next_steps         TEXT,
    created_source     VARCHAR(20)  NOT NULL DEFAULT 'user',
    -- 录入时间与实际发生时间分离（PRD §16.2）：补录时 created_at 可远晚于 started_at
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    deleted_at         TIMESTAMPTZ,

    CONSTRAINT fk_learning_sessions_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    -- 项目是容器、学习记录是事实：删容器不销毁事实（ADR-010 §5 / D-20）
    CONSTRAINT fk_learning_sessions_primary_project
        FOREIGN KEY (primary_project_id) REFERENCES learning_projects (id) ON DELETE SET NULL,
    CONSTRAINT ck_learning_sessions_title
        CHECK (title IS NULL OR length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT ck_learning_sessions_activity_type
        CHECK (activity_type IN ('theory', 'practice', 'review', 'mixed', 'other')),
    CONSTRAINT ck_learning_sessions_interval
        CHECK (ended_at >= started_at),
    CONSTRAINT ck_learning_sessions_duration
        CHECK (duration_minutes >= 0),
    CONSTRAINT ck_learning_sessions_source
        CHECK (created_source IN ('user', 'timer', 'imported'))
);

-- 所有列表与聚合都带 deleted_at IS NULL（ADR-010 决策 3），索引一律带部分条件
CREATE INDEX idx_learning_sessions_owner_started
    ON learning_sessions (owner_id, started_at DESC, id DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_learning_sessions_primary_project
    ON learning_sessions (primary_project_id)
    WHERE primary_project_id IS NOT NULL;

CREATE TABLE session_projects
(
    session_id    UUID        NOT NULL,
    project_id    UUID        NOT NULL,
    relation_role VARCHAR(20) NOT NULL DEFAULT 'related',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (session_id, project_id),
    CONSTRAINT fk_session_projects_session
        FOREIGN KEY (session_id) REFERENCES learning_sessions (id) ON DELETE CASCADE,
    CONSTRAINT fk_session_projects_project
        FOREIGN KEY (project_id) REFERENCES learning_projects (id) ON DELETE CASCADE,
    -- 只存 related：主要项目由 learning_sessions.primary_project_id 唯一表达（ERD §5.3 推荐方案）
    CONSTRAINT ck_session_projects_role
        CHECK (relation_role = 'related')
);

CREATE INDEX idx_session_projects_project ON session_projects (project_id);

CREATE TABLE session_contents
(
    session_id        UUID        NOT NULL,
    content_id        UUID        NOT NULL,
    contribution_type VARCHAR(20) NOT NULL DEFAULT 'covered',
    note              TEXT,

    PRIMARY KEY (session_id, content_id),
    CONSTRAINT fk_session_contents_session
        FOREIGN KEY (session_id) REFERENCES learning_sessions (id) ON DELETE CASCADE,
    CONSTRAINT fk_session_contents_content
        FOREIGN KEY (content_id) REFERENCES learning_contents (id) ON DELETE CASCADE,
    CONSTRAINT ck_session_contents_contribution
        CHECK (contribution_type IN ('primary', 'covered', 'practiced'))
);

CREATE INDEX idx_session_contents_content ON session_contents (content_id);

CREATE TABLE task_sessions
(
    task_id              UUID    NOT NULL,
    session_id           UUID    NOT NULL,
    contribution_minutes INTEGER NOT NULL DEFAULT 0,
    note                 TEXT,

    PRIMARY KEY (task_id, session_id),
    CONSTRAINT fk_task_sessions_task
        FOREIGN KEY (task_id) REFERENCES learning_tasks (id) ON DELETE CASCADE,
    CONSTRAINT fk_task_sessions_session
        FOREIGN KEY (session_id) REFERENCES learning_sessions (id) ON DELETE CASCADE,
    CONSTRAINT ck_task_sessions_contribution
        CHECK (contribution_minutes >= 0)
);

CREATE INDEX idx_task_sessions_session ON task_sessions (session_id);

CREATE TABLE learning_evidence
(
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id      UUID         NOT NULL,
    session_id    UUID,
    evidence_type VARCHAR(30)  NOT NULL,
    title         VARCHAR(200),
    description   TEXT,
    external_url  TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_learning_evidence_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    -- 证据有独立价值：session 消失（或将来被物理删）后仍保留
    CONSTRAINT fk_learning_evidence_session
        FOREIGN KEY (session_id) REFERENCES learning_sessions (id) ON DELETE SET NULL,
    -- v1 只开三类文本 / 链接证据；storage_key 系列字段无生产者，不建列（ADR-010 §4）
    CONSTRAINT ck_learning_evidence_type
        CHECK (evidence_type IN ('note', 'link', 'test_result')),
    CONSTRAINT ck_learning_evidence_title
        CHECK (title IS NULL OR length(btrim(title)) BETWEEN 1 AND 200),
    -- 服务层先校验并给出可读 message，这里是最终兜底
    CONSTRAINT ck_learning_evidence_payload
        CHECK (
            (evidence_type IN ('note', 'test_result')
                AND description IS NOT NULL AND length(btrim(description)) > 0)
            OR (evidence_type = 'link'
                AND external_url IS NOT NULL AND length(btrim(external_url)) > 0)
        )
);

CREATE INDEX idx_learning_evidence_owner_created
    ON learning_evidence (owner_id, created_at DESC, id DESC);
CREATE INDEX idx_learning_evidence_session ON learning_evidence (session_id);
