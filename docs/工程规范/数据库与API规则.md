# 数据库与 API 规则

## 1. 数据库

- 数据库变更必须通过 SQLx Migration。
- 不得修改已经执行的历史 Migration。
- 不得未经批准执行破坏性操作。
- Migration 必须考虑已有数据。
- 用户归属和数据隔离必须明确。
- 合理设计外键、唯一约束和索引。
- 避免不必要的冗余字段。
- 避免 N+1 查询。
- 数据库事务必须具有明确边界。

## 2. API 契约

API 必须遵循项目统一契约，必须明确：

- HTTP Method
- Endpoint
- Request DTO
- Response DTO
- Error Code
- Authentication
- Authorization
- Pagination
- Validation
- Data Ownership

修改 API 时必须同步更新 OpenAPI 和相关文档。不得只修改前端类型而不修改 API 契约，不得只修改后端接口而不通知相关调用方。
