# Learna 领域模型与 ERD 设计文档

**版本：** Domain Model & ERD V1.2\
**状态：** 领域模型评审后设计稿（V1.2 补齐模块设计发现的 6 处模型缺口）\
**适用范围：** Learna 首版多用户学习管理平台\
**设计依据：** Learna PRD V1.0 与领域模型评审确认结果

**V1.2 变更记录（2026-11-27）**

| 变更 | 位置 | 依据 |
|---|---|---|
| `LearningProject` 增 `theory_enabled` / `practice_enabled` | §5.1 | PRD §6.1；模块设计 M02 §5 |
| 新增 `ProjectPriorityOverride` / `ProjectCommitmentRule` / `ProjectEvent` | §5.6 | PRD §5.3、§5.4；TAD §3.1、§3.10；M02 §5 |
| 新增 `ContentTag` / `LearningContentTag` | §5.2 | PRD §6.3；TAD §3.3；M03 §5 |
| 新增 `ReviewPolicy` / `ReviewRescheduleHistory` | §5.4 | PRD §8.5；TAD §3.6；M07 §5 |
| 新增题库与练习 11 张表 | §5.7 | PRD §9；TAD §3.7；M08 §5 |
| 新增实践成果档案 4 张表 | §5.8 | PRD §10；TAD §3.8；M09 §5 |
| 新增生活记录 4 张表 + `AIDataPermission` | §5.9 | PRD §12；TAD §3.11、§3.12；M10 §5 |
| 补 3 张域关系图、关系基数与跨表规则 | §4.5–4.7、§6、§6.1 | 本次新增实体 |
| 索引、枚举、服务边界、实施优先级、待补项同步 | §7、§9–§12 | 本次新增实体 |
| §1.2 补第 8 条原则「派生值不入库」 | §1.2 | ADR-007 契约最小承诺的模型侧对应 |

本次修订**只补模型缺口**，不修改任何既有实体字段、关系与已确认规则。新增表全部沿用 V1.1 的删除语义分层（ADR-006）：只有承载执行历史的表才考虑软删除，其余以状态 / 归档表达退场，不新增 `deleted_at`。决策理由与放弃的方案见 **ADR-008**。

**V1.1 变更记录（2026-11-27）**

| 变更 | 位置 | 依据 |
|---|---|---|
| `ProjectGoal` 增 `parent_goal_id`（自引用，上限两层） | §5.1 | 契约决策 D-06；ADR-005 |
| `LearningTask` 增 `stage_id`（可空 FK） | §5.1 | 契约决策 D-05；ADR-005 |
| `LearningTask` 增 `deleted_at`（唯一软删表） | §5.1、§8 | 契约决策 D-17；ADR-006 |
| `LearningProject` 状态枚举补 `draft`（六值） | §9 | 契约决策 D-16；ADR-005 |
| §8 删除策略按「是否已参与执行」分层收窄 | §8、§8.1 | ADR-006 |
| §7 索引建议补 `stage_id` 与软删过滤方向 | §7 | 契约设计；ADR-007 |
| §12 待补项第 5 条改为已移交 | §12 | 契约决策 D-20 |

本次修订**只做已授权补列与表述收窄**，不改变任何既有实体、关系与业务不变量。

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
8.  **派生值不入库**（V1.2 新增）：达标进度、未达标风险、掌握快照、相似内容提示等可由原始事实计算出的结果不单独建表，按需聚合；确需缓存时必须显式标注为「可重建」，并说明重建来源。

------------------------------------------------------------------------

## 2. 领域划分与上下文

Learna 分为九个业务上下文。前六个是 V1.0 已确认的，后三个由 V1.2 补入（此前只有 PRD 要求或 TAD 概念，没有模型）。

| 上下文 | 职责 | 核心实体 |
|---|---|---|
| Identity & Ownership | 用户身份与数据所有权 | User |
| Learning Planning | 项目、目标、路线、任务，以及项目级调度配置与事件 | LearningProject、ProjectGoal、Roadmap、Stage、Milestone、LearningTask、ProjectPriorityOverride、ProjectCommitmentRule、ProjectEvent |
| Learning Content | 可跨项目复用的学习内容与标签 | LearningContent、ContentRelation、ProjectContent、ContentTag、LearningContentTag |
| Learning Activity | 实际学习活动、关联和证据 | LearningSession、SessionContent、SessionProject、LearningEvidence、TaskSession |
| Mastery & Review | 掌握度评价、复习状态、复习历史与策略 | MasteryAssessment、OverallMastery、ReviewItem、ReviewRecord、ReviewShare、ReviewPolicy、ReviewRescheduleHistory |
| Scheduling & AI Assistance | 每日目标、计划安排与 AI 提案 | DailyLearningGoal、ScheduledActivity、ChangeProposal |
| Question Bank & Practice（V1.2） | 题库、组卷、练习作答、错题状态 | Question、QuestionVersion、QuestionOption、QuestionAsset、QuestionKnowledgePoint、QuestionSet、QuestionSetItem、PracticeSession、PracticeAnswer、WrongQuestionState、QuestionReviewHistory |
| Artifact（V1.2） | 实践成果、附件、外部链接与技能点关联 | Artifact、ArtifactAsset、ArtifactLink、ArtifactContentRelation |
| Life Records & Consent（V1.2） | 可选的生活记录与 AI 数据授权 | SleepRecord、ExerciseRecord、MealRecord、BodyMeasurement、AIDataPermission |

最后三个上下文的边界是刻意的：**题库与成果可以不参与**（不练题、不记成果也能完整使用），**生活记录可以整体关闭**（PRD §12.1）。它们与核心学习域之间只允许「查询时聚合」，不允许核心流程依赖它们。

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

全局图只画核心学习链路。V1.2 新增的项目配置、题库、成果、生活记录四个域见 §4.5–4.7：项目配置与题库、成果通过 `project_id` / `owner_id` 接入主图，生活记录则完全不连线（§5.9 前置约束 1）。

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

### 4.5 项目配置与业务事件 ERD（V1.2）

``` mermaid
erDiagram
    LEARNING_PROJECT ||--o{ PROJECT_PRIORITY_OVERRIDE : marked_by
    LEARNING_PROJECT ||--o{ PROJECT_COMMITMENT_RULE : guarded_by
    LEARNING_PROJECT ||--o{ PROJECT_EVENT : logs
    USER ||--o{ PROJECT_EVENT : triggers
```

临时标记、投入保障规则、业务事件都是**项目级配置**，不进入路线结构：它们改的是调度权重与可追溯性，不是「项目里有什么」。

### 4.6 题库与练习 ERD（V1.2）

