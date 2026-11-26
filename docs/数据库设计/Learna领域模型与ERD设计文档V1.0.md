# Learna 领域模型与 ERD 设计文档

**版本：** Domain Model & ERD V1.0\
**状态：** 领域模型评审后设计稿\
**适用范围：** Learna 首版多用户学习管理平台\
**设计依据：** Learna PRD V1.0 与领域模型评审确认结果

------------------------------------------------------------------------

## 1. 文档目的与设计边界

本文将 Learna 的产品需求转化为可实施的逻辑领域模型和关系数据库
ERD。本文明确核心实体、实体关系、数据所有权、业务不变量、数据库约束、索引方向及删除策略。

本文是数据库逻辑设计基线，不等同于最终 SQL migration。实际 DDL
仍需结合技术栈、数据库版本、认证方案、附件存储方案及 API 契约进行校准。

### 1.1 已确认的五项决策

  -----------------------------------------------------------------------------------------------------------------------------------------------------
  编号              领域问题          已确认决策                                      对模型的影响
  ----------------- ----------------- ----------------------------------------------- -----------------------------------------------------------------
  D-01              学习内容唯一性    C：允许重复；发现相似内容时提示复用或继续创建   不对标题或内容摘要施加全局唯一约束；可增加相似内容检测索引/服务

  D-02              个人总体掌握度    B：综合历史评价和证据，由系统独立评估           区分项目上下文评价与个人总体评价

  D-03              学习记录归属      C：一个主要项目 + 多个关联项目                  主项目计入投入；其他项目仅建立关联，不重复计时

  D-04              复习调度范围      C：默认独立，支持共享                           每个复习状态有明确上下文；共享结果通过显式引用或共享记录表达

  D-05              项目删除策略      B：默认归档，彻底删除需要额外操作               归档不级联删除；彻底删除采用受控删除流程
  -----------------------------------------------------------------------------------------------------------------------------------------------------

### 1.2 设计原则

1.  **事实与计划分离**：计划任务、日程安排、实际学习活动是不同业务对象。
2.  **内容资产与项目上下文分离**：学习内容归用户所有，项目通过关联实体使用内容。
3.  **项目状态与个人总体状态分离**：项目目标等级不等于个人总体掌握等级。
4.  **评价保留历史**：掌握度评价和复习记录采用追加历史的方式，不直接覆盖历史事实。
5.  **AI 输出先提案后应用**：AI
    生成的内容、路线变更和调度建议不能绕过用户确认规则。
6.  **默认禁止重复计量**：学习活动仅有一个主要项目归属；关联项目不重复计算时长。
7.  **默认软删除/归档**：业务对象的删除行为必须显式定义，不依赖数据库的随意级联。

------------------------------------------------------------------------

## 2. 领域划分与上下文

Learna 分为六个业务上下文：

  -----------------------------------------------------------------------------------------------------------------------------------------
  上下文                  职责                             核心实体
  ----------------------- -------------------------------- --------------------------------------------------------------------------------
  Identity & Ownership    用户身份与数据所有权             User

  Learning Planning       项目、目标、路线和任务           LearningProject、ProjectGoal、Roadmap、Stage、Milestone、LearningTask

  Learning Content        可跨项目复用的学习内容           LearningContent、ContentRelation、ProjectContent

  Learning Activity       实际学习活动、关联和证据         LearningSession、SessionContent、SessionProject、LearningEvidence、TaskSession

  Mastery & Review        掌握度评价、复习状态与复习历史   MasteryAssessment、OverallMastery、ReviewItem、ReviewRecord、ReviewShare

  Scheduling & AI         每日目标、计划安排与 AI 提案     DailyLearningGoal、ScheduledActivity、ChangeProposal
  Assistance                                               
  -----------------------------------------------------------------------------------------------------------------------------------------

### 2.1 核心概念区分

  ------------------------------------------------------------------------------------
  概念                    业务含义                             不应与其混淆的对象
  ----------------------- ------------------------------------ -----------------------
  ProjectGoal             项目最终希望达到的结果               Stage、Task

  Stage                   项目学习路线中的阶段                 Milestone

  Milestone               阶段内可验证的阶段性成果             Task

  LearningTask            需要完成的工作项                     ScheduledActivity

  LearningContent         用户可复用的知识/技能/语言内容资产   ProjectContent

  ProjectContent          某项目对某项内容的使用上下文         LearningContent

  LearningSession         已经发生的真实学习活动               ScheduledActivity

  LearningEvidence        支持学习、成果或评价的证据材料       MasteryAssessment

  MasteryAssessment       某次掌握度判断                       当前掌握状态

  ReviewItem              一个明确上下文中的复习调度状态       ReviewRecord

  ReviewRecord            一次实际完成的复习                   ReviewItem

  DailyLearningGoal       某日希望达到的学习投入目标           实际学习时长

  ScheduledActivity       某日计划执行的活动                   LearningSession
  ------------------------------------------------------------------------------------

------------------------------------------------------------------------

## 3. 关键领域决策与模型细节

### 3.1 学习内容：允许重复，提供复用提示

`LearningContent` 是用户级资产，但不强制用户级唯一。

-   同一用户可以创建两个标题相同或相似的内容。
-   标题相同不代表语义相同，因此不建立 `(owner_id, title)` 唯一约束。
-   创建内容时，应用服务可以检索同一用户的相似内容并提示复用。
-   用户选择复用时，建立新的 `ProjectContent` 关联，指向已有
    `LearningContent`。
-   用户选择继续创建时，创建新的 `LearningContent`。
-   内容合并不是首版必需能力；若未来支持，必须有可审计的合并映射，不能静默改写历史引用。

建议 `LearningContent` 采用类型字段区分
`knowledge_point`、`skill_point`、`language_item`。不同类型的专属属性放在类型扩展表或受控
JSON 元数据中；参与筛选、约束和算法计算的关键字段应结构化。

