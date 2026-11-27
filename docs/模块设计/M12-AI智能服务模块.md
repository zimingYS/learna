# M12 AI 智能服务模块

**契约状态：** §4.8 AI 提案接口 ❓ 未设计；内部端点 5 个 501 占位（roadmaps / plans / learning / reports / documents）
**实现状态：** Rust 侧 `ai_task` / `plan_change` 空壳 + `AiClient` ✅ 可用；Python 侧骨架可启动、内部鉴权 ✅ fail-closed（启动期 + 请求期双闸门）、端点未实现
**依据：** PRD §14（AI 能力与密钥管理）、§3.2（AI 深度学习分析属后续版本）、§18 第 16 项（占位接口契约未定）；ERD §5.5 `ChangeProposal`；TAD §6.4（异步任务机制）、§11.9

## 1. 职责与边界

**做什么**：

| 层 | 职责 | 状态 |
|---|---|---|
| Rust `ai_task` | 发起 AI 任务、轮询/等待结果、把产出写入**草稿态**资源 | 🟡 空壳 |
| Rust `plan_change` | 变更提案：AI 建议的调整（改期、调优先级、阶段切换），须用户确认才生效 | 🟡 空壳 |
| Rust `AiClient` | 调 Python 内部端点，30s 超时、错误截断、内部 token | ✅ 可用 |
| Python `ai-service` | LLM 调用、提示词、结构化输出校验、密钥管理 | 🟡 骨架 |

**能力清单（PRD §14.1 九项，按落地成本排序）**：

| # | 能力 | 依赖 | 建议顺序 |
|---|---|---|---|
| 1 | 生成项目阶段、子目标和里程碑 | M04 契约（**已冻结**） | ① 最先做 |
| 2 | 生成知识点 / 技能点结构 | M03（契约未设计） | ③ |
| 3 | 规划学习任务 | M04 契约（已冻结） | ① |
| 4 | 提出阶段切换建议 | M04 状态机 + `ChangeProposal` | ② |
| 5 | 推荐项目资源分配 | M02 重要程度 + M04 调度 | ④ |
| 6 | 评价学习证据、提出掌握等级建议 | M06（算法未定）+ M05 | ⑤ |
| 7 | 分析错题和薄弱知识点 | M08（模型未锁定） | ⑥ |
| 8 | 生成练习题候选 | M08 | ⑥ |
| 9 | 分析学习趋势 | M11（无数据源） | ⑦ |

**不做什么**（红线，违反即设计失败）：

| 不做 | 依据 |
|---|---|
| **Rust 侧不直连 LLM**。`ENCRYPTION_MASTER_KEY` 与所有 provider key 只在 Python 侧 | PRD §14.2（密钥不得进前端/日志）、§16.1 |
| **AI 不得直接写生效数据**。所有产出先落 `draft` 或 `change_proposals.pending`，由用户动作转正 | PRD §4.4、§4.5、§14.1「AI 输出应区分草稿、待确认建议和已确认数据」 |
| **AI 不做判定**。阶段完成、掌握度认定最终由用户确认 | PRD §4.4、§7.2 |
| **占位接口不得向用户表现为已完成能力** | PRD §14.3 |
| **AI 不得擅自修改用户设置的基础重要程度** | PRD §5.1 |

**最后一条的工程含义**：`importance_level` 是用户长期偏好的表达，AI 只能读不能写。若某天要实现「AI 建议调整重要程度」，必须走 `ChangeProposal` 并明确标注为建议，且不能自动应用。

## 2. 目录与文件形态

