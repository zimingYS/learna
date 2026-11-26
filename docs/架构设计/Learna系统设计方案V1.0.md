# Learna 系统设计方案

-   文档版本：V1.0（架构与领域设计草案）
-   依据：《Learna 产品需求规格说明书（PRD）V1.0》
-   状态：推荐设计，非最终数据库/API 定稿
-   目标：为 Rust Backend、Python AI Service、Vue
    前端和数据分析能力提供一致的系统边界

------------------------------------------------------------------------

# 1. 设计原则

1.  领域模型先于数据库表和 API URL。
2.  Project 是目标组织容器，不是强制固定流程。
3.  学习内容、项目上下文、学习活动和学习证据分离。
4.  用户事实、系统评价、AI 建议和用户确认结果分离。
5.  计划是建议或安排，学习活动才是实际发生的事实。
6.  复习队列是由学习证据和复习规则派生的调度对象，不应替代学习内容本身。
7.  AI 不拥有绕过用户确认修改核心业务事实的权限。
8.  模块可选、数据可追溯、历史可保留。
9.  首版简化算法，但保留升级路径。

------------------------------------------------------------------------

# 2. 系统上下文与逻辑架构

## 2.1 逻辑组件

``` text
Vue 3 Web Client
    |
    | HTTPS / JSON API
    v
Rust Backend (Axum)
    |
    +-- Authentication & Authorization
    +-- User / Project Management
    +-- Learning Content
    +-- Planning & Scheduling
    +-- Learning Sessions & Evidence
    +-- Mastery Evaluation
    +-- Review Management
    +-- Question Bank & Practice
    +-- Practice Portfolio
    +-- Life Tracking
    +-- Analytics
    +-- AI Gateway
    |
    +---- PostgreSQL
    +---- Object Storage (images / attachments)
    |
    +---- Python AI Service
             |
             +-- LLM Provider
             +-- OCR / document processing
             +-- analysis and recommendation workflows
```

Rust Backend 是业务事实和权限的权威来源。Python AI Service
提供模型推理、分析和生成能力，不直接成为核心业务数据的唯一写入者。

## 2.2 职责边界

### Vue 3

-   页面呈现、表单校验和交互状态。
-   今日页面、项目页面、题目练习和图表展示。
-   展示 AI 建议并收集用户确认。
-   不负责最终权限校验或掌握等级的权威计算。

### Rust Backend

-   身份认证、资源所有权和授权校验。
-   核心领域规则和数据持久化。
-   项目、阶段、任务、内容、学习活动和复习状态管理。
-   AI 请求编排、结果校验和确认流程。
-   对外提供稳定、版本化的 API。

### Python AI Service

-   学习计划和项目规划建议。
-   内容分类、知识点/技能点候选生成。
-   错题和学习证据分析。
-   题目候选生成与解析。
-   OCR/图像内容提取（若选择在该服务实现）。
-   学习趋势与深度分析。
-   不直接绕过 Rust Backend 修改用户确认的数据。

### PostgreSQL

保存业务实体、状态、关联、历史和分析所需的结构化数据。

### 对象存储

保存题目原图、学习附件、实践成果等二进制文件。数据库保存元数据和受控存储引用，不建议将大文件直接存入普通业务表。

------------------------------------------------------------------------

# 3. 领域模型

以下为逻辑实体建议，不代表最终一张实体对应一张数据库表。具体拆表应在 ERD
阶段完成。

## 3.1 用户与项目

### User

核心字段建议： - id - email / username - password_hash - status -
created_at - updated_at

### Project

核心字段建议： - id - user_id - name - description - project_type -
status - importance_level（I1---I5） -
target_completion_date（软截止） - fixed_deadline（固定截止，可选） -
timezone - theory_enabled - practice_enabled - created_at - updated_at -
archived_at（可选）

项目可以无日期、无阶段。项目状态变更应保留历史事件或审计记录。

### ProjectEvent / ProjectStatusHistory

建议记录状态、重要程度、阶段和关键配置变化，以支持历史追溯和分析。