### 3.2 项目掌握度与个人总体掌握度

不能把跨项目掌握度简单地取最高值或最近值。

建议分为两层：

**项目上下文评价：** `MasteryAssessment` 通过 `project_content_id`
关联某个项目中的学习要求。项目当前认定状态由该项目的评价历史及确认规则得出。

**个人总体评价：** `OverallMastery` 针对用户的一项
`LearningContent`，汇总跨项目评价、学习证据、评价时间和可信程度，形成系统独立评估。它是派生/评估结果，不覆盖任何项目的评价历史。

个人总体评价至少保存： - `system_level`：系统给出的总体等级。 -
`confidence`：可信程度。 - `assessment_method`：评估方法或模型版本。 -
`evidence_cutoff_at`：本次评估采用证据的时间边界。 -
`evaluated_at`：评估时间。 - `explanation`：可解释摘要。

建议另建 `OverallMasteryAssessment`
历史表，避免每次重算覆盖过去的评估。`OverallMastery`
可作为当前快照，历史表作为评估事件。

### 3.3 学习活动：主要项目 + 关联项目

`LearningSession`
有且仅有一个主要项目（可为空，表示自由学习/尚未归属项目）。其他项目通过
`SessionProject` 关联。

计量规则： - `LearningSession.primary_project_id`
是投入统计的唯一项目归属。 - `SessionProject`
中的关联项目仅用于上下文引用、证据复用和项目进展参考。 - 不允许把同一
session 的分钟数同时计入多个项目。 -
若用户事后调整主要项目归属，必须记录变更审计；统计应按明确的归属口径处理。 -
若一次活动确实包含不同项目的独立时间段，用户应拆分为多条
`LearningSession`，而不是按比例复制同一条时长。

`SessionContent` 支持一次活动关联多个知识点/技能点。建议记录
`contribution_type` 或 `role`，如
`primary`、`covered`、`practiced`，避免把"提及"误判为"已学习掌握"。

### 3.4 复习：默认独立，显式共享

`ReviewItem` 必须有明确的调度上下文。首版默认使用项目上下文： -
`project_content_id` 非空：项目专属复习状态。 - `project_content_id`
为空：必须有明确的 `scope_type` 与 `scope_id`
语义，表示用户级复习，不可仅凭 NULL 推断为全局共享。

建议首版先支持： - `scope_type = project_content`：项目独立调度。 -
`scope_type = personal_content`：个人级调度，仅在用户明确启用时创建。

共享复习结果不意味着共享两个 `ReviewItem` 的间隔、到期日和算法状态。通过
`ReviewShare`
明确记录来源复习记录被哪些上下文引用。接收项目是否据此更新自身调度状态，仍需按项目规则形成自己的
`ReviewRecord` 或调度事件，不能直接把来源项目的状态覆盖过去。

### 3.5 项目归档与彻底删除

项目默认只能归档： - `archived_at`、`status` 记录归档状态。 -
归档项目仍可查询历史任务、学习记录、评价和成果。 -
归档不触发级联删除。 - 用户重新开启项目时，保留历史状态和记录。

彻底删除必须是独立、明确、二次确认的操作。删除前展示影响范围，并按数据保留策略处理： -
删除项目专属规划数据及项目关联。 - `LearningContent`
若仍被其他项目使用，不删除。 - `LearningSession`
若属于该项目的主要学习记录，不能因项目删除而悄悄丢失；建议解除项目归属并保留为个人历史记录，或在用户明确选择后按数据删除策略删除。 -
项目关联证据不应误删其原始文件；附件按引用计数或对象存储清理策略处理。 -
彻底删除过程应记录审计事件；用户账号级删除另按账户删除流程执行。

------------------------------------------------------------------------

## 4. 逻辑 ERD

### 4.1 全局领域关系图

``` mermaid
erDiagram
    USER ||--o{ LEARNING_PROJECT : owns
    USER ||--o{ LEARNING_CONTENT : owns
    USER ||--o{ LEARNING_SESSION : records

    LEARNING_PROJECT ||--o{ PROJECT_GOAL : has
    LEARNING_PROJECT ||--o{ ROADMAP : versions
    ROADMAP ||--o{ STAGE : contains
    STAGE ||--o{ MILESTONE : contains
    LEARNING_PROJECT ||--o{ LEARNING_TASK : contains
    MILESTONE o|--o{ LEARNING_TASK : groups
    LEARNING_TASK }o--o{ LEARNING_TASK : depends_on

    LEARNING_CONTENT ||--o{ PROJECT_CONTENT : reused_in
    LEARNING_PROJECT ||--o{ PROJECT_CONTENT : includes
    LEARNING_CONTENT ||--o{ CONTENT_RELATION : source
    LEARNING_CONTENT ||--o{ CONTENT_RELATION : target

    LEARNING_SESSION ||--o{ SESSION_CONTENT : covers
    LEARNING_CONTENT ||--o{ SESSION_CONTENT : referenced_by
    LEARNING_SESSION ||--o{ SESSION_PROJECT : associated_with
    LEARNING_PROJECT ||--o{ SESSION_PROJECT : references
    LEARNING_SESSION ||--o{ LEARNING_EVIDENCE : produces
    LEARNING_TASK ||--o{ TASK_SESSION : progressed_by
    LEARNING_SESSION ||--o{ TASK_SESSION : contributes

    PROJECT_CONTENT ||--o{ MASTERY_ASSESSMENT : assessed_in
    LEARNING_CONTENT ||--o{ OVERALL_MASTERY_ASSESSMENT : evaluated_globally
    LEARNING_CONTENT ||--o| OVERALL_MASTERY : has_snapshot

    PROJECT_CONTENT ||--o{ REVIEW_ITEM : reviewed_in
    REVIEW_ITEM ||--o{ REVIEW_RECORD : has_history
    REVIEW_RECORD ||--o{ REVIEW_SHARE : shared_as_evidence

    USER ||--o{ DAILY_LEARNING_GOAL : sets
    USER ||--o{ SCHEDULED_ACTIVITY : schedules
    LEARNING_TASK o|--o{ SCHEDULED_ACTIVITY : planned_as
    REVIEW_ITEM o|--o{ SCHEDULED_ACTIVITY : planned_for

    USER ||--o{ CHANGE_PROPOSAL : receives
```

