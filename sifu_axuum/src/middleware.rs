use axum::Json;
use axum::RequestPartsExt;
use axum::body::Body;
use axum::extract::{Extension, FromRequestParts};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use http::request::Parts;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use uuid::Uuid;

/// JWT 密钥 — 通过 Extension 注入，避免硬编码
#[derive(Clone)]
pub struct JwtSecret(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claims {
    pub id: String,
    pub jti: String,
    pub exp: usize,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub username: String,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUID: {}", self.id)
    }
}

impl AuthBody {
    fn new(token: String) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 首先尝试从请求的扩展中获取 Claims
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(claims.clone());
        }

        let secret = parts.extensions.get::<JwtSecret>()
            .map(|s| s.0.as_bytes().to_vec())
            .unwrap_or_else(|| b"fallback_secret".to_vec());
        let keys = Keys::new(&secret);

        // 尝试从 Authorization header 获取 token
        if let Ok(TypedHeader(Authorization(bearer))) =
            parts.extract::<TypedHeader<Authorization<Bearer>>>().await
        {
            if let Ok(token_data) = decode::<Claims>(bearer.token(), &keys.decoding, &Validation::default()) {
                return Ok(token_data.claims);
            }
        }

        // 尝试从 query 参数 ?token= 获取（用于 <video>/<img> 等无法发 header 的场景）
        if let Some(query) = parts.uri.query() {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    if key == "token" {
                        let decoded = urlencoding::decode(value).unwrap_or_default();
                        if let Ok(token_data) = decode::<Claims>(decoded.as_ref(), &keys.decoding, &Validation::default()) {
                            return Ok(token_data.claims);
                        }
                    }
                }
            }
        }

        Err(AuthError::InvalidToken)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

// 认证中间件函数，用于验证 JWT token
pub async fn auth_middleware(req: Request<Body>, next: axum::middleware::Next) -> Response {
    // 从请求中提取 Claims，这会自动验证 token
    let (mut parts, body) = req.into_parts();
    let result = Claims::from_request_parts(&mut parts, &()).await;

    match result {
        Ok(claims) => {
            // 检查 token 是否在黑名单中
            if let Some(redis_conn) = parts.extensions.get::<ConnectionManager>() {
                let mut conn = redis_conn.clone();
                let blacklist_key = format!("token:blacklist:{}", claims.jti);
                let is_blacklisted: bool = conn
                    .exists(&blacklist_key)
                    .await
                    .unwrap_or(false);

                if is_blacklisted {
                    return AuthError::InvalidToken.into_response();
                }
            }

            // token 验证成功，将请求重新组合并传递给下一个处理器
            let req = Request::from_parts(parts, body);
            next.run(req).await
        }
        Err(err) => {
            // token 验证失败，返回认证错误
            err.into_response()
        }
    }
}

// 从 token 中提取并返回 UUID
pub fn get_uid_from_token(token: &str, secret: &str) -> Result<String, AuthError> {
    let keys = Keys::new(secret.as_bytes());
    let token_data = decode::<Claims>(token, &keys.decoding, &Validation::default())
        .map_err(|_| AuthError::InvalidToken)?;

    Ok(token_data.claims.id)
}
