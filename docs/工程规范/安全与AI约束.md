# 安全与 AI 约束

## 1. 业务数据权威

- Rust Backend 是正式业务数据的唯一写入权威。
- Python AI Service 不直接连接业务数据库。
- Vue Frontend 不得绕过 Backend 直接写入业务数据。

## 2. AI Proposal

AI 生成的业务变更必须经过以下流程：

1. 创建 AI Task。
2. 构建必要上下文。
3. 调用 AI Workflow。
4. 校验输出结构。
5. 执行业务规则校验。
6. 执行权限校验。
7. 生成 Proposal。
8. 根据 Proposal 的审批规则由用户确认。
9. 由 Rust Backend 的领域服务应用变更。
10. 保留必要的操作记录。

AI 不得直接覆盖或删除历史学习记录。AI Task 和 Proposal 是不同的领域实体，不得合并。

## 3. 可用性

AI 服务不可用时，核心学习管理功能仍然必须能够正常使用。

## 4. 安全

- 所有用户数据必须经过身份认证和资源级授权。
- 不同用户的数据必须严格隔离。
- 密钥必须保存在服务端。
- 不得在前端暴露 `LLM_API_KEY`。
- `AI_SERVICE_TOKEN` 与 `LLM_API_KEY` 必须区分。
- 不得提交真实密钥、密码或隐私数据。
- 「不得绕过现有权限机制」的推论：**内部服务鉴权必须 fail-closed**。`AI_SERVICE_TOKEN` 未配置（缺失、空串、纯空白）时，ai-service **启动失败**，且即使绕过启动检查，`/internal/*` 也必须返回 503 `INTERNAL_AUTH_UNCONFIGURED` —— 任何情况下都不允许降级为放行。

## 5. 内部服务鉴权（backend ↔ ai-service）

| 层 | 未配置 `AI_SERVICE_TOKEN` 时的行为 |
|---|---|
| `deploy/compose.yaml` | 启动失败（`${AI_SERVICE_TOKEN:?…}`，与 `JWT_SECRET` 同级） |
| ai-service 启动 | lifespan 校验抛 `RuntimeError`，进程退出，不进入服务状态 |
| ai-service 请求 | 503 `INTERNAL_AUTH_UNCONFIGURED`（纵深防御，防止运行期被换成空配置） |
| backend 调用 | 收到 503 → `AppError::Upstream` → 对前端 502，日志给出可定位原因 |

- 两层闸门（启动期 + 请求期）都必须存在：只留启动期，运行期配置被清空就漏；只留请求期，部署错误会被静默拖到第一次调用才暴露。
- `/healthz` 免鉴权（存活探针），**不得**在探针里塞业务数据。
- 令牌比较必须按 bytes 走 `secrets.compare_digest`：字符串版遇非 ASCII 会抛 `TypeError`，把非法 header 变成 500。