### 4.2 学习规划 ERD

``` mermaid
erDiagram
    LEARNING_PROJECT ||--o{ PROJECT_GOAL : contains
    LEARNING_PROJECT ||--o{ ROADMAP : has_versions
    ROADMAP ||--o{ STAGE : contains
    STAGE ||--o{ MILESTONE : contains
    LEARNING_PROJECT ||--o{ LEARNING_TASK : owns
    MILESTONE o|--o{ LEARNING_TASK : groups
    LEARNING_TASK ||--o{ TASK_DEPENDENCY : predecessor
    LEARNING_TASK ||--o{ TASK_DEPENDENCY : successor
    PROJECT_GOAL }o--o{ STAGE : achieved_by
```

`ProjectGoal` 与 `Stage` 的多对多关系应使用 `GoalStage`
关联表实现。`TaskDependency`
也应使用独立关联表，不能使用自引用数组字段。

### 4.3 学习内容与学习活动 ERD

``` mermaid
erDiagram
    LEARNING_CONTENT ||--o{ PROJECT_CONTENT : used_by
    LEARNING_PROJECT ||--o{ PROJECT_CONTENT : includes
    LEARNING_CONTENT ||--o{ CONTENT_RELATION : source
    LEARNING_CONTENT ||--o{ CONTENT_RELATION : target

    LEARNING_SESSION ||--o{ SESSION_CONTENT : covers
    LEARNING_CONTENT ||--o{ SESSION_CONTENT : covered_in
    LEARNING_SESSION ||--o{ SESSION_PROJECT : linked_to
    LEARNING_PROJECT ||--o{ SESSION_PROJECT : references
    LEARNING_SESSION ||--o{ LEARNING_EVIDENCE : generates
    LEARNING_TASK ||--o{ TASK_SESSION : tracked_by
    LEARNING_SESSION ||--o{ TASK_SESSION : advances
```

### 4.4 掌握度、复习与调度 ERD

``` mermaid
erDiagram
    PROJECT_CONTENT ||--o{ MASTERY_ASSESSMENT : has
    LEARNING_CONTENT ||--o{ OVERALL_MASTERY_ASSESSMENT : has
    LEARNING_CONTENT ||--o| OVERALL_MASTERY : current

    PROJECT_CONTENT ||--o{ REVIEW_ITEM : scheduled
    REVIEW_ITEM ||--o{ REVIEW_RECORD : history
    REVIEW_RECORD ||--o{ REVIEW_SHARE : referenced_by

    USER ||--o{ DAILY_LEARNING_GOAL : configures
    USER ||--o{ SCHEDULED_ACTIVITY : owns
    LEARNING_TASK o|--o{ SCHEDULED_ACTIVITY : schedules
    REVIEW_ITEM o|--o{ SCHEDULED_ACTIVITY : schedules
```

------------------------------------------------------------------------

## 5. 核心实体数据字典

以下字段名采用 snake_case。所有业务表建议统一包含
`id`、`created_at`、`updated_at`；需要软删除的表额外包含
`deleted_at`。时间统一使用带时区语义的时间戳；业务日期使用
DATE。数据库具体类型以目标数据库为准。

### 5.1 身份与项目规划

#### User

  字段           类型建议       约束/说明
  -------------- -------------- ------------------------------
  id             UUID           PK
  email          VARCHAR(320)   UNIQUE，若采用邮箱登录
  display_name   VARCHAR(80)    NOT NULL
  status         VARCHAR(20)    active / disabled / deleting
  created_at     TIMESTAMPTZ    NOT NULL
  updated_at     TIMESTAMPTZ    NOT NULL

认证凭据应放入独立凭据表或由认证模块管理，不在此重复设计密码字段。

#### LearningProject

  ---------------------------------------------------------------------------
  字段                        类型建议                约束/说明
  --------------------------- ----------------------- -----------------------
  id                          UUID                    PK

  owner_id                    UUID                    FK -\> User

  name                        VARCHAR(160)            NOT NULL

  description                 TEXT                    NULL

  project_type                VARCHAR(30)             study /
                                                      skill_development /
                                                      challenge / deliverable

  status                      VARCHAR(20)             使用项目状态机

  importance_level            SMALLINT                1--5

  importance_override_until   TIMESTAMPTZ             手动重要程度生效期限

  deadline_type               VARCHAR(20)             none / soft / fixed

  deadline_at                 TIMESTAMPTZ             可空；fixed
                                                      不允许普通延期

  archived_at                 TIMESTAMPTZ             归档时间

  reopened_at                 TIMESTAMPTZ             最近一次重新开启时间
  ---------------------------------------------------------------------------

约束： - `importance_level BETWEEN 1 AND 5`。 - `deadline_type = none`
时 `deadline_at IS NULL`。 - `deadline_type IN (soft, fixed)` 时
`deadline_at IS NOT NULL`。 - `fixed` 日期修改必须走特殊确认流程。 -
项目归档不删除其子数据。

#### ProjectGoal

  字段                类型建议       约束/说明
  ------------------- -------------- ------------------------------------
  id                  UUID           PK
  project_id          UUID           FK -\> LearningProject
  title               VARCHAR(200)   NOT NULL
  description         TEXT           NULL
  goal_type           VARCHAR(30)    outcome / performance / completion
  target_definition   JSONB          结构化目标定义；关键字段应另设列
  status              VARCHAR(20)    active / achieved / cancelled
  target_at           TIMESTAMPTZ    NULL