``` mermaid
erDiagram
    QUESTION ||--o{ QUESTION_VERSION : versions
    QUESTION_VERSION ||--o{ QUESTION_OPTION : offers
    QUESTION ||--o{ QUESTION_ASSET : illustrated_by
    QUESTION }o--o{ LEARNING_CONTENT : tests
    QUESTION ||--o{ QUESTION_REVIEW_HISTORY : audited_by
    QUESTION_SET ||--o{ QUESTION_SET_ITEM : contains
    QUESTION_SET_ITEM }o--|| QUESTION : picks
    PRACTICE_SESSION ||--o{ PRACTICE_ANSWER : answers
    PRACTICE_ANSWER }o--|| QUESTION_VERSION : locks
    QUESTION ||--o| WRONG_QUESTION_STATE : tracked_in
    PRACTICE_SESSION |o--o| LEARNING_SESSION : counts_as
```

选项与作答都指向**版本**而不是题目：题目可修订，历史作答不能因修订而改变判分依据。`wrong_book` 题组是动态视图，不落 `QUESTION_SET_ITEM`。

### 4.7 成果与生活记录 ERD（V1.2）

``` mermaid
erDiagram
    LEARNING_PROJECT ||--o{ ARTIFACT : produces
    ARTIFACT ||--o{ ARTIFACT_ASSET : attaches
    ARTIFACT ||--o{ ARTIFACT_LINK : links
    ARTIFACT }o--o{ LEARNING_CONTENT : demonstrates
    USER ||--o{ SLEEP_RECORD : logs
    USER ||--o{ EXERCISE_RECORD : logs
    USER ||--o{ MEAL_RECORD : logs
    USER ||--o{ BODY_MEASUREMENT : records
    USER ||--o{ AI_DATA_PERMISSION : grants
```

生活记录四表**只连 User，不连任何学习域实体**——这是刻意隔离：可选模块不能成为核心表结构的前置依赖（PRD §12.1）。

------------------------------------------------------------------------

## 5. 核心实体数据字典

以下字段名采用 snake_case。所有业务表建议统一包含
`id`、`created_at`、`updated_at`。**软删除列 `deleted_at` 仅
`LearningTask` 需要**（V1.1 收窄，见 §8）：其余对象要么物理删除，要么以状态
/ 归档表达退场，不引入 `deleted_at` 过滤成本。时间统一使用带时区语义的时间戳；业务日期使用
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

  theory_enabled              BOOLEAN                 NOT NULL DEFAULT true

  practice_enabled            BOOLEAN                 NOT NULL DEFAULT true

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

约束：

- `importance_level BETWEEN 1 AND 5`。
- `deadline_type = none` 时 `deadline_at IS NULL`；`deadline_type IN (soft, fixed)` 时 `deadline_at IS NOT NULL`。
- `fixed` 日期修改必须走特殊确认流程。
- 项目归档不删除其子数据。
- `theory_enabled` / `practice_enabled` 默认 `true`；关闭模块**只停止新增相关安排，不删除历史记录与成果**（PRD §6.1）。
- 项目类型不决定开关值：考研类项目是否关闭实践模块由用户创建时决定，DB 不硬编码「类型 → 开关」映射，服务层可给推荐默认。

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
  parent_goal_id      UUID           FK -\> ProjectGoal，可空自引用；
                                     仅两层（主目标 / 子目标）
  ---------------------------------------------------------------------

`parent_goal_id` 为 V1.1 补列：`NULL` 表示主目标，非空表示子目标。服务层须保证
同项目内、不得自引用、不得成环、层级上限两层（PRD §4.5「一个主要目标及多个子目标」）。

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

  stage_id                UUID                    FK -\> Stage，可空

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

  deleted_at              TIMESTAMPTZ             NULL；软删除标记
  -----------------------------------------------------------------------

`milestone_id` 可空，支持不设置阶段的简单项目。`stage_id` 为 V1.1
补列，允许任务直挂阶段（不设里程碑）；两者同时存在时，阶段须与里程碑所属阶段一致（服务层校验）。

`deleted_at` 为 V1.1 补列：**全库唯一需要软删除的表**。任务承载执行历史
（`completed_at`），删除后行保留但不再参与任何查询与计数。其余对象按 §8 分层处理。

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

#### ContentTag

用户私有的标签词表（PRD §6.3）。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| name | VARCHAR(60) | NOT NULL |
| color | VARCHAR(20) | NULL，仅展示用 |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：`UNIQUE(owner_id, name)` —— 标签是用户私有词表，同名必须合并而不是并存；标签不跨用户共享。

#### LearningContentTag

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| content_id | UUID | PK/FK → LearningContent |
| tag_id | UUID | PK/FK → ContentTag |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 复合主键 `(content_id, tag_id)`。
- 内容与标签必须属于同一 `owner`（服务层校验，见 §6.1）。
- 标签只用于内容：题目↔知识点、成果↔技能点的多对多分别由 `QuestionKnowledgePoint` / `ArtifactContentRelation` 承担，**不共用本表**。PRD §6.3 的三句话是三种不同关系，合并成一张通用「打标表」会让「标签」与「知识点」两个概念混淆。

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

#### ReviewPolicy

复习策略（PRD §8.5；TAD §3.6）。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| scope_type | VARCHAR(20) | `global` / `project` / `content` |
| project_id | UUID | FK → LearningProject，可空 |
| content_id | UUID | FK → LearningContent，可空 |
| content_type | VARCHAR(30) | 可空；仅在 global 作用域按内容类型细分 |
| enabled | BOOLEAN | NOT NULL |
| algorithm_type | VARCHAR(40) | NOT NULL |
| policy_config | JSONB | NOT NULL DEFAULT '{}' |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- 作用域一致性（DB CHECK 可表达）：`global` → `project_id`、`content_id` 均为空；`project` → `project_id` 非空且 `content_id` 为空；`content` → `content_id` 非空且 `project_id` 为空。
- 同一作用域只允许一条策略：`(owner_id, scope_type, project_id, content_id)` 唯一，按作用域拆成部分唯一索引（见 §7）。
- 优先级 `content` > `project` > `global`，由服务层解析后写入 `ReviewItem` 的调度参数；`ReviewItem` 不冗余存作用域来源。
- 首版不向普通用户暴露复杂 FSRS 参数（PRD §8.5）：`policy_config` 由服务端按 `algorithm_type` 写固定结构，用户侧只操作 `enabled` 与「暂停 / 单独改期」。

#### ReviewRescheduleHistory

