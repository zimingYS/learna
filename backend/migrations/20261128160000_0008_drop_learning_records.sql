-- M05 旧 Records 下线收尾（D-34 / ADR-010 §6）
--
-- 前置条件（本迁移已在开发库逐条核对）：
--   1. 0007 已把 learning_records 全量迁入 learning_sessions（反连接计数为 0）
--   2. duration_minutes 逐行比对一致
--   3. /api/v1/records 路由与 learning_record 模块已从代码中移除，无残留查询
--
-- 保留一版以便回滚：0006/0007 是「表还在、数据已复制」的回滚点；
-- 本迁移只负责删表，回滚需恢复备份（或重跑 0003 建表 + 补录数据），
-- 因为 learning_records 的 subject 已并入 summary 前缀，无法反解出原列。

DROP TABLE IF EXISTS learning_records;