#### Roadmap

  字段             类型建议      约束/说明
  ---------------- ------------- -----------------------------------
  id               UUID          PK
  project_id       UUID          FK -\> LearningProject
  version          INTEGER       NOT NULL
  status           VARCHAR(20)   draft / active / superseded
  source           VARCHAR(20)   user / ai / template
  change_summary   TEXT          NULL
  confirmed_at     TIMESTAMPTZ   用户确认时间
  created_by       UUID          FK -\> User，可按系统操作语义为空

约束：`UNIQUE(project_id, version)`；一个项目最多一个 active
roadmap（可用部分唯一索引实现）。

#### Stage

  --------------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- --------------------------
  id                      UUID                    PK

  roadmap_id              UUID                    FK -\> Roadmap

  name                    VARCHAR(160)            NOT NULL

  description             TEXT                    NULL

  sequence_no             INTEGER                 推荐顺序，不代表强制线性

  status                  VARCHAR(20)             pending / active /
                                                  completed / skipped

  completion_rule         JSONB                   可选完成条件

  completion_method       VARCHAR(20)             user / system_confirmed /
                                                  ai_suggested

  started_at              TIMESTAMPTZ             NULL

  completed_at            TIMESTAMPTZ             NULL
  --------------------------------------------------------------------------

约束：`UNIQUE(roadmap_id, sequence_no)`。

#### Milestone

  字段              类型建议       约束/说明
  ----------------- -------------- ---------------------------------------------
  id                UUID           PK
  stage_id          UUID           FK -\> Stage
  title             VARCHAR(200)   NOT NULL
  description       TEXT           NULL
  status            VARCHAR(20)    pending / in_progress / completed / skipped
  completion_rule   JSONB          可选
  completed_at      TIMESTAMPTZ    NULL

#### GoalStage

  字段           类型建议      约束/说明
  -------------- ------------- -----------------------
  goal_id        UUID          PK/FK -\> ProjectGoal
  stage_id       UUID          PK/FK -\> Stage
  contribution   VARCHAR(20)   primary / supporting

复合主键：`(goal_id, stage_id)`。

#### LearningTask

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  project_id              UUID                    FK -\> LearningProject

  milestone_id            UUID                    FK -\> Milestone，可空

  title                   VARCHAR(200)            NOT NULL

  description             TEXT                    NULL

  status                  VARCHAR(20)             todo / in_progress /
                                                  blocked / completed /
                                                  cancelled

  priority                SMALLINT                1--5

  due_at                  TIMESTAMPTZ             NULL

  estimated_minutes       INTEGER                 CHECK \>= 0

  completed_at            TIMESTAMPTZ             NULL

  created_source          VARCHAR(20)             user / ai / template
  -----------------------------------------------------------------------

`milestone_id` 可空，支持不设置阶段的简单项目。

#### TaskDependency

  字段                 类型建议      约束/说明
  -------------------- ------------- ----------------------------------
  task_id              UUID          PK/FK -\> LearningTask，后继任务
  depends_on_task_id   UUID          PK/FK -\> LearningTask，前置任务
  dependency_type      VARCHAR(20)   finish_to_start

约束： - 复合主键 `(task_id, depends_on_task_id)`。 -
`task_id <> depends_on_task_id`。 -
两个任务必须属于同一项目；由领域服务校验。 -
新增依赖时检查环路，首版不允许形成循环依赖。

### 5.2 学习内容

#### LearningContent

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  content_type            VARCHAR(30)             knowledge_point /
                                                  skill_point /
                                                  language_item

  title                   VARCHAR(200)            NOT NULL

  description             TEXT                    NULL

  parent_id               UUID                    自引用 FK，可空

  metadata                JSONB                   仅放类型扩展属性

  created_source          VARCHAR(20)             user / ai_confirmed /
                                                  template

  status                  VARCHAR(20)             active / archived
  -----------------------------------------------------------------------

**不要**建立 `(owner_id, title)` 唯一约束。可建立
`(owner_id, content_type)`
普通索引和标题检索索引；相似度提示由应用服务或数据库全文/向量能力实现，不作为创建硬阻断。

#### ContentRelation

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  source_content_id       UUID                    FK -\> LearningContent

  target_content_id       UUID                    FK -\> LearningContent

  relation_type           VARCHAR(30)             prerequisite / related
                                                  / contains / extends /
                                                  alternative
  -----------------------------------------------------------------------

约束： - `source_content_id <> target_content_id`。 - 对
`(source_content_id, target_content_id, relation_type)` 建唯一约束。 -
两端内容必须属于同一 owner。 - prerequisite 关系新增时可检查循环依赖。

#### ProjectContent

  ------------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- ------------------------
  id                      UUID                    PK

  project_id              UUID                    FK -\> LearningProject

  content_id              UUID                    FK -\> LearningContent

  target_mastery_level    SMALLINT                0--4

  current_status          VARCHAR(20)             not_started / learning /
                                                  reviewing / achieved /
                                                  waived

  priority                SMALLINT                1--5

  auto_review_enabled     BOOLEAN                 项目上下文自动复习开关

  linked_at               TIMESTAMPTZ             NOT NULL

  achieved_at             TIMESTAMPTZ             NULL
  ------------------------------------------------------------------------

约束：`UNIQUE(project_id, content_id)`；项目 owner 必须与内容 owner
一致。该约束需要应用服务校验，或在数据库层采用复合外键/触发器加强。

### 5.3 学习活动与证据