## 3.2 阶段、目标、里程碑和任务

### ProjectGoal

建议区分主要目标和子目标： - id - project_id - parent_goal_id（可选） -
title - description - goal_type - target_date（可选） -
completion_status - source（user / ai） - confirmation_status

### Stage

-   id
-   project_id
-   name
-   description
-   sequence_order
-   status
-   planned_start_date
-   planned_end_date
-   actual_started_at
-   actual_completed_at
-   completion_condition（可选）
-   source（user / ai）
-   confirmation_status

Stage 的 sequence_order 只表示推荐顺序，不应构成不可绕过的强制状态机。

### Milestone

-   id
-   project_id
-   stage_id（可选）
-   title
-   description
-   target_date（可选）
-   completion_condition（可选）
-   status
-   source
-   confirmation_status

### Task

-   id
-   project_id
-   stage_id（可选）
-   milestone_id（可选）
-   title
-   description
-   task_type
-   priority
-   estimated_minutes（可选）
-   due_date（可选）
-   status
-   source（user / ai）
-   confirmation_status

### TaskDependency

首版仅实现简单前置依赖： - predecessor_task_id - successor_task_id -
dependency_type（首版可固定为 finish-to-start） - created_at

必须防止任务依赖环。复杂依赖图、自动关键路径和资源约束后续实现。

## 3.3 学习内容与项目上下文

建议将"内容本体"和"内容在某项目中的使用"分开。

### LearningContent

代表可复用的知识点或技能点： - id - owner_user_id - title -
description - content_type - parent_content_id（可选） -
difficulty（可选） - source - created_at - updated_at

### ProjectContent

表示某项目采用某项学习内容： - id - project_id - learning_content_id -
project_specific_objective - required_mastery_level（可选） - status -
sequence_order - created_at

同一 LearningContent 可以被多个 ProjectContent
引用。项目上下文的目标要求和完成状态独立。

### ContentTag / LearningContentTag

支持内容标签。内容树与标签是两种不同的组织方式：树表达结构，标签表达多维分类。

### 内容复用与掌握证据

建议保留用户层面的内容掌握证据，同时保留项目上下文评价： -
用户对内容的全局掌握估计。 - 某项目要求的掌握等级。 -
该项目下的学习和测试证据。

新项目规划时，AI 比较"已有掌握等级 + 证据充分程度 +
最近表现"与"新项目要求等级"，再决定跳过、诊断、回顾或重新学习。

不能将项目 A 的任务完成状态直接复制为项目 B 的任务完成状态。

## 3.4 学习活动与证据

### LearningSession

代表实际发生的一次学习活动： - id - user_id - primary_project_id -
started_at（可选） - ended_at（可选） - duration_minutes - occurred_at -
recording_method（manual / timer / imported） - summary - notes -
created_at - updated_at

### LearningSessionContent

一次学习活动可关联多个知识点或技能点： - learning_session_id -
learning_content_id - project_content_id（可选） - relation_type

### LearningSessionTask

可选关联任务。无预先任务也可以创建 LearningSession。

### LearningEvidence

保存可用于掌握评价的证据： - id - user_id - learning_content_id -
project_content_id（可选） - evidence_type（quiz / review / practice /
self_assessment / artifact / imported） - source_id - observed_at -
evidence_payload / structured_metrics - created_at

证据需要保留来源，避免将 AI 推断当作直接观测事实。

## 3.5 掌握评价

建议将评价记录设计为不可覆盖的历史事件。

### MasteryAssessment

-   id
-   user_id
-   learning_content_id
-   project_content_id（可选）
-   assessment_type（self / system / confirmed）
-   level（L0---L4）
-   confidence_level
-   rationale
-   evidence_snapshot / evidence references
-   assessed_at
-   created_at

### MasteryState

保存当前状态的便捷投影： - user_id - learning_content_id - self_level -
system_level - confirmed_level - disagreement_status -
evidence_confidence - updated_at

当前状态可由历史评价事件更新，但历史评价不应被覆盖删除。

