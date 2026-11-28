//! 证据类型与必填字段规则（ADR-010 §4 / D-36）。

use crate::shared::{AppError, AppResult, EvidenceType};

/// 标题长度上限，与 DB 的 `VARCHAR(200)` 同值
pub const MAX_TITLE_LEN: usize = 200;

/// v1 只开三类文本 / 链接证据；`code` / `image` / `file` 回 422 而不是 500
pub fn parse_evidence_type(value: &str) -> AppResult<EvidenceType> {
    EvidenceType::parse(value).map_err(|_| {
        AppError::Validation(format!(
            "证据类型 {value} 在 v1 未开放，仅支持 note / link / test_result"
        ))
    })
}

/// 空白归一为 NULL
pub fn normalize_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

pub fn normalize_title(value: Option<String>) -> AppResult<Option<String>> {
    let title = normalize_text(value);

    if let Some(title) = &title
        && title.chars().count() > MAX_TITLE_LEN
    {
        return Err(AppError::Validation(format!(
            "标题不能超过 {MAX_TITLE_LEN} 个字符"
        )));
    }
    Ok(title)
}

/// 只接受 http(s)：`javascript:` / `data:` 这类可执行协议不进库
pub fn normalize_url(value: Option<String>) -> AppResult<Option<String>> {
    let url = normalize_text(value);

    if let Some(url) = &url {
        let lowered = url.to_ascii_lowercase();
        if !(lowered.starts_with("http://") || lowered.starts_with("https://")) {
            return Err(AppError::Validation(
                "external_url 只接受 http(s) 链接".to_string(),
            ));
        }
    }
    Ok(url)
}

/// 按类型校验必填字段：`note` / `test_result` 用 `description`，`link` 用 `external_url`
pub fn ensure_payload(
    evidence_type: EvidenceType,
    description: &Option<String>,
    external_url: &Option<String>,
) -> AppResult<()> {
    match evidence_type {
        EvidenceType::Note | EvidenceType::TestResult => {
            if description.is_none() {
                return Err(AppError::Validation(
                    "note / test_result 类型必须填写 description".to_string(),
                ));
            }
        }
        EvidenceType::Link => {
            if external_url.is_none() {
                return Err(AppError::Validation(
                    "link 类型必须填写 external_url".to_string(),
                ));
            }
        }
    }
    Ok(())
}
