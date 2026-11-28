use serde::{Deserialize, Serialize};

/// 注册请求
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) display_name: String,
}

/// 登录请求
#[derive(Deserialize)]
pub struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

/// 认证响应
#[derive(Serialize)]
pub struct AuthResponse {
    pub(crate) token: String,
    pub(crate) user: UserResponse,
}

/// 用户响应
#[derive(Serialize)]
pub struct UserResponse {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}