复习延期历史（TAD §3.6：「不能只保存当前下一次日期而丢失延期历史」）。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| review_item_id | UUID | FK → ReviewItem，NOT NULL |
| original_next_review_at | TIMESTAMPTZ | NOT NULL |
| suggested_next_review_at | TIMESTAMPTZ | NULL |
| confirmed_next_review_at | TIMESTAMPTZ | NULL |
| postpone_reason | TEXT | NULL |
| operation_source | VARCHAR(20) | `user` / `ai` / `system` |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 只追加、不更新。
- `ReviewItem.next_review_at` 是当前快照，本表是它的变更历史；两者不一致时以本表最新一行为准，必要时可重放修正。
- `operation_source = ai` 的延期必须能关联到一条 `ChangeProposal`（服务层保证），与「AI 建议须用户确认」一致。

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

### 5.6 项目调度配置与业务事件

本节三张表服务于 M02 的项目级配置与可追溯性，都是 V1.2 新增。

#### ProjectPriorityOverride

临时标记（PRD §5.3）：本周重点、冲刺项目、保持最低投入、降低投入、暂停投入。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，NOT NULL |
| marker_type | VARCHAR(30) | `this_week_focus` / `sprint` / `minimum_commitment` / `reduced_commitment` / `paused_commitment` |
| effective_from | TIMESTAMPTZ | NOT NULL |
| effective_until | TIMESTAMPTZ | NULL = 持续生效直到取消 |
| status | VARCHAR(20) | `active` / `expired` / `cancelled` |
| note | TEXT | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- `effective_until IS NULL OR effective_until > effective_from`。
- 同一项目同一 `marker_type` 同时只允许一条 `active`（部分唯一索引，见 §7）；不同 `marker_type` 可并存，冲突由服务层提示。
- 到期失效**按 `effective_until` 实时判定**，后台任务置 `expired` 只是把状态追平；不允许「任务没跑 → 标记还生效」。
- 历史保留：取消置 `cancelled`，不物理删除（PRD §5.3「历史标记保留」）。
- 冲突（如 `paused_commitment` 与投入保障规则同时存在）必须提示用户调整，**不得静默覆盖**（PRD §5.3）。
- 标记只影响调度建议，不得覆盖用户设置的基础 `importance_level`（PRD §5.2）。

#### ProjectCommitmentRule

投入保障规则（PRD §5.4）：用户可加多条，独立统计。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，NOT NULL |
| period_type | VARCHAR(20) | `daily` / `weekly` / `monthly` |
| metric_type | VARCHAR(20) | `learning_days` / `learning_minutes` |
| target_value | INTEGER | CHECK > 0 |
| effective_from | DATE | NOT NULL |
| effective_until | DATE | NULL |
| enabled | BOOLEAN | NOT NULL DEFAULT true |
| note | TEXT | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- `metric_type = learning_days` 时目标值必须落在周期自然上界内：`daily ≤ 1`、`weekly ≤ 7`、`monthly ≤ 31`（DB CHECK 可表达）。`learning_minutes` 不设上界。
- `effective_until IS NULL OR effective_until >= effective_from`。
- **多条规则分别判断**，不设唯一约束（PRD §5.4「规则独立统计，多条规则必须分别判断」）。
- **不落达标结果表**：周期内已投入与未达标风险按 `LearningSession` 实时聚合（§1.2 原则 8）。「有效学习日」不能由固定分钟数机械判定，需结合活动内容并允许用户纠正（PRD §5.4）——该判定算法 v1 不实现，见 §12 第 11 项。
- 系统不得擅自删除、停用或改值用户规则（PRD §5.4）；`enabled` 只由用户操作改变。

#### ProjectEvent

项目级业务事件（TAD §3.1「项目状态变更应保留历史事件或审计记录」）。

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，NOT NULL，ON DELETE CASCADE |
| event_type | VARCHAR(40) | 见下方枚举 |
| from_value / to_value | JSONB | 变更前后快照，可空 |
| reason | TEXT | NULL |
| actor | VARCHAR(20) | `user` / `system` / `ai` |
| occurred_at | TIMESTAMPTZ | NOT NULL |

`event_type` 建议值：`created` / `status_changed` / `importance_changed` / `deadline_changed` / `module_toggled` / `priority_override_changed` / `commitment_rule_changed` / `archived` / `reopened` / `deleted`。

约束：

- 追加写，不更新不删除（项目 `purge` 时随外键级联删除）。
- `actor = ai` 的事件必须能追溯到产生它的 `ChangeProposal`，否则无法解释「谁改的」（服务层保证）。
- **与审计表的分工必须分清**：本表是**业务可追溯**记录，项目没了它就该没；审计事件属独立审计表（ERD 尚未设计，见 §12 第 12 项），审计表**不得有指向业务表的外键**——否则 `purge` 会连带删掉审计，审计形同虚设（ADR-006 §5 的结论）。

### 5.7 题库与练习

V1.1 完全没有本域的表。V1.2 按 PRD §9 与 TAD §3.7 建立 11 张表，先写三条贯穿全域的硬约束：

1. **未 `approved` 的题目不得进入任何正式掌握统计**（PRD §9.4）——统计口径一律先过滤 `review_status = 'approved'`。
2. **判分来源必须可区分**（PRD §9.3）——`PracticeAnswer.graded_by` 承载「谁判的」，AI 评分只能是 `ai_assisted`。
3. **原图、OCR 原文、用户修订三者分开保存**（PRD §9.5）——`QuestionAsset.ocr_text` 存原始识别，修订后的题干进 `QuestionVersion.stem`，互不覆盖。

#### Question

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| question_type | VARCHAR(30) | `single_choice` / `multiple_choice` / `true_false` / `fill_blank` / `short_answer` / `calculation` / `programming` / `essay` / `translation` |
| source | VARCHAR(20) | `user_created` / `ai_generated` / `imported` / `image_ocr` |
| review_status | VARCHAR(20) | `draft` / `pending_review` / `approved` / `rejected` |
| difficulty | SMALLINT | NULL，1–5 |
| archived_at | TIMESTAMPTZ | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- 题型用可扩展标识，**不为每种题型建表**；答案结构由 `QuestionVersion.answer_payload` + 题型 JSON Schema 约束（TAD §3.7）。
- 当前版本取该题 `MAX(version_no)`，**不设 `current_version_id`**——否则 `Question ↔ QuestionVersion` 形成循环外键，插入需要延迟约束。
- `archived_at` 非空即退出题库列表与组卷，但历史作答仍可回溯（不物理删除）。

#### QuestionVersion

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| question_id | UUID | FK → Question，NOT NULL |
| version_no | INTEGER | CHECK ≥ 1；`UNIQUE(question_id, version_no)` |
| stem | TEXT | NOT NULL |
| answer_payload | JSONB | NOT NULL，按题型 JSON Schema 校验 |
| answer_schema_version | VARCHAR(20) | NOT NULL |
| analysis | TEXT | NULL |
| created_by | VARCHAR(20) | `user` / `ai` |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 只追加：修订题目 = 新增版本，**不覆盖旧版本**。错题与历史作答必须能指回当时的题干。
- `answer_schema_version` 必填：题型演进后旧数据仍可解释；缺它就只能靠猜。
- AI 生成的版本在 `Question.review_status != 'approved'` 前只能用于临时练习（PRD §9.4）。

