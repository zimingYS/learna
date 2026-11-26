# Learna — 个人学习进度记录与 AI 分析系统

Learna 是一款以**学习进度记录和学习计划管理**为核心的自托管个人学习系统：支持同时管理考研、编程、外语、自律挑战等多个学习项目，记录计划与实际学习活动，评估知识掌握程度，调度复习，并由 AI 提供学习分析、计划建议与周期报告。PC 浏览器优先，适配手机浏览器。

> 设计依据（正式文档）：`docs/产品需求/Learna产品需求规格说明书V1.0.md`（需求基线）、`docs/架构设计/Learna系统设计方案V1.0.md`（系统设计）、`docs/数据库设计/Learna领域模型与ERD设计文档V1.0.md`（领域模型与 ERD）。`docs/历史参考/` 下的早期文档仅作历史参考。

## 核心能力

- **多项目并行**：每个项目可配置目标、阶段、里程碑、任务、理论/实践模块与投入保障规则。
- **学习过程可追踪**：学习活动（LearningSession）、计划安排（ScheduledActivity）与任务（Task）三者分离，事实与计划不混淆。
- **掌握程度可评估**：L0-L4 掌握度，自评、系统评价、当前认定分离保存，历史评价不可覆盖。
- **复习可调度**：按项目独立调度复习队列，支持显式共享复习结果，逾期只生成重排建议、由用户确认。
- **时间分配有依据**：按项目重要程度（I1-I5）、截止日期、阶段与投入保障生成可解释的调度建议；每日学习时长是目标而非硬上限。
- **低负担记录**：统一记录入口、一次活动可关联多个知识点/技能点，支持手动填写、计时与补录。
- **AI 辅助不越权**：AI 输出均为草案/建议，经结构校验与用户确认后才写入业务事实；核心学习功能不依赖 AI 在线。

## 架构速览

```
用户（PC / 手机浏览器）
          |
          v
     Vue 3 Web 前端
          |
   HTTPS / REST API
          v
   Rust 核心业务服务 -------------- PostgreSQL
          |                              |
          | 内部 HTTP API                | 结构化业务数据
          v                              |
     Python AI 服务                      |
          |       |                      |
          |       +---- 第三方模型 API    |
          |       +---- 文档解析 / OCR    |
          |                              |
   本地文件系统 <--- Storage 抽象 --------+
```

核心原则：

- **单一业务写入权威**：正式业务数据由 Rust 服务写入；Python AI 服务不直连业务库，Vue 前端不绕过后端写数据。
- **AI 输出 ≠ 业务事实**：模型输出必须经过结构校验、业务规则校验和权限判断；AI 建议由用户确认后生效。
- **核心学习功能不依赖 AI 可用性**：AI 服务故障时，手动学习管理仍可正常使用。
- **数据可追溯**：用户事实、系统评价、AI 建议、用户确认结果分离；计划、安排、实际活动分离；重要状态变更保留历史。

## 目录结构

```
learna/
├── frontend/                 # Vue 3 + TypeScript + Vite Web 前端
├── backend/                  # Rust Axum 核心业务服务
│   ├── migrations/           # SQLx 数据库迁移
│   ├── src/
│   └── tests/
├── ai-service/               # Python FastAPI AI 服务（仅内网）
│   ├── app/
│   └── tests/
├── deploy/                   # 部署配置
│   ├── compose.yaml          # 正式运行（Docker Compose）
│   ├── compose.dev.yaml      # 开发模式（仅数据库）
│   ├── docker/               # 各服务 Dockerfile
│   └── nginx/                # 反向代理配置
├── docs/                     # 文档（见下方文档地图）
├── scripts/                  # 工程脚本
├── .env.example              # 环境变量清单（复制为 .env 后填真实值）
└── README.md
```

## 快速开始（开发模式）

### 前置工具

