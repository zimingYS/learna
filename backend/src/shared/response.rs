//! 统一 API 响应结构，使用字符串业务码
//! 成功为 `CODE_OK`，失败使用稳定业务码。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use super::error::AppResult;
use super::types::RequestId;

/// 成功业务码
pub const CODE_OK: &str = "200";

/// 统一响应信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 业务码
    pub code: String,
    /// 人类可读信息
    pub message: String,
    /// 业务数据
    pub data: Option<T>,
    /// 请求追踪 ID
    pub request_id: String,
}

impl<T> ApiResponse<T> {
    /// 成功封装
    pub fn ok(data: T) -> Self {
        Self {
            code: CODE_OK.to_string(),
            message: "success".to_string(),
            data: Some(data),
            request_id: String::new(),
        }
    }

    /// 失败封装
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            data: None,
            request_id: String::new(),
        }
    }

    /// 追加 request_id
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = request_id.into();
        self
    }
}

/// 成功响应直接以 HTTP 200 返回
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}

/// 处理器统一返回值：成功与失败响应都携带真实 request_id
pub struct ApiReply<T>(pub AppResult<T>, pub RequestId);

impl<T: Serialize> IntoResponse for ApiReply<T> {
    fn into_response(self) -> Response {
        let id = self.1.as_str().to_string();
        match self.0 {
            Ok(data) => (
                StatusCode::OK,
                axum::Json(ApiResponse::ok(data).with_request_id(id)),
            )
                .into_response(),
            Err(err) => err.into_response_with_request_id(&id),
        }
    }
}