#### QuestionOption

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| question_version_id | UUID | FK → QuestionVersion，NOT NULL |
| option_key | VARCHAR(10) | NOT NULL |
| content | TEXT | NOT NULL |
| is_correct | BOOLEAN | NOT NULL DEFAULT false |
| sequence_no | SMALLINT | NOT NULL |

约束：

- `UNIQUE(question_version_id, option_key)`。
- 选项挂在**版本**而不是题目上：改选项即产生新版本，否则历史作答里的「B」会指向不同内容。
- 只有选择题类需要选项，其他题型该表无行；题型与选项的一致性由服务层按 schema 校验，不在 DB 硬绑。

#### QuestionAsset

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| question_id | UUID | FK → Question，NOT NULL |
| question_version_id | UUID | FK → QuestionVersion，可空 |
| asset_role | VARCHAR(20) | `original_image` / `compressed_image` / `figure` |
| storage_key | TEXT | NOT NULL |
| mime_type | VARCHAR(120) | NULL |
| size_bytes | BIGINT | CHECK ≥ 0 |
| ocr_status | VARCHAR(20) | `pending` / `succeeded` / `failed`，可空 |
| ocr_text | TEXT | NULL，OCR 原始文本 |
| ocr_confidence | NUMERIC(4,3) | NULL，0–1 |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- **OCR 失败不丢题**：`ocr_status = failed` 时仍允许仅凭原图保存与练习（PRD §9.5）。
- `question_version_id` 为空表示题目级素材（尚未归属某次修订）；非空表示该版本专用图。
- 与 `LearningEvidence` 保持同一形态：二进制在对象存储，DB 只存 `storage_key` 与元数据。首版**不建**统一 `stored_assets` 登记表，见 §12 第 13 项。

#### QuestionKnowledgePoint

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| question_id | UUID | PK/FK → Question |
| content_id | UUID | PK/FK → LearningContent |
| is_primary | BOOLEAN | NOT NULL DEFAULT false |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 复合主键 `(question_id, content_id)`；题目可关联多个知识点（PRD §6.3）。
- 题目与知识点必须同 `owner`（§6.1）。
- 建议每题至少一个 `is_primary`，由服务层保证：「按知识点专项练习」需要主知识点归集，缺失会让该模式退化。

#### QuestionSet

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，可空 |
| title | VARCHAR(160) | NOT NULL |
| description | TEXT | NULL |
| set_type | VARCHAR(20) | `manual` / `auto` / `wrong_book` |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：`wrong_book` 是系统按 `WrongQuestionState` 动态生成的题组，成员**不落** `QuestionSetItem`——否则错题状态一变就要重算整表。

#### QuestionSetItem

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| question_set_id | UUID | PK/FK → QuestionSet |
| question_id | UUID | PK/FK → Question |
| sequence_no | SMALLINT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：复合主键 `(question_set_id, question_id)`；`set_type = wrong_book` 的题组不得有本表行（服务层保证）。

#### PracticeSession

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，可空 |
| mode | VARCHAR(20) | `knowledge_point` / `random` / `wrong_question` / `custom` / `question_set` / `timed` |
| content_id | UUID | FK → LearningContent，可空（专项练习） |
| question_set_id | UUID | FK → QuestionSet，可空 |
| time_limit_seconds | INTEGER | CHECK > 0，可空 |
| status | VARCHAR(20) | `in_progress` / `completed` / `abandoned` |
| started_at | TIMESTAMPTZ | NOT NULL |
| finished_at | TIMESTAMPTZ | NULL |
| total_questions | INTEGER | CHECK ≥ 0 |
| answered_count | INTEGER | CHECK ≥ 0 |
| correct_count | INTEGER | CHECK ≥ 0 |
| learning_session_id | UUID | FK → LearningSession，可空 |

约束：

- `mode = knowledge_point` → `content_id` 非空；`mode IN (question_set, wrong_question)` → `question_set_id` 非空；`random` / `custom` 两者可空（前两条 DB CHECK 可表达）。
- `status = completed` → `finished_at` 非空。
- 三个计数是**可重建的冗余快照**（由 `PracticeAnswer` 聚合），写答案时同事务更新；列表页不聚合，避免 N+1。
- `learning_session_id` 用于把练习计入学习时长与投入。为空表示纯练习、不计投入——否则与 M05 的实际活动重复计量（ERD §3.3 原则）。

#### PracticeAnswer

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| practice_session_id | UUID | FK → PracticeSession，NOT NULL |
| question_id | UUID | FK → Question，NOT NULL |
| question_version_id | UUID | FK → QuestionVersion，NOT NULL |
| sequence_no | SMALLINT | NOT NULL |
| user_answer | JSONB | NULL |
| score | NUMERIC(6,2) | NULL |
| max_score | NUMERIC(6,2) | NULL |
| is_correct | BOOLEAN | NULL（主观题可空） |
| graded_by | VARCHAR(20) | `auto` / `self` / `ai_assisted` / `user_corrected` |
| ai_feedback | TEXT | NULL |
| duration_seconds | INTEGER | CHECK ≥ 0 |
| answered_at | TIMESTAMPTZ | NOT NULL |

约束：

- `UNIQUE(practice_session_id, question_id)`：一次练习内每题一行，重答是更新而不是追加。
- **`graded_by` 必须落库**（PRD §9.3）：只有 `score` 时前端无法区分「AI 给的 3 分」与「我给的 3 分」，「AI 评分必须标明为辅助结果」就落空。
- 用户改判后置 `user_corrected`，**保留 `ai_feedback` 供对照**，不覆盖 AI 原始意见。
- `question_version_id` 锁定作答时的版本，题目后续修订不影响历史判分依据。

#### WrongQuestionState

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，可空 |
| question_id | UUID | FK → Question，NOT NULL |
| wrong_count | INTEGER | NOT NULL DEFAULT 0，CHECK ≥ 0 |
| consecutive_correct | INTEGER | NOT NULL DEFAULT 0，CHECK ≥ 0 |
| status | VARCHAR(20) | `active` / `mastered` / `ignored` |
| last_wrong_at | TIMESTAMPTZ | NULL |
| mastered_at | TIMESTAMPTZ | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- 同一 owner + project + question 只允许一行。`project_id` 可空会让 SQL 唯一约束失效（NULL ≠ NULL），必须用**两条部分唯一索引**表达（见 §7）。
- 「已掌握」是可被推翻的用户判断（PRD §9.2 错题重练）：`mastered` 仍可回到 `active`，且不清空历史计数。
- 本表只存错题状态，不复刻题目内容；错题本视图 join `Question` 与最新 `QuestionVersion`。