首次评价：confirmed_level 初始采用用户自评；system_level
证据不足时为空或标为证据不足。

系统评价变化：生成建议，不直接覆盖 confirmed_level。用户确认后写入新的
confirmed 评价事件。

## 3.6 复习系统

### ReviewPolicy

定义默认或项目级复习规则： - scope（global / project / content） -
content_type - enabled - algorithm_type - policy_config - updated_at

### ReviewItem

表示一个可复习的内容队列项： - user_id - learning_content_id -
project_content_id（可选） - eligibility_status - next_review_at -
review_status - last_reviewed_at - suspended_at - archived_at

### ReviewSession / ReviewFeedback

记录每次复习活动、类型化反馈、表现证据和算法输入。

### ReviewRescheduleHistory

记录原到期时间、建议新时间、用户确认时间、延期原因（可选）和操作来源。

复习队列不能只保存"当前下一次日期"而丢失延期历史。

## 3.7 题库与练习

建议实体： - Question - QuestionVersion - QuestionOption -
QuestionAsset - QuestionKnowledgePoint - QuestionSet - PracticeSession -
PracticeAnswer - WrongQuestionState - QuestionReviewHistory

题目类型应使用可扩展类型标识。不同题型的答案结构应由类型化 DTO/JSON
Schema 约束，避免把所有题目硬编码成单一文本字段。

题目来源： - user_created - ai_generated - imported - image_ocr

AI 生成题目需要状态： - draft - pending_review - approved - rejected

未 approved 的题目不得进入正式掌握统计。

### 图片与 OCR

原图存对象存储，压缩副本用于常规访问。建议同时保存： -
original_asset_id - compressed_asset_id - OCR 文本及版本 -
用户修订后的题目内容 - OCR 状态和置信信息

用户校对内容与 OCR 原始结果应区分保存。

## 3.8 实践成果档案

### Artifact

-   id
-   user_id
-   project_id
-   title
-   description
-   artifact_type
-   external_url（可选）
-   asset_id（可选）
-   created_at

### ArtifactContentRelation

一个成果可以关联多个技能点。关键阶段可以要求成果附件或链接，但普通学习活动不强制上传。

## 3.9 今日计划与时间安排

建议区分三个概念：

1.  DailyLearningGoal：用户希望达到的每日学习时长。
2.  ScheduledActivity：系统建议或用户确认的安排。
3.  LearningSession：实际发生的学习活动。

### DailyLearningGoal

-   user_id
-   date
-   target_minutes
-   timezone
-   updated_at

### ScheduledActivity

-   id
-   user_id
-   project_id
-   task_id（可选）
-   review_item_id（可选）
-   scheduled_date
-   start_time（可选）
-   end_time（可选）
-   estimated_minutes
-   source（user / system / ai）
-   status
-   confirmed_at

计划与实际活动不可混为一谈。用户临时调整目标分钟数，不应自动重写历史学习活动。

## 3.10 项目调度配置

### ProjectPriorityOverride

保存临时标记： - project_id - marker_type - effective_from -
effective_until（可选） - status - created_by - created_at

### ProjectCommitmentRule

-   project_id
-   period_type（daily / weekly / monthly）
-   metric_type（learning_days / learning_minutes）
-   target_value
-   effective_from
-   effective_until（可选）
-   enabled

有效学习日不能简单使用固定分钟数判断。AI
可以辅助评价活动是否构成有效投入，但必须给出理由并允许用户纠正。

## 3.11 生活记录

建议按类别保存： - SleepRecord - ExerciseRecord - MealRecord -
BodyMeasurement

这些数据与学习数据逻辑隔离，通过明确授权的分析接口提供给
AI。首版先支持手动录入和基础导入。

## 3.12 文件与授权

建议实体： - StoredAsset - UserAIDataPermission -
UserAIProviderCredential（后续个人 Key） -
AdminAIProviderConfig（首版管理员配置） - AuditEvent

密钥明文不可返回前端。健康数据授权应记录授权范围、状态、更新时间和用途。

