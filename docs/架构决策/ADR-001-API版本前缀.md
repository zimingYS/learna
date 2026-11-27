# ADR-001 API 版本前缀 /api/v1

**状态：** 已采纳（2026-11-27）  
**范围：** HTTP API 版本策略

## 背景

现有路由（`/auth/*`、`/records/*`）无版本前缀，且 dev 代理（vite rewrite）与 nginx（`proxy_pass` 带尾斜杠）都在剥离 `/api` 前缀，前端与后端看到的路径不一致。TAD §6.1 建议 API 前缀 `/api/v1`。

## 决策

1. 所有业务接口统一挂在 **URL 路径前缀 `/api/v1`** 下（如 `/api/v1/auth/login`、`/api/v1/records`）。
2. 健康检查 `/healthz`、`/readyz` 为基础设施端点，**不版本化**，保留根路径。
3. 反向代理**透传完整 URI**：vite 代理删除 rewrite，nginx `proxy_pass` 去掉尾斜杠。
4. 前端 axios `baseURL` 改为 `/api/v1`。
5. 版本演进：v1 内只做向后兼容增量；破坏性变更开 `/api/v2`。

## 后果

- 正面：路径即版本，契约清晰；前后端/代理三方路径口径统一；为 v2 并行留出空间。
- 负面：旧路径立即失效——项目处于 pre-release，调用方仅自家前端与集成测试，已同步更新，无外部兼容负担。
- 集成测试与前端 api 封装已随本 ADR 同步。
