# Learna 模块设计（总览）

**状态：** 随实现滚动更新的工程文档
**定位：** 回答「**代码怎么组织**」——模块边界、目录形态、内部分层、依赖方向、事务与错误映射、测试落点。

与相邻文档的分工（避免重复）：

| 文档 | 回答的问题 |
|---|---|
| `产品需求/` | 产品要做什么、版本边界 |
| `架构设计/`（TAD） | 系统有哪些组件、核心业务流程、调度算法 |
| `数据库设计/`（ERD） | 有哪些实体、字段、约束、索引、删除策略 |
| `接口契约/` | 对外接口长什么样（路径 / DTO / 错误码 / 信封） |
| `工程规范/架构约束.md` | 分层与模块化的**原则** |
| **本目录** | 上述原则在**每个具体模块**上的落法：文件放哪、谁调谁、事务开在哪、测试写在哪 |

一句话：其余文档说「应该怎样」，本目录说「这个模块具体这样」。

## 1. 模块划分原则

本目录按**业务能力**划分模块，与 `产品需求/` 的章节和 `数据库设计/` 的领域服务边界一一对应，而**不是**按 `backend/src/modules/` 的目录结构划分。两者是两套视角：

| 视角 | 粒度 | 用途 |
|---|---|---|
| 业务模块 M01~M12 | 一个完整能力（可能横跨多个 Rust 目录） | 讨论「这个功能归谁管」、排优先级、划事务与责任边界 |
| Rust 目录 `modules/*` | 一个聚合或一组强相关聚合 | 编译单元、`pub mod` 声明、代码落盘位置 |

因此存在三种非一一对应的情况，均属正常：

1. **一个 M 模块对应多个 Rust 目录**：M04 = `roadmap` + `task` + `schedule`；M05 = `learning_record`（过渡）+ `learning_session`（待建）+ `evidence`（待建）。
2. **一个 Rust 目录被一个 M 模块独占**：`auth` 只属 M01。
3. **横切能力不占模块号**：`shared` / `config` / `app_state` / `router` / `infrastructure` / `audit` 以及整个 `frontend/src` 没有 M 编号，约定收在本文件 §6。**审计不是业务能力**，它是所有模块共用的副作用，强行编号会诱导把 `audit` 写成业务域。

> **注意**：这里的「模块」是**逻辑边界**，不是部署单元。全部 M01~M12 在同一个 Rust 进程内，模块间通过函数调用协作，不引入 RPC、不拆服务。唯一的进程外边界是 M12 的 `ai-service`（Python）。

## 2. 分层与依赖方向

```text
                     ┌─────────────────┐
   HTTP 请求  ──────▶ │  router/        │  仅做路由表与方法绑定，不含逻辑
                     └────────┬────────┘
                              ▼
                     ┌─────────────────┐
                     │  modules/*/     │  handler → service → (repository)
                     │  handler        │  业务域
                     │  service        │
                     │  dto            │
                     └────────┬────────┘
                              ▼
        ┌──────────────────────────────────────────┐
        │  infrastructure/  database / storage /   │  技术能力，不含业务语义
        │                   ai_client / telemetry  │
        └──────────────────────────────────────────┘
                              ▲
                     ┌────────┴────────┐
                     │  shared/        │  error / response / middleware / types
                     │  config/        │  被所有层依赖，自身不依赖业务
                     │  app_state.rs   │
                     └─────────────────┘
```

**依赖规则（硬约束）**

1. 依赖只能**向下**：`router → modules → infrastructure → shared`。反向依赖一律禁止。
2. `modules` 之间**不得互相 import service**；需要协作时由调用方在自己的 service 里编排，或后续抽出领域服务。
   - 已知的跨模块协作只有一处：M02 的 `draft` 删除与 `purge` 要级联清理 M04 的规划子结构。约定由 **M02 在自己事务内直接执行级联 SQL**，不调用 M04 的 service——跨模块调用 service 会各自 `begin()`，事务边界立刻失效。