#### QuestionReviewHistory

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| question_id | UUID | FK → Question，NOT NULL |
| question_version_id | UUID | FK → QuestionVersion，可空 |
| from_status | VARCHAR(20) | 可空（首次提交审核时为空） |
| to_status | VARCHAR(20) | NOT NULL |
| actor | VARCHAR(20) | `user` / `ai` |
| reason | TEXT | NULL |
| reviewed_at | TIMESTAMPTZ | NOT NULL |

约束：只追加。每次 `approved` 都必须留痕——AI 生成题进入正式题库要「用户确认」，这张表就是那条产品规则的最小证据（PRD §9.4）。

### 5.8 实践成果档案

#### Artifact

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| project_id | UUID | FK → LearningProject，NOT NULL |
| title | VARCHAR(200) | NOT NULL |
| description | TEXT | NULL |
| artifact_type | VARCHAR(30) | `code_project` / `running_result` / `design_work` / `writing` / `audio` / `certificate` / `other` |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- **成果必须归属项目**（`project_id` NOT NULL）：PRD §10 把成果放在实践语境，TAD §3.8 同样只给了 `project_id`。若将来需要「无项目成果」，要改为可空并新增归属枚举，属模型变更而非补列，见 §12 第 14 项。
- `artifact_type` 由 V1.2 首次给出（TAD 只有字段名、没有枚举），新增类型必须同步前端筛选与统计口径。
- **不设 `external_url` 单值字段**：外部链接统一走 `ArtifactLink`，避免「一个链接在 artifact 表、另一个在 links 表」的两处真相。

#### ArtifactAsset

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| artifact_id | UUID | FK → Artifact，NOT NULL |
| asset_role | VARCHAR(20) | `screenshot` / `document` / `audio` / `video` / `archive` / `other` |
| storage_key | TEXT | NOT NULL |
| file_name | VARCHAR(255) | NULL |
| mime_type | VARCHAR(120) | NULL |
| size_bytes | BIGINT | CHECK ≥ 0 |
| sequence_no | SMALLINT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 一个成果可挂多个附件（PRD §10.2「运行截图、图片和文档」是复数）。TAD 的单个 `asset_id` 不足，V1.2 明确拆表。
- 附件**默认可选**，不强制每次实践都上传（PRD §10.2）。
- 删除成果时附件行随之删除；对象存储清理走后台任务，v1 不做引用计数（§8.1 第 10 步）。

#### ArtifactLink

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| artifact_id | UUID | FK → Artifact，NOT NULL |
| link_type | VARCHAR(20) | `git_repo` / `commit` / `pull_request` / `demo` / `other` |
| url | TEXT | NOT NULL |
| title | VARCHAR(200) | NULL |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：不做 URL 可达性校验——离线环境与私有仓库都会误判；只由服务层校验 scheme 为 `http` / `https`。

#### ArtifactContentRelation

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| artifact_id | UUID | PK/FK → Artifact |
| content_id | UUID | PK/FK → LearningContent |
| contribution | VARCHAR(20) | `primary` / `supporting` |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- 复合主键 `(artifact_id, content_id)`；一个成果关联多个技能点（PRD §6.3）。
- 成果与内容必须同 `owner`（§6.1）。**不要求**内容与成果同属一个项目——跨项目复用技能点是 PRD §6.4 的既定行为。
- `contribution` 与 `GoalStage.contribution` 同语义同取值，不另造枚举。

### 5.9 生活记录与数据授权

三条前置约束先于表定义：

1. **生活记录是可选模块**（PRD §12.1）：本域表**不得被核心学习流程外键引用**，本域可有指向 `User` 的外键。这条是硬边界，违反就会让「不开启生活模块」变成「数据库不完整」。
2. **建模路线选 A（四张类型表）**，不用「一张主表 + JSONB」。理由：四类记录的字段差异是「四种不同的东西」而非同一实体的可选属性；PRD §13.1 的生活分析本就按维度查询，不需要混排；JSONB 化会把「体重必须 > 0」这类约束退化为应用层校验。
3. **默认拒绝**：`AIDataPermission` 无有效行即不允许 AI 读取（PRD §12.4）。

#### SleepRecord

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| record_date | DATE | NOT NULL |
| sleep_start_at | TIMESTAMPTZ | NOT NULL |
| wake_at | TIMESTAMPTZ | NOT NULL |
| duration_minutes | INTEGER | CHECK > 0 |
| note | TEXT | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- `wake_at > sleep_start_at`（DB CHECK 可表达）。
- 跨午夜由 TIMESTAMPTZ 天然表达；`record_date` 表示「这次睡眠归属哪一天」，口径固定为**起床日**，由服务层写入。口径不固定，统计就会各写各的。
- `duration_minutes` 与两端差不做强绑定：用户常只记得时长，允许手工填写。

#### ExerciseRecord

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| record_date | DATE | NOT NULL |
| exercise_type | VARCHAR(40) | NOT NULL |
| duration_minutes | INTEGER | CHECK > 0 |
| distance_meters | INTEGER | CHECK ≥ 0，可空 |
| note | TEXT | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：`exercise_type` 用自由文本而非枚举——PRD §12.2 只给「运动类型」，枚举化会把「拉伸」「爬楼梯」这类正常输入挡在门外。

#### MealRecord

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| record_date | DATE | NOT NULL |
| meal_type | VARCHAR(20) | `breakfast` / `lunch` / `dinner` / `snack` |
| content | TEXT | NOT NULL |
| note | TEXT | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：`content` 是「吃了什么」（PRD §12.2「饮食内容」），`note` 是备注，两列语义不重叠。

#### BodyMeasurement

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| measured_at | TIMESTAMPTZ | NOT NULL |
| weight_kg | NUMERIC(5,2) | CHECK > 0 AND < 500 |
| note | TEXT | NULL |
| created_at | TIMESTAMPTZ | NOT NULL |

约束：

- PRD §12.2 只列「体重等」，没有列举其他指标。V1.2 **只落体重**，不为「等」预留 JSONB——体脂率、围度等待 PRD 明确后加列（ADR-007 的最小承诺原则）。
- 一天可多次称重，不设按日唯一约束；展示「每日最后一次」由查询决定。

#### AIDataPermission

| 字段 | 类型建议 | 约束/说明 |
|---|---|---|
| id | UUID | PK |
| owner_id | UUID | FK → User，NOT NULL |
| data_category | VARCHAR(20) | `sleep` / `exercise` / `meal` / `body` / `all` |
| granted | BOOLEAN | NOT NULL |
| purpose | TEXT | NOT NULL |
| granted_at | TIMESTAMPTZ | NOT NULL |
| revoked_at | TIMESTAMPTZ | NULL |
| created_at / updated_at | TIMESTAMPTZ | NOT NULL |