#### LearningSession

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  primary_project_id      UUID                    FK -\>
                                                  LearningProject，可空

  title                   VARCHAR(200)            NULL

  activity_type           VARCHAR(30)             theory / practice /
                                                  review / mixed / other

  started_at              TIMESTAMPTZ             NOT NULL

  ended_at                TIMESTAMPTZ             NOT NULL

  duration_minutes        INTEGER                 CHECK \>= 0

  summary                 TEXT                    NULL

  what_done               TEXT                    NULL

  problems                TEXT                    NULL

  next_steps              TEXT                    NULL

  created_source          VARCHAR(20)             user / timer / imported
  -----------------------------------------------------------------------

约束： - `ended_at >= started_at`。 - `duration_minutes`
应由服务端根据时间区间或用户确认的有效学习时长生成，不能仅信任前端。 -
`primary_project_id` 若非空，项目 owner 必须等于 session owner。 -
主要项目最多一个；不允许通过多条关联复制时长。

#### SessionProject

  字段            类型建议      约束/说明
  --------------- ------------- ---------------------------
  session_id      UUID          PK/FK -\> LearningSession
  project_id      UUID          PK/FK -\> LearningProject
  relation_role   VARCHAR(20)   primary / related
  created_at      TIMESTAMPTZ   NOT NULL

约束： - `(session_id, project_id)` 唯一。 - `primary` 关联必须与
`LearningSession.primary_project_id` 一致。 - 一个 session 最多一个
primary；可使用部分唯一索引。 - 若 session 的 primary_project_id
为空，则允许没有 primary 关联。 - 所有项目必须属于 session owner。

实现上可以只保留 `LearningSession.primary_project_id`
作为权威字段，`SessionProject` 不重复保存 primary
角色；更推荐该方案：`SessionProject` 仅存 related
项目。这样避免双重事实。本文 ERD 中的 primary 语义以
`LearningSession.primary_project_id` 为准。

#### SessionContent

  字段                类型建议      约束/说明
  ------------------- ------------- -------------------------------
  session_id          UUID          PK/FK -\> LearningSession
  content_id          UUID          PK/FK -\> LearningContent
  contribution_type   VARCHAR(20)   primary / covered / practiced
  note                TEXT          NULL

复合主键 `(session_id, content_id)`；session 与 content
必须属于同一用户。

#### LearningEvidence

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  session_id              UUID                    FK -\>
                                                  LearningSession，可空

  evidence_type           VARCHAR(30)             note / code / image /
                                                  file / link /
                                                  test_result

  title                   VARCHAR(200)            NULL

  description             TEXT                    NULL

  storage_key             TEXT                    文件存储对象键，可空

  external_url            TEXT                    外部链接，可空

  mime_type               VARCHAR(120)            NULL

  size_bytes              BIGINT                  CHECK \>= 0

  created_at              TIMESTAMPTZ             NOT NULL
  -----------------------------------------------------------------------

附件二进制内容存储在对象存储或文件服务中，数据库保存元数据与存储键。原图保留策略、压缩版本和访问授权由文件服务定义。

#### TaskSession

  字段                   类型建议   约束/说明
  ---------------------- ---------- ---------------------------
  task_id                UUID       PK/FK -\> LearningTask
  session_id             UUID       PK/FK -\> LearningSession
  contribution_minutes   INTEGER    CHECK \>= 0
  note                   TEXT       NULL

复合主键 `(task_id, session_id)`。`contribution_minutes`
仅用于任务贡献分摊，不能累加成超过 session
总时长的项目投入；若要记录多任务分摊，应校验同一 session
的分摊总和不超过有效时长。

### 5.4 掌握度与复习

#### MasteryAssessment

  字段                  类型建议       约束/说明
  --------------------- -------------- ---------------------------
  id                    UUID           PK
  owner_id              UUID           FK -\> User
  project_content_id    UUID           FK -\> ProjectContent
  system_level          SMALLINT       0--4，可空
  self_level            SMALLINT       0--4，可空
  recognized_level      SMALLINT       0--4
  confidence            NUMERIC(4,3)   0--1
  disagreement_status   VARCHAR(20)    none / mild / significant
  assessment_reason     TEXT           评价说明
  assessment_method     VARCHAR(80)    规则/模型版本
  assessed_at           TIMESTAMPTZ    NOT NULL
  confirmed_at          TIMESTAMPTZ    NULL

该表是历史记录，原则上只追加，不覆盖。当前项目认定状态可由最新有效评价投影得到，必要时在
`ProjectContent` 增加 `current_mastery_level`
快照字段，但必须明确它是可重建的缓存。

#### OverallMasteryAssessment

  字段                 类型建议       约束/说明
  -------------------- -------------- ------------------------
  id                   UUID           PK
  owner_id             UUID           FK -\> User
  content_id           UUID           FK -\> LearningContent
  system_level         SMALLINT       0--4
  confidence           NUMERIC(4,3)   0--1
  assessment_method    VARCHAR(80)    算法/模型版本
  evidence_cutoff_at   TIMESTAMPTZ    本次证据时间边界
  explanation          TEXT           可解释摘要
  evaluated_at         TIMESTAMPTZ    NOT NULL

#### OverallMastery

  字段                   类型建议       约束/说明
  ---------------------- -------------- ---------------------------------
  owner_id               UUID           PK/FK -\> User
  content_id             UUID           PK/FK -\> LearningContent
  latest_assessment_id   UUID           FK -\> OverallMasteryAssessment
  current_level          SMALLINT       0--4
  confidence             NUMERIC(4,3)   0--1
  evaluated_at           TIMESTAMPTZ    NOT NULL

该表为当前快照；历史以 `OverallMasteryAssessment`
为准。`(owner_id, content_id)` 唯一。