| 工具 | 版本要求 | 用途 |
|---|---|---|
| Rust | stable（rustup） | backend |
| Node.js | 20+ | frontend（包管理使用 npm） |
| Python | 3.11+（推荐 uv） | ai-service |
| Docker | 最新稳定版 | 开发数据库 / 正式部署 |

### 启动步骤

```bash
# 1. 准备环境变量
cp .env.example .env
# 按需修改 .env 中的配置；AI_SERVICE_TOKEN 可用 openssl rand -hex 32 生成

# 2. 启动开发数据库
docker compose -f deploy/compose.dev.yaml up -d

# 3. 启动后端（端口 8080）
cd backend && cargo run

# 4. 启动 AI 服务（端口 8001，仅内网）
cd ai-service && uv sync
uv run uvicorn app.main:app --reload --port 8001
# 备选（venv + pip）：python -m venv .venv && source .venv/bin/activate && pip install -e . && uvicorn app.main:app --reload --port 8001

# 5. 启动前端（端口 5173）
cd frontend && npm install && npm run dev
```

### 端口分配

| 服务 | 端口 | 网络边界 |
|---|---|---|
| frontend（Vite dev） | 5173 | 本机 |
| backend（Rust API） | 8080 | 对浏览器开放 |
| ai-service（FastAPI） | 8001 | 仅内网，浏览器不可直连 |
| PostgreSQL | 5432 | 仅服务端访问 |

## 文档地图

| 目录 / 文件 | 性质 | 说明 |
|---|---|---|
| `docs/README.md` | 文档导航 | docs 目录总索引与维护约定 |
| `docs/AGENTS.md` | **硬约束** | Agent 开发硬约束，任务前必读（优先级高于 `docs/工程规范/`） |
| `docs/工程规范/` | 工程规范 | 工作流、代码风格、架构边界、领域不变量、安全与 AI、数据库与 API、测试、变更管理、交付报告 |
| `docs/产品需求/` | 确定文档 | 《Learna 产品需求规格说明书 V1.0》 |
| `docs/架构设计/` | 确定文档 | 《Learna 系统设计方案 V1.0》 |
| `docs/数据库设计/` | 确定文档 | 《Learna 领域模型与 ERD 设计文档 V1.0》 |
| `docs/接口契约/` | 确定文档 | API 契约（随开发沉淀） |
| `docs/开发指南/` | 确定文档 | 开发环境、约定与流程 |
| `docs/架构决策/` | 确定文档 | 架构决策记录（ADR） |
| `docs/历史参考/` | **历史参考** | 早期未定稿文档，仅供讨论参考，不作为实现依据 |

约定：**边开发边编写确定文档并放入对应位置**；修改领域规则必须先向 `docs/架构决策/` 提交 ADR 或设计变更说明（见 `docs/工程规范/领域模型不变量.md`）。开发前请阅读 `docs/AGENTS.md`。

## 技术栈

| 层级 | 技术 |
|---|---|
| 前端 | Vue 3 + TypeScript + Vite，Vue Router + Pinia，Axios |
| 后端 | Rust + Axum + Tokio，SQLx + Serde，Tracing |
| AI 服务 | Python + FastAPI + Pydantic |
| 数据库 | PostgreSQL（后续按需引入 pgvector） |
| 文件存储 | 本地文件系统（Storage 抽象，预留对象存储） |
| 部署 | Docker Compose + Nginx |

## CI

推送与 Pull Request 会触发 GitHub Actions（`.github/workflows/ci.yml`）：

- **backend**：`cargo check` + `cargo test`
- **frontend**：`npm ci` + `npm run build`
- **ai-service**：`uv sync` + `python -m compileall app`
- **CI**：汇总闸门作业，三个构建 job 全绿才通过（分支保护必需检查）

## 许可证

本项目采用 [PolyForm Noncommercial License 1.0.0](./LICENSE)：允许非商业目的的使用、修改与分发，**不允许商业使用**。商业授权请联系项目作者。