```text
backend/src/modules/ai_task/        🟡 空 mod.rs
├── mod.rs
├── dto.rs                          # 待建：AiTaskRequest / AiTaskResult
├── handler.rs                      # 待建：发起 / 查询
└── service.rs                      # 待建：编排 AiClient + 落草稿

backend/src/modules/plan_change/    🟡 空 mod.rs
├── mod.rs
├── dto.rs                          # 待建：ProposalItem / AcceptProposalRequest
├── handler.rs                      # 待建：列表 / 详情 / accept / reject
├── service.rs                      # 待建：提案 CRUD + 接受时的领域校验编排
└── payload.rs                      # 待建：各类提案 payload 的结构校验（不能绕过领域规则）

backend/src/infrastructure/ai_client/   ✅
├── mod.rs                          # pub use client::AiClient
└── client.rs                       # new / post_internal（30s 超时、错误截断、token）

ai-service/                         🟡 骨架（鉴权已收紧）
├── app/main.py                     # FastAPI 实例 + lifespan 启动闸门 + /healthz + 挂 /internal
├── app/api/internal/
│   ├── router.py                   # prefix=/internal，统一 require_internal_token
│   ├── auth.py                     # X-Internal-Token 校验（fail-closed），错误走 ApiResponse 信封
│   ├── roadmaps.py                 # POST /internal/roadmaps/generate        → 501
│   ├── plans.py                    # POST /internal/plans/propose-adjustment → 501
│   ├── learning.py                 # POST /internal/learning/analyze         → 501
│   ├── reports.py                  # POST /internal/reports/generate         → 501
│   └── documents.py                # POST /internal/documents/parse          → 501
├── app/schemas/base.py             # ApiResponse + schema_version="v1"
└── app/config/settings.py          # pydantic-settings（ai_service_token + internal_token 归一化）
```

**为什么 Python 侧不按业务模块拆**：Python 的职责是「文本进、结构化 JSON 出」，它**不理解领域模型**——它不知道什么是 `superseded` 路线，也不需要知道。按业务拆成 12 个包会诱导把领域规则写进提示词与 Python 代码，形成两套规则。现状的「按端点类型拆（roadmaps / plans / learning / reports / documents）」是对的：一个端点对应一类产出结构。

## 3. 契约映射

**对外（浏览器 → Rust）**：

| 方法 | 路径 | 状态 |
|---|---|---|
| POST | `/api/v1/ai/roadmaps/generate` | ❓ |
| POST | `/api/v1/ai/plans/propose` | ❓ |
| GET | `/api/v1/ai/tasks/{id}` | ❓（若走异步） |
| GET | `/api/v1/proposals` | ❓ |
| GET | `/api/v1/proposals/{id}` | ❓ |
| POST | `/api/v1/proposals/{id}/accept` | ❓ |
| POST | `/api/v1/proposals/{id}/reject` | ❓ |

**对内（Rust → Python）**：

| 端点 | 状态 |
|---|---|
| `POST /internal/roadmaps/generate` | 501 |
| `POST /internal/plans/propose-adjustment` | 501 |
| `/internal/learning/*` | 空 router |
| `/internal/reports/*` | 空 router |
| `/internal/documents/*` | 空路由 |
| `GET /healthz` | ✅ 返回 `{"status":"ok"}` |

**内外信封刻意不同**：

| 层 | 信封 | `code` 形态 |
|---|---|---|
| 对外 | `接口契约/README.md` 的 `{code,message,data,request_id}` | `"200"` 字符串 |
| 对内 | `schemas/base.py` 的 `ApiResponse` + `schema_version: "v1"` | `"NOT_IMPLEMENTED"` 这类语义码 |

Rust 的 `post_internal` 只判断 HTTP 状态与反序列化，**不解析 `code`**。这样 Python 侧改语义码不会破坏 Rust，代价是 Rust 无法从 `code` 拿到细分错误——需要时靠 HTTP 状态 + 结构化 `error` 字段。

**九项待决策**（建议编号接续 D-90 起）：

| # | 待决策 | 影响 |
|---|---|---|
| 1 | **AI 线形态**（P0-③ 未启动） | 见 §10 阻塞 1。这是整个模块的总闸 |
| 2 | **同步 vs 异步** | v1 单用户建议同步（30s 超时）；长任务（整本书解析）需 `ai_tasks` 表 + 轮询 |
| 3 | **先做哪些能力** | 见 §1，建议路线生成 + 任务规划先行 |
| 4 | **提案 payload 的结构** | `content_create / roadmap_change / schedule_change / mastery_change` 四类的 schema 全部未定 |
| 5 | **提案过期规则** | `expired` 需时间依据（不能「查的时候判断」），过期时长未定 |
| 6 | **`accepted` 与 `applied` 的失败处理** | 应用失败回 `accepted` 还是新增 `failed` 状态 |
| 7 | **provider 与模型选择** | `settings.py` 只有 `ai_service_token`，缺 provider key、模型名、超时、重试 |
| 8 | **成本与配额** | 是否需要记录 token 消耗、设置日限额 |
| 9 | **降级策略** | AI 不可用时前端如何表现（PRD §14.3「允许用户继续使用不依赖 AI 的核心功能」） |

## 4. 调用链与分层