#### ReviewItem

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  project_content_id      UUID                    FK -\>
                                                  ProjectContent，可空

  content_id              UUID                    FK -\> LearningContent

  scope_type              VARCHAR(30)             project_content /
                                                  personal_content

  review_type             VARCHAR(30)             spaced_repetition /
                                                  wrong_question / manual

  status                  VARCHAR(20)             active / paused /
                                                  completed / disabled

  next_review_at          TIMESTAMPTZ             NULL

  interval_seconds        BIGINT                  CHECK \>= 0

  ease_factor             NUMERIC(6,3)            算法参数，可空

  algorithm_version       VARCHAR(40)             NOT NULL

  auto_review_enabled     BOOLEAN                 NOT NULL

  created_at              TIMESTAMPTZ             NOT NULL
  -----------------------------------------------------------------------

约束： - `scope_type = project_content` 时 `project_content_id`
必须非空。 - `scope_type = personal_content` 时 `project_content_id`
必须为空。 - `content_id` 必须与 `project_content_id`
所引用的内容一致。 - 每个 scope、content、review_type
的有效复习状态应有唯一性约束，避免重复创建活动队列。 -
算法状态只代表当前调度状态；每次真实复习写入 `ReviewRecord`。

#### ReviewRecord

  ---------------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- ---------------------------
  id                      UUID                    PK

  review_item_id          UUID                    FK -\> ReviewItem

  learning_session_id     UUID                    FK -\>
                                                  LearningSession，可空

  reviewed_at             TIMESTAMPTZ             NOT NULL

  performance             VARCHAR(20)             again / hard / good / easy
                                                  或统一反馈枚举

  user_feedback           TEXT                    NULL

  error_count             INTEGER                 CHECK \>= 0

  duration_minutes        INTEGER                 CHECK \>= 0

  resulting_level         SMALLINT                0--4，可空

  scheduler_snapshot      JSONB                   记录本次算法输入/输出摘要
  ---------------------------------------------------------------------------

#### ReviewShare

  字段                        类型建议      约束/说明
  --------------------------- ------------- -----------------------------------------
  id                          UUID          PK
  source_review_record_id     UUID          FK -\> ReviewRecord
  target_project_content_id   UUID          FK -\> ProjectContent
  share_type                  VARCHAR(30)   evidence_only / propose_schedule_update
  created_at                  TIMESTAMPTZ   NOT NULL
  accepted_at                 TIMESTAMPTZ   NULL

共享记录默认只作为证据。若目标项目要改变自己的复习状态，必须产生自己的确认记录或调度事件。

### 5.5 时间调度与 AI 提案

#### DailyLearningGoal

  字段             类型建议      约束/说明
  ---------------- ------------- -------------
  id               UUID          PK
  owner_id         UUID          FK -\> User
  goal_date        DATE          NOT NULL
  target_minutes   INTEGER       CHECK \>= 0
  note             TEXT          NULL
  created_at       TIMESTAMPTZ   NOT NULL
  updated_at       TIMESTAMPTZ   NOT NULL

约束：`UNIQUE(owner_id, goal_date)`。目标时长不是硬上限。

#### ScheduledActivity

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  scheduled_date          DATE                    NOT NULL

  starts_at               TIMESTAMPTZ             NULL

  ends_at                 TIMESTAMPTZ             NULL

  planned_minutes         INTEGER                 CHECK \>= 0

  activity_type           VARCHAR(30)             task / review /
                                                  free_learning

  task_id                 UUID                    FK -\>
                                                  LearningTask，可空

  review_item_id          UUID                    FK -\> ReviewItem，可空

  content_id              UUID                    FK -\>
                                                  LearningContent，可空

  status                  VARCHAR(20)             proposed / scheduled /
                                                  completed / skipped /
                                                  cancelled

  source                  VARCHAR(20)             user / ai

  proposal_id             UUID                    FK -\>
                                                  ChangeProposal，可空
  -----------------------------------------------------------------------

约束： - `ends_at >= starts_at`（两者都非空时）。 -
task、review_item、content 的关联组合需要由领域服务验证。 - 标记
completed 不代表已经产生
LearningSession；完成状态应以实际记录为依据或要求关联实际 session。 -
同一用户同一时段冲突检查由调度服务完成；允许用户明确覆盖冲突。

#### ChangeProposal

  -----------------------------------------------------------------------
  字段                    类型建议                约束/说明
  ----------------------- ----------------------- -----------------------
  id                      UUID                    PK

  owner_id                UUID                    FK -\> User

  proposal_type           VARCHAR(40)             content_create /
                                                  roadmap_change /
                                                  schedule_change /
                                                  mastery_change

  target_type             VARCHAR(40)             目标实体类型

  target_id               UUID                    目标实体 ID，可空

  payload                 JSONB                   提议变更

  reason                  TEXT                    提议理由

  status                  VARCHAR(20)             pending / accepted /
                                                  rejected / expired /
                                                  applied

  created_at              TIMESTAMPTZ             NOT NULL

  reviewed_at             TIMESTAMPTZ             NULL

  applied_at              TIMESTAMPTZ             NULL
  -----------------------------------------------------------------------

`payload`
不能绕过领域校验。接受提案后仍由对应领域服务执行命令；提案状态与实际业务变更应在同一事务或可靠的
outbox 流程中保持一致。

------------------------------------------------------------------------

