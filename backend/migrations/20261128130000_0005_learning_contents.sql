-- 学习内容域（M03）：内容本体 / 内容关系 / 标签词表 / 项目内容关联
-- 依据：ERD V1.2 §5.2、§8.2；ADR-009；契约设计 D-23 ~ D-29
-- 外键 ON DELETE 按 ERD §8.2 三条判据推导：
--   归属根（NOT NULL）→ CASCADE ／ 可选归属 → SET NULL ／ 纯关联行 → CASCADE

-- 相似度提示（D-24）依赖 similarity()，扩展缺失时创建语句直接报错，不静默跳过
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE learning_contents
(
    id             UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    owner_id       UUID         NOT NULL,
    content_type   VARCHAR(30)  NOT NULL,
    title          VARCHAR(200) NOT NULL,
    description    TEXT,
    parent_id      UUID,
    body           TEXT,
    body_format    VARCHAR(20)  NOT NULL DEFAULT 'markdown',
    source_url     VARCHAR(2048),
    metadata       JSONB,
    created_source VARCHAR(20)  NOT NULL DEFAULT 'user',
    status         VARCHAR(20)  NOT NULL DEFAULT 'active',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_learning_contents_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    -- 子内容是独立资产：父内容消失时提升为根，而非连带删除
    CONSTRAINT fk_learning_contents_parent
        FOREIGN KEY (parent_id) REFERENCES learning_contents (id) ON DELETE SET NULL,
    CONSTRAINT ck_learning_contents_type
        CHECK (content_type IN ('knowledge_point', 'skill_point', 'language_item')),
    CONSTRAINT ck_learning_contents_title
        CHECK (length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT ck_learning_contents_description
        CHECK (description IS NULL OR length(description) <= 500),
    CONSTRAINT ck_learning_contents_body
        CHECK (body IS NULL OR length(body) <= 100000),
    CONSTRAINT ck_learning_contents_body_format
        CHECK (body_format IN ('markdown', 'plain_text')),
    CONSTRAINT ck_learning_contents_created_source
        CHECK (created_source IN ('user', 'ai_confirmed', 'template')),
    CONSTRAINT ck_learning_contents_status
        CHECK (status IN ('active', 'archived')),
    -- 自引用在服务层拦（422），DB 兜底
    CONSTRAINT ck_learning_contents_parent_self
        CHECK (parent_id IS NULL OR parent_id <> id)
);

-- 不建 (owner_id, title) 唯一约束：ERD §3.1 明确允许重复创建，只做复用提示
CREATE INDEX idx_learning_contents_owner_status_updated
    ON learning_contents (owner_id, status, updated_at DESC, id DESC);

CREATE INDEX idx_learning_contents_owner_type
    ON learning_contents (owner_id, content_type);

CREATE INDEX idx_learning_contents_parent
    ON learning_contents (parent_id)
    WHERE parent_id IS NOT NULL;

CREATE TABLE content_relations
(
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id          UUID        NOT NULL,
    source_content_id UUID        NOT NULL,
    target_content_id UUID        NOT NULL,
    relation_type     VARCHAR(30) NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_content_relations_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    -- 关系行不承载事实，内容 purge 时由外键清理两端
    CONSTRAINT fk_content_relations_source
        FOREIGN KEY (source_content_id) REFERENCES learning_contents (id) ON DELETE CASCADE,
    CONSTRAINT fk_content_relations_target
        FOREIGN KEY (target_content_id) REFERENCES learning_contents (id) ON DELETE CASCADE,
    CONSTRAINT ck_content_relations_type
        CHECK (relation_type IN ('prerequisite', 'related', 'contains', 'extends', 'alternative')),
    CONSTRAINT ck_content_relations_not_self
        CHECK (source_content_id <> target_content_id),
    -- 单向存储（D-25）：对称关系也只存一行
    CONSTRAINT uq_content_relations_edge
        UNIQUE (source_content_id, target_content_id, relation_type)
);

-- 双向查询的入边方向；出边已被唯一约束的前缀覆盖
CREATE INDEX idx_content_relations_target
    ON content_relations (target_content_id);

CREATE TABLE content_tags
(
    id         UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    owner_id   UUID         NOT NULL,
    name       VARCHAR(60)  NOT NULL,
    color      VARCHAR(20),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_content_tags_owner
        FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT ck_content_tags_name
        CHECK (length(btrim(name)) BETWEEN 1 AND 60),
    -- 用户私有词表：同名必须合并而非并存（重名请求由服务层转 409）
    CONSTRAINT uq_content_tags_owner_name
        UNIQUE (owner_id, name)
);

CREATE TABLE learning_content_tags
(
    content_id UUID        NOT NULL,
    tag_id     UUID        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT pk_learning_content_tags PRIMARY KEY (content_id, tag_id),
    CONSTRAINT fk_learning_content_tags_content
        FOREIGN KEY (content_id) REFERENCES learning_contents (id) ON DELETE CASCADE,
    CONSTRAINT fk_learning_content_tags_tag
        FOREIGN KEY (tag_id) REFERENCES content_tags (id) ON DELETE CASCADE
);

-- 按标签反查内容与删标签时的级联查找
CREATE INDEX idx_learning_content_tags_tag
    ON learning_content_tags (tag_id);

CREATE TABLE project_contents
(
    id                   UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    project_id           UUID        NOT NULL,
    content_id           UUID        NOT NULL,
    target_mastery_level SMALLINT    NOT NULL DEFAULT 0,
    current_status       VARCHAR(20) NOT NULL DEFAULT 'not_started',
    priority             SMALLINT    NOT NULL DEFAULT 3,
    auto_review_enabled  BOOLEAN     NOT NULL DEFAULT FALSE,
    linked_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    achieved_at          TIMESTAMPTZ,
    -- 契约 §4.4 PATCH「同态提交不刷新 updated_at」要求存在该列；不进响应 DTO
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_project_contents_project
        FOREIGN KEY (project_id) REFERENCES learning_projects (id) ON DELETE CASCADE,
    -- D-26 已保证 purge 时引用为 0，此级联只是兜底
    CONSTRAINT fk_project_contents_content
        FOREIGN KEY (content_id) REFERENCES learning_contents (id) ON DELETE CASCADE,
    CONSTRAINT ck_project_contents_mastery
        CHECK (target_mastery_level BETWEEN 0 AND 4),
    CONSTRAINT ck_project_contents_status
        CHECK (current_status IN ('not_started', 'learning', 'reviewing', 'achieved', 'waived')),
    CONSTRAINT ck_project_contents_priority
        CHECK (priority BETWEEN 1 AND 5),
    -- 反向不成立：离开 achieved 保留 achieved_at（「曾经达成」是事实，D-27）
    CONSTRAINT ck_project_contents_achieved_at
        CHECK (current_status <> 'achieved' OR achieved_at IS NOT NULL),
    CONSTRAINT uq_project_contents_project_content
        UNIQUE (project_id, content_id)
);

CREATE INDEX idx_project_contents_project_status
    ON project_contents (project_id, current_status, updated_at DESC, id DESC);

-- 跨项目复用查询：「这份内容在哪些项目里、到哪一步了」
CREATE INDEX idx_project_contents_content
    ON project_contents (content_id);
