-- M05 旧 Records 数据一次性迁移（D-34 / ADR-010 §6）
--
-- 映射规则：
--   title            → title（截断到 200，空串归 NULL）
--   content          → summary
--   subject          → 并入 summary 前缀「科目：<subject>」（D-30，不丢弃）
--   duration_minutes → duration_minutes；区间按 created_at 合成
--   created_at       → created_at；started_at = created_at，ended_at = created_at + duration
--   created_source   → 'imported'（API 拒绝该值，只有迁移能写）
--
-- 区间合成是**近似**：过渡记录没有真实起止时间，只能拿录入时间当发生时间，
-- 因此 summary 末尾追加一行说明，不假装精确（ADR-010 §6）。
-- 不双写：迁移后 /records 与 learning_records 进入下线流程，两侧不再并行写入。

INSERT INTO learning_sessions (
    id, owner_id, primary_project_id, title, activity_type,
    started_at, ended_at, duration_minutes,
    summary, created_source, created_at, updated_at
)
SELECT r.id,
       r.user_id,
       NULL,
       NULLIF(left(btrim(r.title), 200), ''),
       'mixed',
       r.created_at,
       r.created_at + make_interval(mins => GREATEST(r.duration_minutes, 0)),
       GREATEST(r.duration_minutes, 0),
       NULLIF(
           btrim(concat_ws(E'\n',
               CASE
                   WHEN r.subject IS NOT NULL AND btrim(r.subject) <> ''
                       THEN '科目：' || btrim(r.subject)
               END,
               r.content,
               '由过渡记录迁移，时间区间为估算'
           )),
           ''
       ),
       'imported',
       r.created_at,
       r.updated_at
FROM learning_records r
WHERE NOT EXISTS (SELECT 1 FROM learning_sessions s WHERE s.id = r.id);
