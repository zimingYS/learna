# ADR-003 成功与失败响应统一携带 request_id

**状态：** 已采纳（2026-11-27）  
**范围：** 统一响应信封的 request_id 注入机制

## 背景

请求 ID 由中间件生成并放入请求扩展，但 `AppError: IntoResponse` 构造错误响应时未传入该 ID，错误信封的 `request_id` 恒为空字符串；只有成功路径手动 `.with_request_id(...)`。契约要求成功和失败响应都包含实际请求 ID。

## 决策

1. **移除 `AppError` 的 `IntoResponse` 实现**，改为 `into_response_with_request_id(self, request_id)`——从编译层面杜绝「错误响应缺 request_id」的路径。
2. 处理器统一返回 `ApiReply<T>(AppResult<T>, RequestId)`：成功与失败都在出口处注入真实 request_id。
3. `auth_middleware` 的错误路径同样从请求扩展读取 request_id 注入后返回。
4. 认证校验逻辑抽为**同步**函数：避免跨 await 持有 `&Request`（axum `Body` 是 `!Sync`，future 会失去 `Send`）。

## 后果

- 正面：所有经 `AppError` 的响应（含中间件拒绝）都带真实 request_id；漏传是编译错误而非运行时疏忽。
- 负面：`AppError` 不能再被 axum 自动转响应，处理器/中间件必须显式走 `ApiReply` 或 `into_response_with_request_id`——这是有意为之的约束。
- 已知边界：extractor 拒绝（JSON 解析失败等）仍走框架默认响应，不经统一信封——是否纳入属**待决策**（见现有接口审计 §3.3）。