------------------------------------------------------------------------

# 4. 核心业务流程

## 4.1 创建项目

1.  用户填写项目名称、目标、重要程度和日期。
2.  选择是否启用理论模块、实践模块。
3.  AI 可提出阶段、子目标、里程碑和初始内容结构。
4.  用户审阅、修改、确认或删除 AI 草案。
5.  用户决定项目是否立即进入进行中。
6.  创建项目投入保障规则（可选）。

AI 生成内容在用户确认前属于草案，不应作为正式计划或事实。

## 4.2 生成学习计划

输入： - 活跃项目及重要程度。 - 项目动态调度信息。 -
项目阶段和截止日期。 - 未完成任务及依赖。 - 复习队列和逾期情况。 -
投入保障规则。 - 用户每日目标分钟数和可用时间段。 - 用户手动标记。

输出： - 项目层资源分配建议。 - 项目内部任务优先级。 -
今日复习和新学习安排。 - 预计时长和推荐日期/时间段。 -
超出预算时的取舍解释。

系统先推荐，用户确认后写入正式安排。用户的每日学习分钟数是目标而非硬上限。

## 4.3 学习活动记录

1.  用户从统一入口创建记录。
2.  选择主要项目。
3.  可关联多个知识点/技能点。
4.  可选关联任务。
5.  选择手动填写、计时或事后补录。
6.  保存实际发生时间、时长和简短内容。
7.  根据活动类型生成学习证据。
8.  更新项目进度、统计和必要的复习资格。

## 4.4 掌握评价

1.  用户提交自评。
2.  系统根据证据生成系统评价。
3.  首次当前认定等级采用用户自评。
4.  系统评价附证据充分程度和解释。
5.  出现显著分歧或等级变化时生成建议。
6.  用户确认后更新当前认定等级，保留历史。

## 4.5 复习

1.  首次学习会话完成后，检查内容是否符合自动复习资格。
2.  L0 内容走单独处理策略。
3.  按内容类型选择复习策略。
4.  到期内容进入今日复习建议。
5.  用户完成复习并提交类型化反馈（允许跳过）。
6.  更新学习证据和下一次建议时间。
7.  逾期时只生成重新安排建议，用户确认后更新日程。

## 4.6 跨项目内容复用

1.  新项目规划发现某内容与用户已有内容相同或高度相关。
2.  系统检索已有掌握等级、证据充分程度和最近表现。
3.  比较新项目的目标要求等级。
4.  提出跳过、诊断、回顾或重新学习建议。
5.  用户确认规划。
6.  新项目的任务完成状态仍独立保存。

## 4.7 阶段切换

1.  系统持续观察阶段目标和关联证据。
2.  达到条件或接近完成时生成切换建议。
3.  展示已完成条件、未完成条件和证据。
4.  用户确认切换、继续当前阶段或手动选择其他阶段。
5.  更新阶段状态和历史，不自动伪造完成记录。

------------------------------------------------------------------------

# 5. 调度系统设计

## 5.1 两级调度

### 项目级调度

决定不同项目的建议资源倾向。输入包括： - 用户基础重要程度 I1---I5。 -
截止日期紧迫度。 - 阶段状态。 - 目标差距和剩余工作量。 - 学习风险。 -
前置依赖。 - 临时标记。 - 投入保障规则。 - 用户实际时间目标。

### 项目内任务调度

在项目获得建议资源后，再决定具体任务： - 到期复习。 - 新知识学习。 -
薄弱内容强化。 - 任务依赖。 - 阶段里程碑。 - 用户指定重点。

不能只给项目分配分钟数而不解释具体任务来源。

## 5.2 初期算法建议

首版先采用可解释的规则评分/排序，不急于引入不可解释的强化学习或复杂优化算法。

建议输出： - priority_score（内部数值）。 -
priority_reasons（可读原因列表）。 - deadline_risk。 -
commitment_risk。 - estimated_minutes。 - confidence /
data_sufficiency。

