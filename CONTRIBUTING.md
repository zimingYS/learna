# 贡献指南（Contributing Guide）

感谢你对本项目的关注与贡献！

为保证代码质量、提交历史的可读性以及协作流程的一致性，所有贡献均应遵循本指南。

## 1. 分支管理

项目采用基于 `main` 的轻量分支模型。

### 1.1 主分支

- `main` 是项目的主分支，始终保持稳定、可构建状态。
- `main` 受分支保护规则约束，禁止直接推送（Direct Push）。
- 所有代码和文档变更必须通过 Pull Request（PR）合并。
- 不得在 `main` 上直接开发或提交变更。

### 1.2 分支命名

开发新功能、修复问题或修改文档时，应从最新的 `main` 创建独立分支。

分支命名格式：

```text
<prefix>/<short-description>
```

允许使用以下前缀：

| 前缀 | 用途 | 示例 |
|---|---|---|
| `feat/` | 新增功能 | `feat/user-auth` |
| `fix/` | 修复缺陷 | `fix/login-error` |
| `refactor/` | 代码重构（不改变外部行为） | `refactor/extract-service` |
| `docs/` | 修改文档 | `docs/update-readme` |
| `chore/` | 日常维护、配置或非功能性变更 | `chore/update-dependencies` |

命名要求：

- 使用小写英文。
- 多个单词使用连字符（`-`）连接。
- 简述应清晰表达本次分支的主要目的。
- 一个分支原则上只处理一项独立任务。

### 1.3 创建分支

```bash
# 获取远程最新代码
git switch main
git pull origin main

# 创建并切换到开发分支
git switch -c feat/your-feature
```

## 2. Commit 提交规范

