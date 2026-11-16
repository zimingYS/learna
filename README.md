# Learna — 个人学习进度记录与 AI 分析系统

个人自托管的学习进度记录与 AI 分析 Web 应用：围绕考研、考公、技能学习等目标建立学习项目，制定/导入学习路线，记录每日学习活动，并由 AI 提供学习分析、计划建议与周期报告。PC 浏览器优先，适配手机浏览器。

> 设计依据：`docs/undetermined/` 下的《产品需求文档》《技术架构设计文档》（参考稿）；随开发沉淀的确定文档见下方「文档地图」。

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
          |       +---- 文档解析          |
          |                              |
   本地文件系统 <--- Storage 抽象 --------+
```

核心原则：

- **单一业务写入权威**：学习项目、计划、记录、审批、报告等正式业务数据由 Rust 服务写入；Python AI 服务不直连业务库。
- **AI 输出 ≠ 业务事实**：模型输出必须经过结构校验、业务规则校验和权限判断；AI 建议由用户确认后生效。
- **核心学习功能不依赖 AI 可用性**：AI 服务故障时，手动学习管理仍可正常使用。

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

| 目录 | 性质 | 说明 |
|---|---|---|
| `docs/undetermined/` | **参考稿** | 未最终确定的设计文档，仅供讨论参考，不作为实现依据 |
| `docs/product/` | 确定文档 | 产品需求（随开发沉淀） |
| `docs/architecture/` | 确定文档 | 架构设计（随开发沉淀） |
| `docs/api/` | 确定文档 | API 契约 |
| `docs/database/` | 确定文档 | 数据库设计 |
| `docs/development/` | 确定文档 | 开发环境、约定与流程 |
| `docs/decisions/` | 确定文档 | 架构决策记录 |

约定：**边开发边编写确定文档并放入对应位置**；`undetermined/` 下的文档保持原样，仅作历史参考。

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

## 许可证

本项目采用 [PolyForm Noncommercial License 1.0.0](./LICENSE)：允许非商业目的的使用、修改与分发，**不允许商业使用**。商业授权请联系项目作者。