## 6. 关系基数与关键约束汇总

  ---------------------------------------------------------------------------------
  关系                       基数                    说明
  -------------------------- ----------------------- ------------------------------
  User -\> LearningProject   1:N                     一个用户拥有多个项目

  LearningProject -\>        1:N                     项目可有多个目标
  ProjectGoal                                        

  LearningProject -\>        1:N                     路线支持版本
  Roadmap                                            

  Roadmap -\> Stage          1:N                     每个路线版本包含多个阶段

  Stage -\> Milestone        1:N                     阶段可有多个里程碑

  Project -\> LearningTask   1:N                     任务属于项目；阶段可为空

  LearningContent -\>        1:N                     内容可被多个项目使用
  ProjectContent                                     

  LearningProject -\>        1:N                     项目关联多项学习内容
  ProjectContent                                     

  LearningSession -\>        1:N                     一次学习活动关联多个内容
  SessionContent                                     

  LearningSession -\>        1:N                     一个主要项目 + 多个关联项目
  SessionProject                                     

  LearningSession -\>        1:N                     一次活动可产生多项证据
  LearningEvidence                                   

  LearningTask \<-\>         M:N                     通过 TaskSession 关联
  LearningSession                                    

  ProjectContent -\>         1:N                     项目上下文评价历史
  MasteryAssessment                                  

  LearningContent -\>        1:N                     个人总体评价历史
  OverallMasteryAssessment                           

  ReviewItem -\>             1:N                     一个复习队列项有多次复习历史
  ReviewRecord                                       

  User -\> DailyLearningGoal 1:N                     每日一个目标

  User -\> ScheduledActivity 1:N                     用户有多个计划活动
  ---------------------------------------------------------------------------------

### 6.1 必须由服务层校验的跨表规则

普通 CHECK
约束无法直接验证跨表所有权和语义，以下规则应由领域服务执行，必要时用复合外键或触发器补强：

1.  项目与其 Goal、Task、Roadmap 必须属于同一用户。
2.  ProjectContent 的 project 与 content 必须属于同一用户。
3.  SessionContent 的 session 与 content 必须属于同一用户。
4.  LearningSession 的 primary_project 必须属于 session owner。
5.  SessionProject 的关联项目必须属于 session
    owner，且不得与主项目重复。
6.  ReviewItem 的 project_content 与 content 必须一致。
7.  ReviewShare 的目标项目内容与来源复习证据必须属于同一用户。
8.  TaskDependency 两端任务必须属于同一项目，且不能形成环。
9.  AI 提案的 owner 必须拥有目标实体。
10. ScheduledActivity 的关联对象必须与 owner 一致。

------------------------------------------------------------------------

## 7. 索引建议

以下为首版候选索引。最终应依据真实查询、数据量和执行计划调整。

  ------------------------------------------------------------------------------------------------
  表                         索引                                          用途
  -------------------------- --------------------------------------------- -----------------------
  LearningProject            `(owner_id, status, updated_at DESC)`         项目列表

  LearningProject            `(owner_id, archived_at)`                     活跃/归档筛选

  ProjectGoal                `(project_id, status)`                        项目目标列表

  Roadmap                    `(project_id, version DESC)`                  路线版本查询

  Stage                      `(roadmap_id, sequence_no)` UNIQUE            阶段顺序

  LearningTask               `(project_id, status, priority)`              项目任务列表

  LearningTask               `(project_id, due_at)`                        截止任务查询

  TaskDependency             `(depends_on_task_id)`                        反向依赖查询

  LearningContent            `(owner_id, content_type, status)`            用户内容列表

  LearningContent            `(owner_id, title)`                           标题检索，非唯一

  ProjectContent             `(project_id, current_status)`                项目学习内容状态

  ProjectContent             `(content_id)`                                内容被哪些项目引用

  LearningSession            `(owner_id, started_at DESC)`                 学习历史

  LearningSession            `(primary_project_id, started_at DESC)`       项目实际投入统计

  SessionContent             `(content_id, session_id)`                    内容学习历史

  LearningEvidence           `(owner_id, created_at DESC)`                 成果档案

  MasteryAssessment          `(project_content_id, assessed_at DESC)`      项目掌握评价历史

  OverallMasteryAssessment   `(owner_id, content_id, evaluated_at DESC)`   总体掌握评价历史

  ReviewItem                 `(owner_id, status, next_review_at)`          到期复习队列

  ReviewRecord               `(review_item_id, reviewed_at DESC)`          复习历史

  DailyLearningGoal          `(owner_id, goal_date)` UNIQUE                每日目标

  ScheduledActivity          `(owner_id, scheduled_date, starts_at)`       日程查询

  ChangeProposal             `(owner_id, status, created_at DESC)`         待处理提案
  ------------------------------------------------------------------------------------------------

如果使用 PostgreSQL，可考虑： - 对活跃 Roadmap
建立部分唯一索引：每个项目最多一个 `status = 'active'`。 - 对有效
ReviewItem 建立部分唯一索引，避免同一上下文重复启用同类型队列。 -
对标题/描述使用全文检索；相似度提示可后续评估 pg_trgm
或向量检索，不应在首版将其变为创建阻断。

------------------------------------------------------------------------

## 8. 删除、归档与数据保留策略

  对象                       默认操作               说明
  -------------------------- ---------------------- ----------------------------------
  LearningProject            归档                   默认不物理删除
  Roadmap                    标记 superseded        保留路线历史
  Stage / Milestone / Task   软删除或取消           保留执行历史引用
  LearningContent            归档                   若被项目引用，不允许直接物理删除
  ProjectContent             解除关联/归档          不删除共享内容
  LearningSession            保留                   更正需保留审计历史
  LearningEvidence           按引用和用户授权管理   删除附件需检查引用
  MasteryAssessment          保留历史               不覆盖历史评价
  ReviewRecord               保留历史               不覆盖历史复习
  ScheduledActivity          取消/删除计划          不影响 LearningSession
  ChangeProposal             保留状态历史           便于追踪 AI 建议

### 8.1 彻底删除项目的推荐事务流程

1.  校验用户身份和项目所有权。
2.  检查项目是否处于允许删除的状态。
3.  展示影响范围：任务、路线、项目内容关联、复习项、日程和主要归属学习记录。
4.  用户二次确认。
5.  将项目状态置为 deleting，阻止并发写入。
6.  删除项目专属规划数据与项目关联。
7.  对主要归属学习记录：默认解除
    `primary_project_id`，保留为个人历史记录；如产品未来允许用户选择彻底删除这些记录，必须另设明确选项。