3. `shared` 与 `config` 不出现任何业务术语（不得出现 project / task 等类型）。
4. `infrastructure` 不返回业务错误语义，只返回技术错误（`Internal` / `Upstream`）。
5. 同层模块之间禁止循环依赖；出现即需拆分。

## 3. 模块清单

| # | 模块 | 后端目录 | 契约组（`接口清单.md`） | 主要表（ERD） | PRD | 优先级 | 实现状态 |
|---|---|---|---|---|---|---|---|
| M01 | 用户与权限 | `auth` `user` | §1 认证、§2 基础设施 | `users` | §15、§16.1 | P0 | ✅ auth 可用；user 空壳 |
| M02 | 学习项目管理 | `project` | §4.1（13 项） | `learning_projects` ＋ ⚠ 优先级覆盖 / 投入保障 / 事件表 | §4.2、§4.3、§5 | P0 | 🟡 空壳，契约已冻结 |
| M03 | 学习内容管理 | `knowledge` `library` | §4.2 | `learning_contents` `content_relations` `project_contents` ＋ ⚠ 标签表 | §6 | P1 | 🟡 空壳，契约未设计 |
| M04 | 学习规划与调度 | `roadmap` `schedule` `task`(待建) | §4.1（35 项）、§4.6 | `project_goals` `roadmaps` `stages` `milestones` `learning_tasks` `task_dependencies` `goal_stages` `daily_learning_goals` `scheduled_activities` | §4.4、§4.5、§5.2、§11.1、§11.4 | P0（规划）/ P3（调度） | 🟡 规划契约已冻结、代码空壳；调度未设计 |
| M05 | 学习活动与证据 | `learning_record` `learning_session`(待建) `evidence`(待建) | §3（过渡）、§4.3 | `learning_sessions` `session_projects` `session_contents` `task_sessions` `learning_evidence` | §11.2、§11.3 | P1 | 🔶 `/records` 过渡可用；正式模型未设计 |
| M06 | 掌握度评价 | `mastery`(待建) | §4.4（掌握度） | `mastery_assessments` `overall_mastery_assessments` `overall_mastery` | §7 | P2 | ⬜ 未开始 |
| M07 | 复习管理 | `review`(待建) | §4.4（复习） | `review_items` `review_records` `review_shares` ＋ ⚠ 策略 / 延期历史表 | §8 | P2 | ⬜ 未开始 |
| M08 | 题库与练习 | `question`(待建) `practice`(待建) | §4.4（错题部分） | ⚠ 全部未定义（ERD §12 第 2 项要求单独评审） | §9 | P2 | ⬜ 未设计 |
| M09 | 实践成果档案 | `artifact`(待建) | §4.5 | ⚠ `artifacts` `artifact_assets` `artifact_content_relations`（仅 TAD 概念） | §10 | P2 | ⬜ 未开始 |
| M10 | 生活记录 | `life_log`(待建) | §4.7 | ⚠ 全部未定义（ERD 无） | §12 | P3 | ⬜ 未设计（可选模块） |
| M11 | 数据分析 | `report` | §4.8（分析） | 无自有表（只读聚合） | §13 | P2 | 🟡 空壳，无数据源 |
| M12 | AI 智能服务 | `ai_task` `plan_change` + `ai-service/` | §4.8（AI） | `change_proposals` ＋ ⚠ `ai_tasks`（v1 不需要） | §14 | P3 | 🟡 骨架 + 5 个 501 占位 |

状态标记：✅ 已实现并可用 ｜ 🔶 过渡实现（非正式契约）｜ 🟡 空壳或骨架 ｜ ⬜ 未开始
表标记：⚠ = **ERD 尚未定义**，只有 TAD 概念或 PRD 要求；写 Migration 前必须先补 ERD（详见各模块文档 §5）