具体权重、归一化、截止日期曲线、冲刺标记加权和冲突规则仍待算法验证，不能在需求文档阶段假定某个公式已获确认。

## 5.3 时间预算策略

每日学习分钟数是目标，不是禁止超出的上限。

当推荐任务总时长超过目标： 1. 对候选任务排序。 2.
均衡模式下兼顾复习与新学习。 3. 可选择复习优先。 4.
可由用户指定延期复习日期。 5.
用户可以设置最低新学习时间保障，也可以不设置固定比例。 6. AI
给出推荐安排，用户确认或修改。

## 5.4 投入保障风险

对每条保障规则计算： - 周期总目标。 - 当前已完成投入。 -
周期剩余天数。 - 尚需投入。 - 当前可行性风险。

当多个项目保障互相冲突时，展示冲突，不擅自删除或改写规则。

------------------------------------------------------------------------

# 6. API 契约设计原则

本节定义契约原则，不冻结全部业务 URL。

## 6.1 基本约定

-   API 前缀建议 `/api/v1`。
-   JSON 请求与响应。
-   时间戳统一采用 ISO 8601/RFC 3339，明确时区或 UTC 语义。
-   日期字段使用 `YYYY-MM-DD`，不伪装成时间戳。
-   DTO 与数据库实体分离。
-   所有用户资源执行后端所有权校验。
-   分页、过滤、排序规范全局一致。
-   错误码稳定，错误信息可供用户理解。
-   PATCH 明确区分字段缺失、显式 null 和具体值。
-   OpenAPI 作为前后端契约来源，并纳入 CI 检查。

## 6.2 响应结构

可延续项目现有统一响应信封方向，例如：

``` json
{
  "code": 200,
  "message": "success",
  "data": {},
  "request_id": "..."
}
```

但需明确 HTTP 状态码仍表达协议层结果，业务 code 不能替代 HTTP
状态码。错误响应应有稳定错误标识、可读信息和可选字段级错误详情。

## 6.3 资源边界建议

可采用以下资源分组作为初步方向： - `/projects` -
`/projects/{project_id}/stages` - `/projects/{project_id}/goals` -
`/projects/{project_id}/milestones` - `/projects/{project_id}/tasks` -
`/learning-contents` - `/learning-sessions` - `/mastery-assessments` -
`/reviews` - `/questions` - `/practice-sessions` - `/artifacts` -
`/daily-goals` - `/scheduled-activities` - `/life-records` -
`/analytics`

项目内资源可采用嵌套路径；全局查询可采用独立资源并支持 `project_id`
过滤。最终路径应在领域模型和权限边界确定后冻结。

## 6.4 AI API 原则

AI API 应区分： - 生成草案。 - 分析证据。 - 生成建议。 -
用户确认建议。 - 取消或拒绝建议。

建议采用异步任务机制处理耗时生成或深度分析。所有 AI
生成结果应带有状态、来源、创建时间、输入上下文版本或引用，以便用户审阅和追溯。

------------------------------------------------------------------------

# 7. 安全、隐私与可靠性

## 7.1 内部 AI 服务认证

内部服务 Token 必须仅存在于服务端配置/密钥管理中。生产环境未配置 Token
时应拒绝请求，不得 permissive fallback。不能将内部服务 Token
暴露到前端可视化设置。

## 7.2 LLM Provider Key

首版由管理员配置全站模型密钥。密钥应加密存储或使用安全密钥管理服务，前端只显示已配置状态和掩码。后续个人
API Key 应按用户隔离。

## 7.3 健康数据

-   默认不向 AI 发送健康数据。
-   用户可按类别授权，也可开启全部数据授权。
-   每次调用仍应检查授权状态和用途。
-   撤销授权后，后续分析不得继续读取对应数据。
-   是否删除历史分析结果需提供清晰策略。

## 7.4 文件安全

-   文件类型、大小、内容和访问权限校验。
-   上传后生成安全存储标识。
-   图片压缩不覆盖唯一原始来源；用户要求保留原图，因此应保存原始图片或可恢复的原始版本。
-   下载链接采用受控访问或短期签名 URL。
-   防止跨用户访问附件。