**生成路线草案（同步，推荐 v1 形态）**：

```text
前端 → POST /api/v1/ai/roadmaps/generate { project_id, goal_description }
  request_id_middleware → auth_middleware → Extension<UserId>
  handler::generate(State, Extension<UserId>, Extension<RequestId>, Json<…>)
    → ai_task::service::generate_roadmap(&db, &ai_client, user_id, req)
        1. 校验项目归属（SQL 带 owner_id）
        2. 构造请求体（只传必要上下文：项目名、目标描述、已有阶段数）
        3. **事务外**调 AiClient：
             state.ai_client.post_internal::<PythonReq, PythonResp>(
                 "/internal/roadmaps/generate", &body)
               - 30s 超时
               - header X-Internal-Token（未配置时省略，ai-service 回 503 并 debug 记日志）
               - 非 2xx → AppError::Upstream（body 截断 200 字符）
        4. **校验返回结构**（serde 反序列化 + 业务校验：阶段数上限、字段非空、层级合法）
        5. 事务：写 `roadmaps`（status=draft）+ `stages`（created_source='ai'）
        6. 返回 draft 路线（**不 confirm**，用户须手动确认）
    → ApiReply(Ok(RoadmapItem), req_id)
```

**为什么第 3 步在事务外**：外部 HTTP 不能进事务（`README.md` §6.3）。长事务会持锁数十秒，期间该项目的任何写操作都被阻塞。

**为什么不写 `ai_tasks` 表**：v1 同步调用，不需要任务表。**不要**为了「将来可能异步」而先建表——那会是一张永远空着的表，且让人误以为已经支持异步。

**接受提案（最复杂的事务）**：

```text
POST /api/v1/proposals/{id}/accept
  → plan_change::service::accept(&db, user_id, proposal_id)
      -- 单事务，四步
      1. SELECT * FROM change_proposals
          WHERE id = $1 AND owner_id = $2 AND status = 'pending'
          FOR UPDATE                       -- 锁提案，防重复接受
      2. 无行 → NotFound（不存在/非本人）或 Conflict（状态不是 pending）
      3. payload::validate(proposal_type, payload)?     -- 不能绕过领域校验
      4. 执行业务变更（由对应领域规则裁决，**不是**直接 UPDATE）：
           - roadmap_change → 走路线/阶段的状态机校验
           - schedule_change → 走调度冲突检测
           - mastery_change → 走 M06 的确认流程
      5. UPDATE change_proposals
            SET status = 'applied', applied_at = now()
      6. 写审计（proposal.apply）
      COMMIT
```

**关键：第 4 步不能跳过**。ERD §5.5 明文：「`payload` 不能绕过领域校验。接受提案后仍由对应领域服务执行命令」。这意味着 `plan_change` 不能写「直接按 payload UPDATE 表」，必须调用（或在同事务内实现）目标领域的校验逻辑。

**跨模块调用的事务陷阱**：若第 4 步调用 M04 的 service，它会自己 `begin()`，事务边界立刻失效。正确做法有两种：

| 方案 | 形态 | 适用 |
|---|---|---|
| **A** | `plan_change` 在同事务内直接执行目标领域的 SQL + 复用 `rules.rs` 纯函数做校验 | v1 推荐：提案类型少（4 类），SQL 简单 |
| B | 目标模块暴露 `apply_in_tx(&mut Transaction, …)` 函数，接受外部事务 | 提案类型增多后 |

A 的代价与 M02 的 `purge` 级联同构（`README.md` §2 依赖规则 2 已约定）。

## 5. 数据与不变量

**唯一相关表：`change_proposals`（ERD §5.5）**：

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | UUID | PK |
| `owner_id` | UUID | FK → `User` |
| `proposal_type` | VARCHAR(40) | `content_create / roadmap_change / schedule_change / mastery_change` |
| `target_type` | VARCHAR(40) | 目标实体类型 |
| `target_id` | UUID | 目标实体 ID，可空（新建类提案无目标） |
| `payload` | JSONB | 提议变更（结构未定，见 §3 决策 4） |
| `reason` | TEXT | 提议理由（可解释性，PRD §16.4） |
| `status` | VARCHAR(20) | `pending / accepted / rejected / expired / applied` |
| `created_at` / `reviewed_at` / `applied_at` | TIMESTAMPTZ | |

**`ai_tasks` 表不存在**。v1 同步调用不需要它；若将来改异步，需新建（ERD 无此表）。

