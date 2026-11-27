//! 统一错误分类：
//! 业务错误码 + HTTP 状态码映射。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::response::ApiResponse;

/// 业务结果别名
pub type AppResult<T> = Result<T, AppError>;

/// 全站错误分类
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 入参校验失败（422）
    #[error("validation error: {0}")]
    Validation(String),

    /// 未认证（401）
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// 无权限（403）
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// 资源不存在（404）
    #[error("not found: {0}")]
    NotFound(String),

    /// 状态冲突 / 并发冲突（409）
    #[error("conflict: {0}")]
    Conflict(String),

    /// 限流（429）
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// 上游服务失败（502）
    #[error("upstream error: {0}")]
    Upstream(String),

    /// 内部错误（500）：非预期故障
    #[error("internal error: {0}")]
    Internal(String),

    /// 哈希错误
    #[error("hash: {0}")]
    Hash(String),

    /// Token异常
    #[error("token: {0}")]
    Token(String),
}

impl AppError {
    /// HTTP 状态码映射
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Hash(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Token(_) => StatusCode::UNAUTHORIZED,
        }
    }

    /// 稳定业务错误码（对外契约，禁止随意改字符串）
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimited(_) => "RATE_LIMITED",
            Self::Upstream(_) => "UPSTREAM_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Hash(_) => "HASH_ERROR",
            Self::Token(_) => "TOKEN_ERROR",
        }
    }
}

impl AppError {
    /// 转换为错误响应，必须携带中间件生成的真实 request_id
    /// （不实现 `IntoResponse`，从编译层面杜绝空 request_id 的错误响应）
    pub fn into_response_with_request_id(self, request_id: &str) -> Response {
        let status = self.status_code();
        let message = match &self {
            // 5xx 细节进日志，对外只给概括（避免泄漏内部结构）
            Self::Internal(detail) => {
                tracing::error!(error = %detail, "internal error");
                "internal error".to_string()
            }
            Self::Upstream(detail) => {
                tracing::warn!(error = %detail, "upstream error");
                detail.clone()
            }
            other => other.to_string(),
        };
        let body = ApiResponse::<()>::error(self.code(), message).with_request_id(request_id);
        (status, axum::Json(body)).into_response()
    }
}

/// 数据库错误统一
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(format!("database error: {err}"))
    }
}

/// HTTP 客户端错误映射为 Upstream
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        Self::Upstream(format!("http call failed: {err}"))
    }
}