> **⚠ 合计 6 处模型缺口**：M02（优先级覆盖/投入保障/事件）、M03（标签）、M07（策略/延期历史）、M08（全部）、M09（全部）、M10（全部）。其中 **M08/M09/M10 三块在 ERD 里连一张表都没有**，属「需要独立建模」级别，不是补几列能解决的。

**模块拆分的三处判断，需要知道理由**：

1. **M02 与 M04 分开**。项目是**容器**，管的是生命周期、重要程度、投入保障；规划是**容器里的结构**，管的是目标、路线版本、阶段、任务。合并会让「项目暂停」和「阶段跳过」这两种完全不同粒度的状态机挤在一个 service 里。
2. **M05 与 M06 分开**。M05 记录「发生了什么」（客观事实），M06 判定「掌握到什么程度」（主观评价 + 证据推断）。ERD §3.2 明确区分了项目掌握度与个人总体掌握度，这说明评价本身有独立生命周期，不能混进活动记录。
3. **M11 没有表**。报表是纯只读聚合，它的复杂度在于「数据源是否存在」而非「表怎么设计」。单独立模块是为了让「没有数据可算」这个事实显式化，而不是躲在某个 service 里。

## 4. 实施优先级

与 ERD §11 一致，按依赖顺序而非模块编号：

| 阶段 | 模块 | 说明 |
|---|---|---|
| **P0** | M01、M02、M04（规划部分） | 身份 → 项目 → 规划结构。三者构成最小可用闭环，也是 48 项已冻结契约的落地区 |
| **P1** | M03、M05 | 有了项目和规划，才谈得上「学什么」和「学了多久」 |
| **P2** | M06、M07、M08、M09、M11 | 评价、复习、题库、成果、报表。互相依赖，建议 M06+M07 一起做（复习反馈要靠掌握度），M11 最后做（依赖前四者的数据） |
| **P3** | M04（调度部分）、M12 | 今日计划与 AI。依赖 P0~P2 的完整数据，AI 还需先定「AI 线形态」 |
| **可选** | M10 | 生活记录，不阻塞任何核心功能（PRD §12.1） |

## 5. 模块目录形态（强制）

遵循 `mod.rs` 纪律：**`mod.rs` 只做声明与门面 re-export，不写逻辑**。

```text
modules/<domain>/
├── mod.rs        # 仅 pub mod xxx; + pub use handler::…;   不超过 6 行
├── dto.rs        # 对外数据结构（请求 / 响应 / 查询参数）
├── handler.rs    # Axum 提取器 + 调 service + 包 ApiReply
└── service.rs    # 业务规则 + 数据访问（v1 不单独建 repository，见下）
```

复杂模块可再拆：`model.rs`（持久化行结构）、`repository.rs`（数据访问）、`rules.rs`（纯函数校验）、`graph.rs`（图算法）、`queries.rs`（聚合 SQL）、`tests.rs`（模块内单测）。

**为什么 v1 不建 repository 层**：`架构约束.md` §3 提到 Repository 职责，但它是**建议**不是硬性要求（原文「不是每个模块都必须包含全部层次」）。当前 `auth` / `learning_record` 的 SQL 直接写在 service 内，查询不超过 5 条、无复用。引入 repository 的门槛定为：

- 单模块聚合根 ≥ 3 个，或
- 同一查询被 ≥ 2 个 service 函数复用，或
- 需要为同一聚合提供多种存储后端

三者任一成立才抽 `repository.rs`。在此之前抽层只会增加跳转成本而无隔离收益。**M04 是最先会触到这条门槛的模块**（7 张表、3 个聚合根），预计拆到 `repository.rs` 的第一处就是它。

## 6. 跨模块约定

### 6.1 错误与响应

- handler 一律返回 `ApiReply<T>`，**不实现 `AppError: IntoResponse`**（`shared/error.rs` 已刻意移除，从编译层面杜绝空 `request_id`）。
- `ApiReply` 同时承载成功与失败，`request_id` 由 `request_id_middleware` 注入 `Extension<RequestId>` 后传入。
- 新增错误语义时**优先复用** `AppError` 现有变体；确需新增才扩展枚举，并同步 `status_code()` / `code()` 与 `接口契约/README.md` 的错误码表。
- 错误码与 HTTP 映射的唯一来源是 `接口契约/README.md` §5，代码不得自创。