**必须保证的不变量**：

| 不变量 | 依据 |
|---|---|
| 提案必属一个用户，且记录提出者（`ai`）与提出时间 | ERD §5.5 |
| **`accepted` 与 `applied` 必须区分**：用户接受 ≠ 系统已执行，中间可能失败 | ERD §5.5 |
| `expired` 需有时间依据，不能「查的时候判断」 | 否则同一提案今天可接受、明天不可，行为不可预测 |
| 一条提案被接受后，产生的变更必须可追溯（写审计） | PRD §14.1、`README.md` §6.4 |
| AI 产物落库时一律 `created_source = 'ai'`（或 `ai_confirmed`） | ERD §5.1 已有该列；便于统计与回溯 |
| 提案不得直接改 `importance_level` | PRD §5.1 |

**最后一条的实现位置**：写进 `payload.rs` 的白名单校验——`roadmap_change` 的 payload 里出现 `importance_level` 字段直接拒绝，而不是指望「业务逻辑不会这么做」。

## 6. 状态与副作用

**`change_proposals` 状态机**：

| 当前 | 动作 | 目标 | 副作用 |
|---|---|---|---|
| （无） | AI 生成建议 | `pending` | `created_at`；`reason` 必填 |
| `pending` | 用户接受且应用成功 | `applied` | `reviewed_at` + `applied_at`；执行业务变更；写审计 |
| `pending` | 用户接受但应用失败 | `accepted`（或新增 `failed`） | `reviewed_at`；业务变更已回滚；**需决策 6** |
| `accepted` | 重试应用 | `applied` | `applied_at` |
| `pending` | 用户拒绝 | `rejected` | `reviewed_at` |
| `pending` | 超时 | `expired` | 需定时任务或写入时计算过期标记 |

**`expired` 的实现陷阱**：若靠「查询时判断 `created_at + 7d < now()`」，则数据库里的 `status` 永远是 `pending`，而 API 返回 `expired`——两处不一致，统计与排查都会出错。正确做法是**定时任务批量置 `expired`**（个人工具可接受每天一次），或干脆**不做 `expired` 状态**、只保留 `pending` 并在响应里返回 `is_expired: true`。**推荐后者**：v1 提案量小，少一个状态机分支就少一类 bug。

**副作用红线**：

| 动作 | 写入 |
|---|---|
| AI 生成路线草案 | `roadmaps`（`status=draft`）+ `stages`；`created_source='ai_confirmed'` 仅在用户确认后改 |
| AI 生成任务建议 | 不直接写 `learning_tasks`，落 `change_proposals`（`proposal_type=roadmap_change`） |
| 接受提案 | 业务变更 + `status=applied` + 审计，**同一事务** |
| AI 调用失败 | 不写任何业务行；`Upstream` 错误带 detail 供排查 |

## 7. 错误映射

| 场景 | 变体 | HTTP | 备注 |
|---|---|---|---|
| ai-service 不可达 / 超时 / 5xx | `Upstream` | 502 | detail 保留（便于定位），但截断 200 字符 |
| AI 返回结构不合预期 | `Upstream` | 502 | **不要映射为 500**——这是上游问题，不是本站 bug |
| 内部 token 不匹配（Python 侧） | — | 401 | body 为 Python 的 `ApiResponse.error("40101", …)` |
| 内部 token 未配置（Python 侧） | — | 503 | `code=INTERNAL_AUTH_UNCONFIGURED`；启动期已拦，此处是纵深防御 |
| 提案不存在 / 非本人 | `NotFound` | 404 | |
| 提案已处理（非 `pending`）再接受 | `Conflict` | 409 | |
| 提案已过期 | `Conflict` | 409 | 若保留 `expired` 状态 |
| `payload` 结构非法 / 含禁止字段 | `Validation` | 422 | `payload.rs` 白名单 |
| `payload` 合法但违反领域规则（如把 `superseded` 路线改为 active） | `Conflict` | 409 | 由目标领域裁决 |
| AI 能力未配置（无 provider key） | `Upstream` / 503 | 502/503 | 需决策 9；message 必须明说「AI 未配置」，不得伪装成通用错误 |

## 8. 事务边界