8.  删除仅被该项目引用且确实属于项目的派生调度数据；共享内容和共享证据不删除。
9.  写入审计事件，完成项目删除。
10. 通过后台任务清理无引用附件或临时资源。

建议首版优先实现归档，不急于提供彻底删除项目的复杂级联能力。

------------------------------------------------------------------------

## 9. 状态枚举建议

枚举值应集中管理，避免前后端自由拼写。

  -----------------------------------------------------------------------
  对象                                建议状态
  ----------------------------------- -----------------------------------
  LearningProject                     active / paused / completed /
                                      archived / deleting

  Roadmap                             draft / active / superseded

  Stage                               pending / active / completed /
                                      skipped

  Milestone                           pending / in_progress / completed /
                                      skipped

  LearningTask                        todo / in_progress / blocked /
                                      completed / cancelled

  LearningContent                     active / archived

  ProjectContent                      not_started / learning / reviewing
                                      / achieved / waived

  ReviewItem                          active / paused / completed /
                                      disabled

  ScheduledActivity                   proposed / scheduled / completed /
                                      skipped / cancelled

  ChangeProposal                      pending / accepted / rejected /
                                      expired / applied
  -----------------------------------------------------------------------

状态机迁移必须通过领域服务执行。前端不能任意写入状态。

------------------------------------------------------------------------

## 10. 领域服务边界建议（Rust 实现参考）

建议以业务上下文组织应用服务，而不是按数据库表机械地建立大量 CRUD
service。

  ----------------------------------------------------------------------------------------------
  应用服务                            主要命令/查询
  ----------------------------------- ----------------------------------------------------------
  ProjectService                      创建、暂停、恢复、完成、归档、重新开启项目

  RoadmapService                      创建路线版本、确认路线、调整阶段、跳过阶段

  TaskService                         创建任务、更新状态、建立依赖、完成任务

  LearningContentService              创建内容、查找相似内容、复用内容、归档内容

  ProjectContentService               将内容加入项目、设置目标等级、解除项目关联

  LearningSessionService              开始/结束/创建学习记录、关联项目和内容

  EvidenceService                     上传证据、关联学习记录、管理附件元数据

  MasteryService                      生成系统评价、记录自评、确认认定等级、计算总体掌握

  ReviewService                       创建复习项、提交复习结果、生成下次复习建议、共享复习证据

  SchedulerService                    生成日程建议、检测冲突、调整计划、统计目标与实际投入

  ProposalService                     创建、接受、拒绝、过期和应用 AI 提案
  ----------------------------------------------------------------------------------------------

领域服务负责业务规则；Repository 负责持久化；Handler 负责 HTTP
输入输出。不要让 Handler 直接拼接跨表事务。

------------------------------------------------------------------------

## 11. 首版实施优先级

建议按依赖顺序实施，而不是一次性创建所有表。

### P0：基础身份与规划

-   User / ownership 基础
-   LearningProject
-   ProjectGoal
-   Roadmap / Stage / Milestone
-   LearningTask / TaskDependency
-   项目状态机、归档机制

### P1：学习内容与实际记录

-   LearningContent
-   ContentRelation
-   ProjectContent
-   LearningSession
-   SessionContent
-   SessionProject
-   LearningEvidence
-   TaskSession

### P2：掌握度与复习

-   MasteryAssessment
-   OverallMasteryAssessment / OverallMastery
-   ReviewItem
-   ReviewRecord
-   ReviewShare
-   错题独立模型及题库模型（单独评审）

### P3：日程与 AI 提案

-   DailyLearningGoal
-   ScheduledActivity
-   ChangeProposal
-   AI 计划建议与确认流程
-   调度算法及冲突处理

生活记录、健康数据、第三方设备接入和高级 AI
分析不应阻塞上述核心领域模型。

------------------------------------------------------------------------

## 12. 尚未锁定、实施前必须补充的设计项

本轮已经明确逻辑实体和主要关系，但以下事项不应在没有讨论的情况下擅自固化：

1.  **掌握等级 L0--L4 的最终定义和各内容类型判定量表。**
2.  **错题与题库的完整领域模型**：Question、QuestionVersion、Attempt、WrongAnswer、QuestionContent
    等。
3.  **语言学习内容扩展模型**：词形、词性、释义、例句、发音、词族及语言专属复习策略。
4.  **学习活动计时规则**：暂停、跨午夜、手动补录、有效时长修正和审计。
5.  **项目删除的最终产品交互**：彻底删除时是否提供删除主要归属学习记录的显式选项。
6.  **个人总体掌握算法**：证据权重、时间衰减、冲突处理、置信度校准。
7.  **复习共享的具体语义**：共享仅作为证据，还是允许用户将复习结果转化为目标项目的评价输入。
8.  **数据隐私与附件授权**：证据跨项目查看的权限、撤销机制及健康数据授权边界。
9.  **多时区策略**：用户时区变更后，历史 `DATE` 与时间戳如何解释。
10. **并发与审计**：关键状态迁移的乐观锁、幂等键、审计事件和事务边界。

------------------------------------------------------------------------

## 13. 评审结论与下一步

当前模型可以作为 Learna
数据库设计的逻辑基线，但不建议直接根据本文一次性生成全部 migration。

推荐下一步顺序：

1.  对本文件的实体和关系做第二轮审查，尤其是学习活动、掌握度、复习和跨项目共享。
2.  单独完成题库/错题领域模型。
3.  确定 L0--L4 的评价规范。
4.  形成 API Contract，明确每个聚合的命令和查询。
5.  根据确定的技术栈生成 PostgreSQL DDL 和分阶段 SQLx migrations。
6.  为关键业务不变量编写领域测试和数据库约束测试。

**最终原则：先保证业务事实、归属关系和状态迁移正确，再优化表结构和查询性能。**
