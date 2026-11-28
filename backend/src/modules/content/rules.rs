//! 内容域的字段校验、树与图的边界、以及状态迁移准入。
//!
//! 三条硬边界集中在这里，避免散落到 service 后互相矛盾：
//! 1. **正文模型**（ADR-009）——上限、空串归一、出处链接协议白名单；
//! 2. **树与图的保护上限**——深度 10、「4–5 层」只是界面建议不在此拒绝；
//! 3. **结论型状态**——`achieved` 只能由 M06 写入（D-27）。

use crate::shared::{AppError, AppResult, BodyFormat};

/// 内容树技术保护上限（M03 §7；界面建议的 4–5 层不在此拒绝）
pub const MAX_TREE_DEPTH: i32 = 10;

/// 单内容关系数上限（契约 §4.2：超出返回 422，列表不分页）
pub const MAX_RELATIONS: usize = 200;

/// 单内容标签数上限（契约 §4.3）
pub const MAX_TAGS: usize = 20;

/// 标题：trim 后 1–200 字符
pub fn ensure_title(field: &str, value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{field} 不能为空")));
    }
    if trimmed.chars().count() > 200 {
        return Err(AppError::Validation(format!(
            "{field} 长度不能超过 200 个字符"
        )));
    }
    Ok(trimmed.to_string())
}

/// 摘要：≤500 字符；空串归一为 `None`
pub fn normalize_description(value: Option<String>) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > 500 {
                return Err(AppError::Validation(
                    "description 长度不能超过 500 个字符".to_string(),
                ));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// 正文：≤100 000 字符；空串归一为 `None`（ADR-009 §1：避免 `''` 与 `NULL` 两种空并存）
pub fn normalize_body(value: Option<String>) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(text) => {
            if text.trim().is_empty() {
                return Ok(None);
            }
            if text.chars().count() > 100_000 {
                return Err(AppError::Validation(
                    "body 长度不能超过 100000 个字符".to_string(),
                ));
            }
            Ok(Some(text))
        }
    }
}

/// 正文解释方式；`body` 为空时格式无意义，仍保留用户选择
pub fn resolve_body_format(value: Option<BodyFormat>) -> BodyFormat {
    value.unwrap_or(BodyFormat::Markdown)
}

/// 出处链接：仅 http(s)、≤2048；空串归一为 `None`
pub fn normalize_source_url(value: Option<String>) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > 2048 {
                return Err(AppError::Validation(
                    "source_url 长度不能超过 2048 个字符".to_string(),
                ));
            }
            let lower = trimmed.to_ascii_lowercase();
            let is_http = lower.starts_with("http://") && trimmed.len() > "http://".len();
            let is_https = lower.starts_with("https://") && trimmed.len() > "https://".len();
            if !is_http && !is_https {
                return Err(AppError::Validation(
                    "source_url 仅支持 http / https 链接".to_string(),
                ));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// 标签名：trim 后 1–60 字符
pub fn ensure_tag_name(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("name 不能为空".to_string()));
    }
    if trimmed.chars().count() > 60 {
        return Err(AppError::Validation(
            "name 长度不能超过 60 个字符".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// 标签色：≤20 字符，仅展示用
pub fn normalize_tag_color(value: Option<String>) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > 20 {
                return Err(AppError::Validation(
                    "color 长度不能超过 20 个字符".to_string(),
                ));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// 关系自引用：422（DB 亦有 `CHECK` 兜底）
pub fn ensure_not_self_relation(source: uuid::Uuid, target: uuid::Uuid) -> AppResult<()> {
    if source == target {
        return Err(AppError::Validation("内容不能与自身建立关系".to_string()));
    }
    Ok(())
}

/// 挂到新父节点后的深度是否越界
pub fn ensure_depth_within_limit(depth: i32) -> AppResult<()> {
    if depth > MAX_TREE_DEPTH {
        return Err(AppError::Validation(format!(
            "内容树深度不能超过 {MAX_TREE_DEPTH} 层"
        )));
    }
    Ok(())
}

/// `ILIKE` 的通配符转义：`q` 是用户输入的片段，不应被 `%` / `_` 放大成模式匹配
pub fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// v1 不可写字段：请求体出现即 422（ADR-009 §3、契约 §2.1）。
///
/// 用「字段是否出现过」判定而不是 `deny_unknown_fields`：后者让错误走 axum 默认
/// rejection，响应体不是统一信封，与全局规范冲突。
pub fn reject_write_protected(field: &str, present: bool) -> AppResult<()> {
    if present {
        return Err(AppError::Validation(format!("{field} 在 v1 不可写")));
    }
    Ok(())
}