| 操作 | 事务 |
|---|---|
| **调用 ai-service** | **绝不进事务**。长事务会持锁数十秒 |
| 写 AI 草稿产物 | 单事务：路线 + 阶段一起写（否则会出现「有路线没阶段」的空壳） |
| **接受提案** | **必须单事务**：锁提案 → 校验 payload → 执行业务变更 → 更新提案状态 → 写审计。五步任一失败整体回滚，否则会出现「提案标记已应用但数据没变」 |
| 拒绝提案 | 单语句 |
| 生成提案（AI 返回后落库） | 单语句（只写 `change_proposals` 一行） |

**正确的三段式**：

```text
事务1（可选）：写入 pending 提案（若 AI 是异步的）
  ↓
事务外：调 AI
  ↓
事务2：校验结果 → 写草稿资源 / 更新提案
```

## 9. 测试策略

| 层 | 方式 |
|---|---|
| `AiClient` | 单测：`base_url` 为空 → 构造失败；非 2xx → `Upstream`；token 缺失 → 不 panic 且不带 header；超时 30s |
| `ai-service` | `tests/test_smoke.py` 已有：`/healthz` 200；`/internal/*` 无 token 401 |
| 提案流程 | 集成测试：提案 → 接受 → 应用 → 审计；重复接受 409；payload 非法 422；payload 合法但违反领域规则 409 |
| **LLM 调用** | **不测真实模型**。用固定响应桩（或 `respx` 拦 httpx），只验「结构解析 + 落库 + 错误映射」 |
| 降级 | ai-service 不可达时，学习管理的全部核心端点仍正常（PRD §14.3） |

**必测的三条**：

1. **AI 返回的路线是 `draft` 而非 `active`**——这条防的是「AI 直接生效」。
2. **接受提案时 payload 违反领域规则 → 409 且提案仍为 `pending`**（事务回滚，状态未被改）。
3. **ai-service 挂掉后，`/api/v1/projects` 等核心端点全部正常**（PRD §14.3 的降级要求）。

## 10. 实现状态与缺口

**已实现**：`AiClient`（超时、错误截断、token 警告）、FastAPI 骨架、内部 token 中间件、统一信封、5 个子路由 + 2 个 501 占位、smoke 测试。

**缺口（按阻塞程度）**：

| # | 缺口 | 影响 |
|---|---|---|
| 1 | **AI 线形态未设计**（P0-③ 未启动） | **整个域无法开工**。需先定：哪些能力先做、同步还是异步、失败重试与降级策略、密钥管理与成本上限 |
| 2 | **`change_proposals` 无 Migration** | `plan_change` 无从下手 |
| 3 | **无 LLM provider 抽象** | `settings.py` 只有 `ai_service_token`；缺 provider key、模型名、超时、重试 |
| 4 | **无结构化输出校验** | LLM 返回的 JSON 需 pydantic 模型校验，否则脏数据直接进库 |
| 5 | **无配额与成本控制** | 个人工具风险低，但至少记录 token 消耗，否则无法判断异常 |
| 6 | **5 个子路由中 3 个空** | `learning.py` / `reports.py` / `documents.py` 只有空 router |
| 7 | **Rust 侧两个模块全是空 `mod.rs`** | 无 DTO、无 handler、无 service |
| 8 | 无审计表 | 提案应用无法追溯（`README.md` §6.4） |

**建议的落地路径（成本最低的切口）**：

先做「**路线生成**」一条端到端链路：

```text
POST /api/v1/ai/roadmaps/generate
  → Python 生成结构化阶段草案
  → Rust 校验后写 draft 路线
  → 用户在前端点「确认」→ 走已有的 M04 confirm 契约
```

选它的三个理由：

1. **契约已冻结**（M04 的 35 项），AI 产物有明确的落点，不需要新设计接口；
2. **风险可控**：产出是 `draft`，用户不确认就不生效，最坏情况是「AI 生成得不好，删掉重来」；
3. **验证价值高**：一条链路能同时验证密钥管理、结构化输出校验、错误降级、审计四件事。

`plan_change`（变更提案）复杂得多——它要对接 4 类领域规则、要区分 `accepted`/`applied`、要处理过期与重试。**建议延后到路线生成跑通之后**，那时对 AI 输出的可信度已有实际体感。

**一个必须提前定的工程约束**：Python 侧的结构化输出校验必须是**严格模式**（`extra='forbid'`），否则 LLM 多返回一个字段就会让 Rust 的 `serde` 反序列化失败或静默忽略。前者表现为 502，后者表现为「AI 说要改 3 个阶段，实际只改了 1 个」——后者更难排查。