约束：

- 同一 `(owner_id, data_category)` 只允许一条有效行（`revoked_at IS NULL`），用部分唯一索引表达（见 §7）。
- **撤销不删行**：置 `revoked_at` 后保留，否则无法回答「什么时候授权过、用途是什么」（PRD §12.4「授权状态和用途应可查看」）。
- `purpose` 必填：无用途说明的授权无法审计。
- `all` 是总开关，但**不得绕过**类别级校验与数据最小化（PRD §12.4）；总开关关闭时，类别级授权一律不生效。
- 读取授权只走本表，不允许在业务代码里硬编码「用户开了生活模块即默认授权」。

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

**V1.2 新增实体关系**（6 处缺口补齐后）

| 关系 | 基数 | 说明 |
|---|---|---|
| LearningProject → ProjectPriorityOverride | 1:N | 同类标记同时只有一条 `active`，异类可并存 |
| LearningProject → ProjectCommitmentRule | 1:N | 多条保障规则独立统计 |
| LearningProject → ProjectEvent | 1:N | 项目级业务事件，随项目级联删除 |
| User → ContentTag | 1:N | 标签是用户私有词表 |
| LearningContent ↔ ContentTag | M:N | 通过 `LearningContentTag` |
| User / Project / Content → ReviewPolicy | 1:N | 按 `scope_type` 三选一，同作用域唯一 |
| ReviewItem → ReviewRescheduleHistory | 1:N | 延期历史只追加 |
| User → Question | 1:N | 题目归用户，可跨项目复用 |
| Question → QuestionVersion | 1:N | 历史版本不可覆盖 |
| QuestionVersion → QuestionOption | 1:N | 选项随版本走 |
| Question → QuestionAsset | 1:N | 原图 / 压缩图 / 插图 |
| Question ↔ LearningContent | M:N | 通过 `QuestionKnowledgePoint` |
| QuestionSet → QuestionSetItem | 1:N | `wrong_book` 类型无成员行 |
| PracticeSession → PracticeAnswer | 1:N | 一次练习多次作答 |
| Question → WrongQuestionState | 1:N | 按 owner + project 区分错题本 |
| LearningProject → Artifact | 1:N | 成果必须归属项目 |
| Artifact → ArtifactAsset / ArtifactLink | 1:N | 附件与链接都是复数 |
| Artifact ↔ LearningContent | M:N | 通过 `ArtifactContentRelation` |
| User → SleepRecord / ExerciseRecord / MealRecord / BodyMeasurement | 1:N | 四类生活记录各自成表 |
| User → AIDataPermission | 1:N | 按类别授权，撤销保留行 |

生活记录与核心学习域**没有任何外键连线**，这是刻意的（§5.9 前置约束 1）。

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
11. 标签与内容必须同 owner；标签名在用户内唯一。
12. 题目与其知识点关联、题组、练习会话、错题状态必须同 owner；练习会话的 `project_id`（非空时）也必须同 owner。
13. `PracticeAnswer.question_version` 必须属于该 `question`，防止作答记录挂到其他题目的版本上。
14. `WrongQuestionState` 的 `(owner, project, question)` 唯一性必须用**两条部分唯一索引**表达（`project_id IS NULL` / `IS NOT NULL`），单条 `UNIQUE` 在可空列上会失效。
15. 成果与关联内容必须同 owner；二者**不要求**同项目（跨项目复用技能点是 PRD §6.4 的既定行为）。
16. `ReviewPolicy` 的 `scope_type` 与 `project_id` / `content_id` 空值组合必须自洽，解析优先级固定为 `content` > `project` > `global`。
17. `ProjectEvent.actor = ai` 与 `ReviewRescheduleHistory.operation_source = ai` 的事件必须能追溯到 `ChangeProposal`。
18. **反向约束**：任何其他模块的表不得有指向 `sleep_records` / `exercise_records` / `meal_records` / `body_measurements` 的外键；跨域读取只能经 `AIDataPermission` 授权的分析接口（PRD §12.4）。


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

  LearningTask               `(project_id, stage_id)`                      阶段下任务与解挂范围

  LearningTask               `(project_id, milestone_id)`                  里程碑下任务与影响面计数

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

**V1.1 补充**：所有 `LearningTask` 索引建议加 `WHERE deleted_at IS NULL`
构成部分索引，与「软删行不参与列表查询、依赖环路检测、影响面计数、结构解挂」的契约规则一致（ADR-007 §6）。新增的
`(project_id, stage_id)` 与 `(project_id, milestone_id)` 服务于两个高频场景：按阶段 / 里程碑筛选任务，以及删除上层结构时计算影响面与解挂范围。

### 7.1 V1.2 新增表的索引

| 表 | 索引 | 用途 |
|---|---|---|
| ProjectPriorityOverride | `(project_id, status, effective_from DESC)` | 当前生效标记 |
| ProjectPriorityOverride | `(project_id, marker_type)` UNIQUE WHERE `status = 'active'` | 同类标记只允许一条生效 |
| ProjectCommitmentRule | `(project_id, enabled)` | 项目保障规则列表 |
| ProjectEvent | `(project_id, occurred_at DESC)` | 项目历史时间线 |
| ContentTag | `(owner_id, name)` UNIQUE | 标签去重 |
| LearningContentTag | `(tag_id, content_id)` | 按标签反查内容 |
| ReviewPolicy | `(owner_id, scope_type)` + 作用域部分唯一索引 | 策略解析（content > project > global） |
| ReviewRescheduleHistory | `(review_item_id, created_at DESC)` | 单条队列的延期历史 |
| Question | `(owner_id, review_status, updated_at DESC)` | 题库列表与待审队列 |
| Question | `(owner_id, question_type)` | 按题型筛选 |
| QuestionVersion | `(question_id, version_no DESC)` UNIQUE | 取当前版本 |
| QuestionOption | `(question_version_id, option_key)` UNIQUE | 选项读取 |
| QuestionAsset | `(question_id, asset_role)` | 题目图片读取 |
| QuestionKnowledgePoint | `(content_id, question_id)` | 按知识点组卷 |
| QuestionSet | `(owner_id, set_type)` | 题组列表 |
| QuestionSetItem | `(question_set_id, sequence_no)` | 题组顺序 |
| PracticeSession | `(owner_id, started_at DESC)` | 练习历史 |
| PracticeSession | `(project_id, started_at DESC)` WHERE `project_id IS NOT NULL` | 项目练习统计 |
| PracticeAnswer | `(practice_session_id, sequence_no)` UNIQUE | 作答读取 |
| WrongQuestionState | `(owner_id, status, last_wrong_at DESC)` | 错题本列表 |
| WrongQuestionState | `(owner_id, project_id, question_id)` UNIQUE ×2（NULL / NOT NULL 各一条） | 防止重复错题行 |
| QuestionReviewHistory | `(question_id, reviewed_at DESC)` | 审核留痕 |
| Artifact | `(project_id, created_at DESC)` | 项目成果列表 |
| Artifact | `(owner_id, artifact_type)` | 按类型筛选 |
| ArtifactAsset | `(artifact_id, sequence_no)` | 附件顺序 |
| ArtifactLink | `(artifact_id)` | 链接读取 |
| ArtifactContentRelation | `(content_id, artifact_id)` | 技能点反查成果 |
| SleepRecord / ExerciseRecord / MealRecord | `(owner_id, record_date DESC)` ×3 | 按日查询 |
| BodyMeasurement | `(owner_id, measured_at DESC)` | 体重趋势 |
| AIDataPermission | `(owner_id, data_category)` UNIQUE WHERE `revoked_at IS NULL` | 单类别唯一有效授权 |

