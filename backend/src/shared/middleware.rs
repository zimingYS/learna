use crate::app_state::AppState;
use crate::shared::types::{Claims, UserId};
use crate::shared::{AppError, RequestId};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{DecodingKey, Validation, decode};
use uuid::Uuid;

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let id = RequestId::new();
    req.extensions_mut().insert(id);
    next.run(req).await
}

/// JWT 身份认证中间件
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 提取 Token
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::Unauthorized("missing token".to_string()))?;

    // 解码验证
    let secret = &state.config.jwt_secret;

    let key = DecodingKey::from_secret(secret.as_bytes())
        .map_err(|e| AppError::Token(format!("encoding key: {e}")))?;

    let token_data = decode::<Claims>(token, &key, &Validation::default())
        .map_err(|_| AppError::Unauthorized("invalid token".to_string()))?;

    // 提取用户ID
    let user_id = Uuid::parse_str(token_data.claims.sub.as_str())
        .map_err(|_| AppError::Unauthorized("invalid token".to_string()))?;

    // 注入扩展
    req.extensions_mut().insert(UserId(user_id));

    // 放行
    Ok(next.run(req).await)
}