## 7.5 数据完整性

-   重要状态变更保留历史。
-   AI 建议与用户确认分离。
-   计划、安排、实际活动分离。
-   自评、系统评价、当前认定分离。
-   删除采用明确的软删除/归档/彻底删除语义。
-   数据迁移和备份必须纳入部署流程。

------------------------------------------------------------------------

# 8. 测试策略

## 8.1 单元测试

重点覆盖： - 项目生命周期转换。 - 日期类型和逾期语义。 -
阶段切换规则。 - 任务依赖环检测。 - 跨项目内容复用判定。 -
掌握等级状态更新。 - 复习资格和 L0 分流。 - 复习延期历史。 -
项目投入保障周期统计。 - 调度冲突识别。 - 健康数据授权判断。

## 8.2 集成测试

-   注册登录和资源隔离。
-   项目创建到任务安排完整流程。
-   学习活动到证据、评价和复习队列流程。
-   题目录入、OCR 校对、练习和错题流程。
-   AI 草案确认流程。
-   文件上传、压缩、下载权限。
-   数据导出和删除。

## 8.3 契约测试

-   OpenAPI 与 DTO 一致。
-   前端请求类型与后端响应一致。
-   PATCH 缺失/null/值语义。
-   分页和错误响应一致。
-   AI Service 内部接口版本兼容。

## 8.4 端到端测试

至少覆盖： 1. 创建考研项目并生成阶段草案。 2. 用户确认阶段和学习内容。
3. 记录学习活动并提交自评。 4. 系统形成评价建议。 5.
首次学习完成后进入复习资格判断。 6. 完成复习并更新证据。 7. 创建 Rust
项目并复用已有内容。 8. 设置每周投入保障并检查风险提醒。 9.
录入题目、拍照 OCR、校对并练习。 10. 用户授权或撤销健康数据 AI
使用权限。

------------------------------------------------------------------------

# 9. 推荐实施路线

## Phase 0：需求和契约基线

-   PRD 评审。
-   领域词汇表。
-   核心业务流程。
-   ERD 草案。
-   API 通用规范。
-   前后端功能状态矩阵。

## Phase 1：核心基础

-   用户认证和授权。
-   Project 生命周期。
-   Stage / Goal / Milestone / Task。
-   理论/实践模块配置。
-   LearningContent 与跨项目关联。
-   基础文件管理。

## Phase 2：学习记录与评价

-   LearningSession。
-   多内容关联。
-   手动、计时、补录。
-   L0---L4 评价。
-   自评/系统评价/当前认定分离。
-   证据可信程度。

## Phase 3：复习系统

-   复习资格。
-   首次学习后入队。
-   L0 单独处理。
-   错题与知识点双轨复习。
-   类型化反馈。
-   逾期重排确认。
-   项目级复习策略。

## Phase 4：题库与实践档案

-   全部首版题型。
-   练习模式。
-   主观题自评与 AI 辅助评价。
-   AI 题目候选和用户确认。
-   图片 OCR、原图与压缩图。
-   实践成果档案。

## Phase 5：计划与调度

-   每日目标分钟数。
-   日期与具体时间段。
-   项目重要程度。
-   动态调度建议。
-   临时标记和期限。
-   投入保障与冲突提醒。
-   超预算任务优先级建议。

## Phase 6：可视化与生活记录

-   Dashboard。
-   项目和学习内容分析。
-   学习计划与实际投入对比。
-   手动睡眠、运动、饮食和体重记录。
-   基础生活数据图表。

## Phase 7：AI 深度分析与外部集成

-   深度学习分析。
-   更完善的错题和薄弱项诊断。
-   跨项目学习资源优化。
-   用户个人模型 API Key。
-   第三方健康数据同步。
-   移动端健康数据接入。
-   高级分析与相关性研究。

阶段可根据团队资源并行或调整，但不建议在核心领域语义尚未稳定前大规模开发
AI 自动规划。