生活记录四表的索引不与任何学习域表共用条件，避免「顺手 join」把可选模块拖进核心查询路径。

------------------------------------------------------------------------

## 8. 删除、归档与数据保留策略

  对象                       默认操作               说明
  -------------------------- ---------------------- ----------------------------------
  LearningProject            归档 / draft 物理删除  非 draft 只能归档或走受控 purge
  Roadmap                    标记 superseded /     已确认路线不可删除
                             draft 物理删除
  Stage / Milestone          仅 draft 路线可删     已确认结构不提供删除，用状态动作表达
  LearningTask               软删除（deleted_at）  保留执行历史引用，不参与查询与计数
  LearningContent            归档                   若被项目引用，不允许直接物理删除
  ProjectContent             解除关联/归档          不删除共享内容
  LearningSession            保留                   更正需保留审计历史
  LearningEvidence           按引用和用户授权管理   删除附件需检查引用
  MasteryAssessment          保留历史               不覆盖历史评价
  ReviewRecord               保留历史               不覆盖历史复习
  ScheduledActivity          取消/删除计划          不影响 LearningSession
  ChangeProposal             保留状态历史           便于追踪 AI 建议

**分层原则（V1.1 新增，依据 ADR-006）**：删除语义由「该对象**是否已参与执行**」决定，不由对象类型决定。

| 层 | 判据 | 对象 | 语义 | 理由 |
|---|---|---|---|---|
| L1 编辑区 | 容器未生效（`draft`） | draft 项目 / draft 路线及其下 stage、milestone | **物理删除** | 从未参与执行，无历史可损 |
| L2 已生效结构 | 容器已 confirm | active / superseded 路线内的 stage、milestone | **不提供删除** | 结构是承诺；调整意图用状态动作（skip）表达 |
| L3 原子执行单元 | 自身承载 `completed_at` | LearningTask | **软删除**（唯一软删对象） | 执行历史须保留可引用 |
| L4 成果 | 达成事实 | achieved 目标、completed / archived 项目 | **禁止删除** | 删除后不可追溯；用状态 / 归档替代 |

「任务永不随上层结构消失」的准确表述是「不随**上层结构**（路线 / 阶段 / 里程碑）消失」：项目是任务的归属根（`project_id` NOT NULL），项目级删除会连带删任务。

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

**v1 落地范围（V1.1 新增）**：上述 10 步中，v1 实现第 1、2、4、5、6 步；
第 3 步的影响面展示降级为删除接口返回 `409` 并携带 `{ impact: { milestones, tasks } }` 计数；
第 7、8、10 步在 v1 无对象可处理（学习记录属 P1 未设计，派生调度与附件清理尚未设计）；
第 9 步的审计事件暂无存储表，以结构化 WARN 日志落地。
删除入口分两条：`status=draft` 走普通 `DELETE`，其余状态走需重输项目名的受控 `purge`。详见 ADR-006。

------------------------------------------------------------------------

## 9. 状态枚举建议

枚举值应集中管理，避免前后端自由拼写。

  -----------------------------------------------------------------------
  对象                                建议状态
  ----------------------------------- -----------------------------------
  LearningProject                     draft / active / paused /
                                      completed / archived / deleting

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

`LearningProject` 的 `draft` 为 V1.1 补入（PRD §4.3 五态 + ERD `deleting` 合并）。
v1 的常规流程只产生前五个值；`deleting` 仅由受控 `purge` 事务内部写入、随事务结束消失，
任何常规写操作置 `deleting` 必须被拒绝，否则会产生无法恢复的僵尸项目。

**V1.2 新增枚举**

| 对象.字段 | 取值 |
|---|---|
| ProjectPriorityOverride.marker_type | `this_week_focus` / `sprint` / `minimum_commitment` / `reduced_commitment` / `paused_commitment` |
| ProjectPriorityOverride.status | `active` / `expired` / `cancelled` |
| ProjectCommitmentRule.period_type | `daily` / `weekly` / `monthly` |
| ProjectCommitmentRule.metric_type | `learning_days` / `learning_minutes` |
| ProjectEvent.event_type | `created` / `status_changed` / `importance_changed` / `deadline_changed` / `module_toggled` / `priority_override_changed` / `commitment_rule_changed` / `archived` / `reopened` / `deleted` |
| ProjectEvent.actor | `user` / `system` / `ai` |
| ReviewPolicy.scope_type | `global` / `project` / `content` |
| ReviewRescheduleHistory.operation_source | `user` / `ai` / `system` |
| Question.question_type | `single_choice` / `multiple_choice` / `true_false` / `fill_blank` / `short_answer` / `calculation` / `programming` / `essay` / `translation` |
| Question.source | `user_created` / `ai_generated` / `imported` / `image_ocr` |
| Question.review_status | `draft` / `pending_review` / `approved` / `rejected` |
| QuestionAsset.ocr_status | `pending` / `succeeded` / `failed` |
| QuestionSet.set_type | `manual` / `auto` / `wrong_book` |
| PracticeSession.mode | `knowledge_point` / `random` / `wrong_question` / `custom` / `question_set` / `timed` |
| PracticeSession.status | `in_progress` / `completed` / `abandoned` |
| PracticeAnswer.graded_by | `auto` / `self` / `ai_assisted` / `user_corrected` |
| WrongQuestionState.status | `active` / `mastered` / `ignored` |
| Artifact.artifact_type | `code_project` / `running_result` / `design_work` / `writing` / `audio` / `certificate` / `other` |
| ArtifactAsset.asset_role | `screenshot` / `document` / `audio` / `video` / `archive` / `other` |
| ArtifactLink.link_type | `git_repo` / `commit` / `pull_request` / `demo` / `other` |
| MealRecord.meal_type | `breakfast` / `lunch` / `dinner` / `snack` |
| AIDataPermission.data_category | `sleep` / `exercise` / `meal` / `body` / `all` |
| `contribution`（通用） | `primary` / `supporting`，`GoalStage` 与 `ArtifactContentRelation` 共用同一枚举，不各造一份 |

