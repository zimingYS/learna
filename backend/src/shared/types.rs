//! 公共类型

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};

/// 请求 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(pub String);

impl RequestId {
    /// 生成新的随机请求 ID（UUID v4）
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for RequestId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let id = RequestId::new();
    req.extensions_mut().insert(id);
    next.run(req).await
}
