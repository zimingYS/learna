use crate::config::AppConfig;
use crate::modules::auth::dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::shared::{AppError, AppResult};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::chrono::Utc;

/// JWT 有效时长
pub const JWT_EXPIRATION: i64 = 7 * 24 * 3600;

/// JWT 载荷
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// 用户 ID
    sub: String,
    /// 过期时间
    exp: usize,
}

/// 数据库用户记录
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    display_name: String,
}

/// 生成 JWT
fn generate_token(user_id: &str, secret: &str) -> AppResult<String> {
    let expiration = Utc::now().timestamp() + JWT_EXPIRATION;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
    };

    let key = EncodingKey::from_secret(secret.as_bytes())
        .map_err(|e| AppError::Token(format!("encoding key: {e}")))?;

    encode(&Header::default(), &claims, &key)
        .map_err(|e| AppError::Internal(format!("JWT Encode: {e}")))
}

/// 密码哈希
async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .map_err(|e| AppError::Internal(format!("password hash task: {e}")))?
        .map_err(|e| AppError::Internal(format!("password hash: {e}")))
}

/// 将数据库记录映射为认证响应
fn build_auth_response(user: UserRow, secret: &str) -> AppResult<AuthResponse> {
    let token = generate_token(&user.id, secret)?;

    Ok(AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    })
}

/// 用户注册
pub async fn register(
    pool: &PgPool,
    config: &AppConfig,
    req: RegisterRequest,
) -> AppResult<AuthResponse> {
    // 检测邮箱是否注册
    let existing = sqlx::query_scalar::<_, String>("SELECT id::text FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("query existing user: {e}")))?;

    if existing.is_some() {
        return Err(AppError::Validation("邮箱已存在".to_string()));
    }

    // 密码哈希
    let password_hash = hash_password(req.password).await?;

    // 插入用户
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (
            email,
            password_hash,
            display_name
        )
        VALUES ($1, $2, $3)
        RETURNING
            id::text AS id,
            email,
            password_hash,
            display_name
        "#,
    )
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.display_name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        // 唯一约束冲突
        if let sqlx::Error::Database(ref db_err) = e
            && db_err.code().as_deref() == Some("23505")
        {
            return AppError::Validation("email already exists".to_string());
        }

        AppError::Internal(format!("insert user: {e}"))
    })?;

    // 签发 JWT
    build_auth_response(user, &config.jwt_secret)
}
pub async fn login(
    pool: &PgPool,
    config: &AppConfig,
    req: LoginRequest,
) -> AppResult<AuthResponse> {
    // 查询用户
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT
            id::text AS id,
            email,
            password_hash,
            display_name
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&req.email)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("query user: {e}")))?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(AppError::Unauthorized("invalid credentials".to_string()));
        }
    };

    // 验证密码
    let valid = verify(req.password, user.password_hash.as_str())
        .map_err(|e| AppError::Hash(format!("verify failed: {e}")))?;

    if !valid {
        return Err(AppError::Unauthorized("invalid credentials".to_string()));
    }

    // 签发JWT
    build_auth_response(user, &config.jwt_secret)
}