项目采用 [Conventional Commits](https://www.conventionalcommits.org/) 规范，以保持提交历史清晰、统一，并与现有提交风格保持一致。

### 2.1 提交格式

```text
type(scope): subject
```

字段说明：

| 字段 | 说明 |
|---|---|
| `type` | 提交类型，必须使用规定的类型 |
| `scope` | 变更影响的模块或范围 |
| `subject` | 对本次变更的简短描述 |

### 2.2 Type 白名单

仅允许使用以下提交类型：

| Type | 说明 |
|---|---|
| `feat` | 新增功能 |
| `fix` | 修复缺陷 |
| `docs` | 文档变更 |
| `chore` | 日常维护及杂项变更 |
| `refactor` | 代码重构，不改变外部行为 |
| `test` | 添加或修改测试 |
| `build` | 构建系统或依赖配置变更 |
| `ci` | 持续集成配置变更 |

不得使用白名单之外的提交类型。

`scope` 推荐使用模块名（如 `auth`、`project`、`roadmap`、`schedule`），也可使用服务/目录名（如 `frontend`、`backend`、`ai-service`、`deploy`、`docs`）。无明确影响范围时可省略 `scope`。

### 2.3 提交示例

**推荐写法：**

```text
feat(auth): add user authentication
fix(api): handle invalid request parameters
docs(readme): update quick start guide
refactor(user): simplify user service
test(auth): add login unit tests
build(deps): update dependency versions
ci(github): add backend test workflow
chore(config): update project configuration
```

**不推荐写法：**

```text
update code
fix bug
修改登录功能
feat: add feature
```

### 2.4 提交要求

- 每次提交应尽量只包含一类相关变更。
- `subject` 应简洁、明确地描述变更内容。
- 不要提交无关文件、临时文件、构建产物或敏感信息。
- 避免使用含义模糊的提交信息。
- 提交前应检查变更内容，确保没有意外修改。
- 仓库提供 Git 钩子辅助检查（`bash scripts/git/install-hooks.sh` 启用）：`pre-commit` 拦截敏感文件并跑快速质量检查，`commit-msg` 校验提交格式，`pre-push` 跑三栈测试。紧急情况可用 `LEARNA_SKIP_HOOKS=1` 临时跳过，事后须补跑 `scripts/check/check.sh` 与 `scripts/test/test.sh`。

## 3. Pull Request 流程

所有变更必须通过 Pull Request 合并到 `main`。

### 3.1 贡献流程

```text
Fork 或从 main 创建开发分支
              |
              v
          编写代码
              |
              v
       本地检查与测试
              |
              v
       推送到远程仓库
              |
              v
       创建 Pull Request
              |
              v
          CI 检查
              |
              v
        Code Review
              |
              v
       Squash and Merge
              |
              v
        合并到 main
```

### 3.2 提交 Pull Request

创建 PR 前，应确保：

- 开发分支基于最新的 `main`。
- 代码已完成必要的本地检查和测试。
- 提交信息符合 Conventional Commits 规范。
- 不包含无关变更或敏感信息。
- PR 描述能够清晰说明变更目的和具体内容。

PR 标题建议使用与 Commit 一致的格式：

```text
feat(scope): subject
```

创建 PR 时会自动带出 PR 模板（`.github/pull_request_template.md`），请按模板填写**变更内容、动机与关联、变更类型**，并完成**自查清单**。

PR 描述应至少包含：

- **变更目的**：本次修改解决了什么问题，或实现了什么功能。
- **主要变更**：说明涉及的模块和关键改动。
- **测试情况**：说明执行过的检查和测试。
- **关联事项**：如有对应 Issue，请附上关联信息。

### 3.3 CI 检查

PR 必须通过项目配置的 CI 检查。

- 所有必需的 CI 检查必须通过。
- CI 检查未通过时，应先修复问题，再重新提交。
- 不得通过跳过检查或绕过分支保护规则的方式合并 PR。
- 合并前必须保证 CI 全绿。

### 3.4 合并方式

项目统一采用 **Squash and Merge**。

- 将 PR 中的多个提交压缩为一个提交。
- 合并后的提交信息应符合 Conventional Commits 规范。
- 不使用普通 Merge Commit 或 Rebase and Merge。
- 合并完成后，可删除已合并的临时开发分支。

## 4. Code Review 代码审查

代码审查用于保证变更的正确性、可维护性和一致性。

### 4.1 审查要求

启用 **1 人审批（1 required approval）**：所有 PR 合并前必须获得至少 1 名审查者的批准，并通过分支保护规则强制执行。

现阶段的合并要求：

- 合并前必须 CI 全绿。
- 合并前必须获得至少 1 名审查者批准（1 required approval）；PR 作者不能批准自己的 PR。
- PR 应完成必要的自查。
- 涉及敏感路径（`.github/`、`deploy/`、`backend/migrations/`、`.env.example`、`LICENSE`）时，由 `CODEOWNERS` 指定的负责人审查（启用 "Require review from Code Owners" 后强制生效）。
- 发现问题时，应修复后再合并。

### 4.2 审查关注点

审查时应重点关注：

- **正确性**：业务逻辑是否正确，是否存在边界条件遗漏。
- **可维护性**：代码结构是否清晰，是否存在不必要的重复或耦合。
- **安全性**：是否存在敏感信息泄露、权限绕过或潜在安全风险。
- **兼容性**：是否影响现有功能、接口或数据结构。
- **测试覆盖**：是否为新增功能和修复逻辑提供必要测试。
- **规范性**：是否符合项目现有代码风格和目录组织约定。

## 5. 开发环境

开发环境准备、依赖安装、项目启动及基础配置，请参考：

- **[docs/开发指南/本地开发环境.md](./docs/开发指南/本地开发环境.md)**：环境搭建、服务启动与质量工具命令（详细版）。
- **[README.md - 快速开始（开发模式）](./README.md#快速开始开发模式)**：五分钟上手摘要。

如开发环境配置、启动命令或依赖版本发生变化，应同步更新上述文档，避免文档与实际项目状态不一致。

## 6. 文档约定

项目采用「边开发边沉淀」的文档策略（详见 README「文档地图」）：

- **正式设计文档**（实现依据）：`docs/产品需求/Learna产品需求规格说明书V1.0.md`（产品需求）、`docs/架构设计/Learna系统设计方案V1.0.md`（系统设计）、`docs/数据库设计/Learna领域模型与ERD设计文档V1.1.md`（领域模型与 ERD）。
- **工程规范**：`docs/AGENTS.md`（硬约束）与 `docs/工程规范/`（工作流、代码风格、架构边界、领域不变量、安全、测试、变更管理等），贡献者与 Agent 均须遵守。
- `docs/历史参考/` 下是**早期历史参考稿**，仅供讨论参考，不作为实现依据。
- 随开发确定的文档放入对应目录：`docs/产品需求/`（产品需求）、`docs/架构设计/`（架构设计）、`docs/接口契约/`（API 契约）、`docs/数据库设计/`（数据库设计）、`docs/开发指南/`（开发环境与流程）、`docs/架构决策/`（架构决策记录）。
- 修改 API、数据库结构或重要架构设计的 PR，必须同步更新对应确定文档（见 PR 自查清单）。
- Bug 报告与功能需求请通过 Issue 提交（`.github/ISSUE_TEMPLATE/` 提供表单）。

## 7. 其他约定

- 本项目使用 **PolyForm Noncommercial License 1.0.0**（见 `LICENSE`）：允许非商业目的的使用、修改与分发，**不允许商业使用**。提交贡献即表示你同意以相同许可授权你的贡献内容。

- 不得提交 API Key、密码、访问令牌、私钥及其他敏感信息。
- 不得将本地环境配置、数据库运行数据、日志、缓存和编译产物提交到 Git。
- 新增依赖前应评估必要性，避免引入不必要的第三方依赖。
- 修改 API、数据库结构或重要架构设计时，应同步更新相关文档（见第 6 节）。
- 对于不确定的设计或规范问题，建议先通过 Issue 或 PR 讨论，再实施变更。
- 紧急热修若经管理员绕过分支保护直接修复，事后须补 PR 说明变更与原因，保持审计可追溯。
- Dependabot 自动提交的依赖升级 PR（`chore(deps)` / `ci(deps)` 前缀）在 CI 全绿后可合并；升级失败或行为存疑时转人工排查。

---

再次感谢你为项目做出的贡献！