### 6.2 权限校验位置

- 认证在中间件完成，产出 `Extension<UserId>`。认证中间件的两条硬约束见 §6.7。
- **归属校验在 service 的 SQL 条件里**（`WHERE id = $1 AND owner_id/user_id = $2`），不做事先 SELECT 再判断——一次查询同时完成存在性与归属校验，越权与不存在都返回 404，天然防 ID 探测。
- v1 **没有角色模型**：只有「本人的数据 / 非本人」两态。`AppError::Forbidden` 保留但不用。引入任何共享或协作能力前，必须先设计权限模型，不得在 service 里散写 `if role == ...`。

### 6.3 事务边界

- 事务开在 **service**，不开在 handler。多步写用一个 `pool.begin()` 贯穿。
- 跨表级联的完整清单见 `接口契约/设计记录/学习规划模块契约设计.md` §7；ADR-006 给出四层删除语义。
- **外部 HTTP 调用不得进事务**（M12 调 ai-service、M09 上传文件到对象存储）。正确形态是「事务1 写状态 → 事务外调外部 → 事务2 落结果」。

### 6.4 审计（横切，不占模块号）

- 必须审计的动作：`project.purge`、`project.extend_deadline`、`proposal.apply`、账号注销。
- 审计写入必须与业务变更**同一事务**，`record()` 的签名必须接受 `&mut sqlx::Transaction` 而非 `&PgPool`——传 pool 会另开连接，事务隔离立刻失效。
- **审计表不得有指向业务表的外键**。若加 `FK → projects ON DELETE CASCADE`，`purge` 会把审计记录一起删掉，审计形同虚设。只存裸 ID。
- 当前状态：**无审计表**，以 `tracing::warn!` 结构化字段替代（不可查询、不可关联、日志轮转即丢）。补表时需先补 ERD（属「契约层发现的模型缺口」）。
- 审计不提供对外查询端点（个人工具直接查库）。若将来要开，形态为 `GET /api/v1/audit/events?entity=&action=&from=&to=`，须先登记契约。

### 6.5 DTO 与 OpenAPI 的关系

- `openapi.yaml` 是**对外契约的唯一来源**；Rust `dto.rs` 与 TS `api/*.ts` 的字段名必须与之逐字对应（snake_case）。
- DTO 到 `openapi` 的偏移视为契约破坏，必须先改契约再由契约改代码，不允许反向。
- 「已设计未实现」的路径在 `openapi.yaml` 上标 `x-learna-status: designed`；实现后改标 `implemented`。`interim` 只用于 `/records` 这类过渡实现。

### 6.6 前端组织（横切，不占模块号）

```text
frontend/src/
├── api/<domain>.ts     # 一个 M 模块一个文件；只写资源段（如 /records），不带 /api/v1 前缀
├── stores/             # 只有跨页面共享的状态进 store（当前仅 auth）
├── views/              # 一个页面一个文件；超过 300 行必须拆子组件或 composable
└── router/index.ts     # 路由与守卫
```

- `http.ts` 的 `baseURL` 固定 `/api/v1`，vite proxy 与 nginx 均**透传 URI 不剥前缀**，改前缀只需改一处。
- 业务请求**直连 api 层，不经 store**。store 只装跨页面共享状态。
- 前端不做业务规则判断：字段合法性、状态机、权限一律由后端裁决，前端只做输入收敛与错误提示。
- 需要多步确认的操作（如删除带子项的项目）由「先 409 拿影响面 → 用户确认 → 带 `?cascade=true` 重发」驱动，不在前端预判级联范围。

### 6.7 认证中间件的两条硬约束（都踩过坑，勿回退）