------------------------------------------------------------------------

# 10. 关键设计风险与防护

  风险                        防护措施
  --------------------------- --------------------------------------------
  项目模型过度复杂            项目能力模块可选，简单项目不强制阶段
  跨项目内容误判为已完成      内容共享与项目进度分离，比较目标等级和证据
  AI 规划未经确认写入         草案、建议、已确认状态分离
  系统评价覆盖用户自评        三种等级分别保存，历史不可覆盖
  复习延期丢失历史            保存原始到期日和重排记录
  学习时间被误当硬上限        每日分钟数定义为目标值
  投入保障变成惩罚            只做监控和建议，不自动扣掌握等级
  记录负担过重                统一入口、快速表单、一次活动多内容关联
  AI 评分污染数据             AI 评分标记为辅助，用户可修订
  健康数据越权                明确授权、最小化读取、撤销检查
  AI 服务不可用阻断核心学习   核心记录和管理功能不依赖 AI 在线可用

------------------------------------------------------------------------

# 11. 当前不应冻结的设计

在完成进一步评审前，不要将以下内容写成不可变的数据库或 API 约束：

1.  动态调度的具体权重公式。
2.  每种内容类型的复习算法参数。
3.  L0---L4 的量化阈值。
4.  证据可信程度的具体计算方法。
5.  跨项目掌握度聚合公式。
6.  投入保障的有效学习日 AI 判定标准。
7.  AI 题目质量自动评分阈值。
8.  OCR 供应商、图片压缩参数和文件上限。
9.  AI 分析的异步任务状态机细节。
10. 完整数据库表拆分和最终 API URL。

这些项目应通过原型、测试数据和小规模算法实验确定。

------------------------------------------------------------------------

# 12. 设计评审检查表

在进入正式实现前，应完成：

-   [ ] 用户确认 PRD 中的需求基线。
-   [ ] 建立统一领域词汇表。
-   [ ] 明确 Project / Stage / Goal / Milestone / Task 边界。
-   [ ] 明确 LearningContent 与 ProjectContent 边界。
-   [ ] 明确 LearningSession、LearningEvidence 和 MasteryAssessment
    关系。
-   [ ] 明确复习队列和错题队列关系。
-   [ ] 明确计划、安排和实际活动的数据语义。
-   [ ] 明确用户数据隔离与文件权限。
-   [ ] 明确健康数据授权流程。
-   [ ] 完成 ERD 和数据字典评审。
-   [ ] 完成 API 契约和 OpenAPI 评审。
-   [ ] 建立前后端功能实施状态矩阵。
-   [ ] 将核心业务规则转化为自动化测试。
-   [ ] 明确首版不实现功能及其后续接口预留。

------------------------------------------------------------------------

# 附录 A：统一领域词汇

  名称                定义
  ------------------- --------------------------------------
  Project             围绕一个学习或成长目标组织资源的项目
  Stage               项目中的阶段
  Goal                项目或阶段需要达到的目标
  Milestone           可检查的阶段性成果节点
  Task                可执行的具体任务
  LearningContent     可复用的知识点或技能点本体
  ProjectContent      某项目对某学习内容的上下文关联
  LearningSession     一次实际发生的学习活动
  LearningEvidence    可用于评价掌握程度的证据
  MasteryAssessment   一次掌握等级评价事件
  MasteryState        当前掌握状态的便捷投影
  ReviewItem          自动或手动复习队列中的项目
  Question            题目
  PracticeSession     一次练习活动
  Artifact            实践成果档案
  DailyLearningGoal   用户某日希望达到的学习时长
  ScheduledActivity   计划或确认的学习安排
  LifeRecord          可选生活记录

# 附录 B：决策状态约定

本方案中： - 用户明确确认的产品规则作为需求约束。 -
本方案提出的实体、字段、模块边界和流程属于设计建议。 -
算法权重、阈值和复杂边界仍需验证。 -
任何后续偏离需求基线的实现，应先记录变更理由、影响范围和迁移方案。