两点提醒：新增枚举必须在代码层集中定义一处，禁止前后端各自拼写；`ReviewItem.status`（队列状态）与 `WrongQuestionState.status`（错题处置）同名不同义，不要合并成一张枚举表。

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

**V1.2 新增应用服务**

| 应用服务 | 主要命令/查询 |
|---|---|
| ProjectConfigService | 设置 / 取消临时标记、维护投入保障规则、写入并查询项目事件 |
| TagService | 建标签、改名、合并标签、给内容打标与去标 |
| ReviewPolicyService | 解析作用域优先级、开关自动复习、登记延期历史 |
| QuestionBankService | 建题、修订版本、提交审核、确认或拒绝、归档、组卷 |
| PracticeService | 开始练习、提交作答、判分（含 `graded_by` 标记）、更新错题状态 |
| ArtifactService | 记录成果、挂附件与链接、关联技能点 |
| LifeRecordService | 四类生活数据的增删改查与统计 |
| DataPermissionService | 授予 / 撤销 AI 数据授权、校验读取资格 |

生活记录域**不对外提供跨域服务调用**：M11 分析要读生活数据，必须先经 `DataPermissionService` 校验，而不是直接查表——否则「默认拒绝」只是一句口号。

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
-   ProjectEvent（项目状态与配置变更留痕）

### P1：学习内容与实际记录

-   LearningContent
-   ContentRelation
-   ProjectContent
-   LearningSession
-   SessionContent
-   SessionProject
-   LearningEvidence
-   TaskSession
-   ContentTag / LearningContentTag（内容标签）

### P2：掌握度与复习

-   MasteryAssessment
-   OverallMasteryAssessment / OverallMastery
-   ReviewItem
-   ReviewRecord
-   ReviewShare
-   ReviewPolicy / ReviewRescheduleHistory
-   Question 及题库其余 10 张表（§5.7）
-   WrongQuestionState（错题状态与错题重练）
-   Artifact / ArtifactAsset / ArtifactLink / ArtifactContentRelation

### P3：日程与 AI 提案

-   DailyLearningGoal
-   ScheduledActivity
-   ChangeProposal
-   AI 计划建议与确认流程
-   调度算法及冲突处理
-   ProjectPriorityOverride / ProjectCommitmentRule（调度权重与投入保障）
-   SleepRecord / ExerciseRecord / MealRecord / BodyMeasurement / AIDataPermission

生活记录、健康数据、第三方设备接入和高级 AI
分析不应阻塞上述核心领域模型——V1.2 已为其建模（§5.9），但不进入 P0–P2 的表依赖链。

------------------------------------------------------------------------

## 12. 尚未锁定、实施前必须补充的设计项

本轮已经明确逻辑实体和主要关系，但以下事项不应在没有讨论的情况下擅自固化：

1.  **掌握等级 L0--L4 的最终定义和各内容类型判定量表。**
2.  **各题型的 `answer_payload` JSON Schema**：V1.2 已锁定题库 11 张表的实体与关系（§5.7），
    但 9 种题型的答案结构仍需逐个定义，否则 `answer_schema_version` 无处落地。
3.  **语言学习内容扩展模型**：词形、词性、释义、例句、发音、词族及语言专属复习策略。
4.  **学习活动计时规则**：暂停、跨午夜、手动补录、有效时长修正和审计。
5.  **项目删除时学习记录的处置**（契约决策 D-20，已移交 P1）：v1 的 `purge`
    只删规划域数据；`LearningSession` 设计时须决定「主要归属学习记录」是一并删除还是解挂保留为个人历史。
6.  **个人总体掌握算法**：证据权重、时间衰减、冲突处理、置信度校准。
7.  **复习共享的具体语义**：共享仅作为证据，还是允许用户将复习结果转化为目标项目的评价输入。
8.  **数据隐私与附件授权**：证据跨项目查看的权限、撤销机制及健康数据授权边界。
9.  **多时区策略**：用户时区变更后，历史 `DATE` 与时间戳如何解释。
10. **并发与审计**：关键状态迁移的乐观锁、幂等键、审计事件和事务边界。
11. **「有效学习日」判定算法**：PRD §5.4 要求结合活动内容评估并允许用户纠正，V1.2 只落了规则表（§5.6 `ProjectCommitmentRule`），未定义判定口径与阈值。
12. **审计事件表**：`ProjectEvent` 是业务可追溯（随项目级联删除），审计事件仍无表。设计审计表时**不得**有指向业务表的外键（§5.6 已说明原因）。
13. **统一附件登记表**：`LearningEvidence` / `QuestionAsset` / `ArtifactAsset` 三处各自持 `storage_key`，是否收敛为一张 `stored_assets`（含引用计数与清理策略）待定。
14. **成果是否允许无项目归属**：V1.2 定为 `project_id NOT NULL`；若要支持「个人成果档案」，需改为可空并引入归属枚举，属模型变更而非补列。
15. **生活记录的其他身体指标**：PRD 只列「体重等」，体脂率、围度、睡眠质量等未列举，暂不加列。
16. **学习内容的正文存储形态**：`LearningContent` 现有 `description TEXT`（摘要）与 `metadata JSONB`（类型扩展属性），二者都不适合放富文本 / LaTeX / 代码块正文。是新增 `body TEXT` + `body_format`，还是存对象存储并引用 `storage_key`，需在 M03 实现前定。

------------------------------------------------------------------------

## 13. 评审结论与下一步

当前模型可以作为 Learna
数据库设计的逻辑基线，但不建议直接根据本文一次性生成全部 migration。

推荐下一步顺序：

1.  对本文件的实体和关系做第二轮审查，尤其是学习活动、掌握度、复习和跨项目共享。
2.  评审 V1.2 新增的四个域（§5.6–5.9），并逐个定义题型 `answer_payload` 的 JSON Schema。
3.  确定 L0--L4 的评价规范。
4.  形成 API Contract，明确每个聚合的命令和查询。
5.  根据确定的技术栈生成 PostgreSQL DDL 和分阶段 SQLx migrations。
6.  为关键业务不变量编写领域测试和数据库约束测试。

**最终原则：先保证业务事实、归属关系和状态迁移正确，再优化表结构和查询性能。**