1. `authenticate()` 必须是**同步 fn**。若写成 `async`，会跨 `await` 持有 `&Request`，而 axum 的 `Body` 是 `!Sync`，导致 `future` 失去 `Send`，`from_fn_with_state` 编译不通过。
2. `AppError` **不实现 `IntoResponse`**。错误出口只有 `into_response_with_request_id(&str)`，编译器强制每个错误响应携带真实 `request_id`。

### 6.8 测试落点

| 层级 | 位置 | 要求 |
|---|---|---|
| 集成测试（跨 HTTP + DB） | `backend/tests/<domain>_flow.rs` | 用 `sqlx::test`（每用例建独立库，需 `CREATEDB` 权限）；公共夹具在 `tests/support/mod.rs` |
| 模块内单测 | 同文件 `#[cfg(test)] mod tests` | 只测纯函数（校验、计算、状态判定、图算法），不碰 DB |
| 前端单测 | `frontend/tests/*.test.ts` | 测 API 封装与纯函数，不测组件渲染 |

**不要全量跑测试**：改动哪个模块就跑该模块的测试（`cargo test <module>`），提交前跑相关集成测试。

**夹具陷阱**：`tests/support/mod.rs` 里的 `register_user` 打的是 `/api/v1/auth/register`。改 API 前缀时必须同步此文件——曾因只改测试文件漏改 support 导致 pre-push 全红。

## 7. 新增一个模块的检查单

1. 在 `接口契约/接口清单.md` 登记路径，确认状态标记；未定项标 `❓` 并写进设计记录，**不得**先写代码后补契约。
2. 在 `modules/mod.rs` 声明 `pub mod <name>;`，建 `mod.rs`（仅声明 + re-export）。
3. 建 `dto.rs`（字段对照 openapi）、`service.rs`（业务规则 + SQL）、`handler.rs`（提取器 + `ApiReply`）。
4. 在 `router/` 下建同名路由文件，由 `router/routes.rs` 统一 `nest("/api/v1", …)`；需要认证的挂 `auth_middleware`。
5. 在 `openapi.yaml` 把该路径的 `x-learna-status` 从 `designed` 改为 `implemented`，并更新 `接口清单.md` 状态列。
6. 建 `backend/tests/<name>_flow.rs`：至少覆盖「正常路径 + 401 + 404（越权）+ 409/422（业务拒绝）」四类。
7. 在本目录补一份 M 模块文档，模板见 §8；若该模块跨多个 Rust 目录，文档只写一份，目录映射写进 §3 表格。

## 8. 模块文档模板

每份 M 文档固定 10 节，缺项写「不适用」而非删除小节：

1. **职责与边界**——做什么；明确列出不做什么（防止职责膨胀），并给出依据（PRD/ERD 章节）。
2. **目录与文件形态**——文件树 + 每个文件的职责一句话；未建的目录标注「待建」。
3. **契约映射**——对应 openapi 路径组与接口清单条目，含状态标记。
4. **调用链与分层**——一次请求从中间件到 SQL 的路径。
5. **数据与不变量**——涉及表、必守约束、由服务层而非 DB 保证的规则。
6. **状态与副作用**——状态机引用、时间戳与审计副作用。
7. **错误映射**——该模块每个业务失败对应哪个 `AppError` 变体与错误码。
8. **事务边界**——哪些操作必须单事务、锁策略。
9. **测试策略**——该模块的测试文件与必测场景。
10. **实现状态与缺口**——已完成 / 空壳 / 阻塞项，按依赖顺序排。

## 9. 文档维护

- 模块实现推进时**同步更新对应文档的第 10 节**，不另建进度文件。
- 模块边界变化（拆分 / 合并 / 改名 / 换编号）必须同步本文件 §3 表格与 `架构约束.md`。
- 新增跨模块约定时改本文件 §6，而非散落在各模块文档里。
- 编号 M01~M12 **不复用**：模块被合并时保留编号并在 §3 标注「已并入 Mxx」，避免历史文档引用失效。
