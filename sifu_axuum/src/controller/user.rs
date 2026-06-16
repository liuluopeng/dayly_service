use crate::middleware::{AuthBody, AuthError, AuthPayload, Claims, JwtSecret};
use anyhow;
use chrono;
use axum::Json;
use axum::Router;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use common::api::user::{LoginRequest, LoginResponse};
use jsonwebtoken::{EncodingKey, Header, encode};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct User {
    id: Uuid,
    username: String,
    password: String,
    is_admin: bool,
    hash: Option<String>,
}

pub async fn user_login(
    Extension(pg_pool): Extension<PgPool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<ApiResponse<LoginResponse>> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(ApiError::bad_request(ApiError::EMPTY_CREDENTIALS, "用户名和密码不能为空"));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password, is_admin, hash FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_optional(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    if let Some(user) = user {
        // 优先用 bcrypt hash 验证，fallback 到明文比较（过渡期兼容）
        let password_ok = if let Some(ref hash) = user.hash {
            bcrypt::verify(&payload.password, hash).unwrap_or(false)
        } else {
            user.password == payload.password
        };
        if password_ok {
            let uid = user.id.to_string();
            let jti = Uuid::now_v7().to_string();
            let claims = Claims {
                id: uid,
                jti,
                exp: (chrono::Utc::now().timestamp() + 86400 * 30) as usize,
                is_admin: user.is_admin,
                username: user.username,
            };

            let encoding_key = EncodingKey::from_secret(jwt_secret.0.as_bytes());
            let token = encode(&Header::default(), &claims, &encoding_key)
                .map_err(|e| ApiError::Internal(format!("Token 生成失败: {}", e)))?;

            return Ok(ApiResponse::ok(LoginResponse {
                token,
                token_type: "Bearer".to_string(),
            }));
        }
    }

    Err(ApiError::unauthorized(ApiError::WRONG_PASSWORD))
}

pub async fn user_logout(
    claims: Claims,
    Extension(mut redis_conn): Extension<ConnectionManager>,
) -> impl IntoResponse {
    // 将 token 的 jti 加入黑名单，TTL 设为 token 剩余有效期
    let now = chrono::Utc::now().timestamp() as usize;
    let ttl = if claims.exp > now {
        (claims.exp - now) as u64
    } else {
        0
    };

    let blacklist_key = format!("token:blacklist:{}", claims.jti);
    let _: Result<(), _> = redis_conn
        .set_ex(&blacklist_key, "1", ttl)
        .await;

    StatusCode::OK
}

#[derive(Debug, Serialize)]
struct UserListItem {
    id: Uuid,
    username: String,
    is_admin: bool,
}

// 管理员：列出所有用户
pub async fn list_users(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<UserListItem>>> {
    if !claims.is_admin {
        return Err(ApiError::forbidden(ApiError::ADMIN_REQUIRED, "需要管理员权限"));
    }

    let users = sqlx::query_as::<_, (Uuid, String, bool)>(
        "SELECT id, username, is_admin FROM users ORDER BY username",
    )
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("查询用户失败: {}", e)))?;

    let items: Vec<UserListItem> = users
        .into_iter()
        .map(|(id, username, is_admin)| UserListItem { id, username, is_admin })
        .collect();

    Ok(ApiResponse::ok(items))
}

// ---- User Settings ----

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserSettings {
    pub language: String,
    pub flutter_theme: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSettings {
    pub language: Option<String>,
    pub flutter_theme: Option<String>,
}

pub async fn get_user_settings(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<UserSettings>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let settings = sqlx::query_as::<_, UserSettings>(
        "SELECT language, flutter_theme FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("查询用户设置失败: {}", e)))?;

    match settings {
        Some(s) => Ok(ApiResponse::ok(s)),
        None => Err(ApiError::not_found(ApiError::USER_NOT_FOUND, "用户不存在")),
    }
}

pub async fn update_user_settings(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
    Json(payload): Json<UpdateUserSettings>,
) -> ApiResult<ApiResponse<UserSettings>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let settings = sqlx::query_as::<_, UserSettings>(
        "UPDATE users SET
            language = COALESCE($2, language),
            flutter_theme = COALESCE($3, flutter_theme)
         WHERE id = $1
         RETURNING language, flutter_theme",
    )
    .bind(user_id)
    .bind(payload.language)
    .bind(payload.flutter_theme)
    .fetch_optional(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("更新用户设置失败: {}", e)))?;

    match settings {
        Some(s) => Ok(ApiResponse::ok(s)),
        None => Err(ApiError::not_found(ApiError::USER_NOT_FOUND, "用户不存在")),
    }
}

pub fn user_routes() -> Router {
    Router::new()
        .route("/login", post(user_login))
        .route("/logout", post(user_logout))
}

pub fn secured_user_routes() -> Router {
    Router::new()
        .route("/list", get(list_users))
        .route("/settings", get(get_user_settings))
        .route("/settings", put(update_user_settings))
        .route("/debug/redis", get(debug_redis))
}

async fn debug_redis(
    claims: Claims,
    Extension(mut redis_conn): Extension<ConnectionManager>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !claims.is_admin {
        return Err(ApiError::forbidden(ApiError::ADMIN_REQUIRED, "需要管理员权限"));
    }
    use redis::AsyncCommands;

    let mut result = serde_json::Map::new();

    // 获取所有 key
    let keys: Vec<String> = redis_conn
        .keys("*")
        .await
        .unwrap_or_default();

    for key in &keys {
        // 获取类型
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query_async(&mut redis_conn)
            .await
            .unwrap_or_else(|_: redis::RedisError| "unknown".to_string());

        let value = match key_type.as_str() {
            "string" => {
                let v: Option<String> = redis_conn.get(key).await.unwrap_or(None);
                serde_json::json!({ "type": "string", "value": v })
            }
            "list" => {
                let v: Vec<String> = redis_conn.lrange(key, 0, -1).await.unwrap_or_default();
                serde_json::json!({ "type": "list", "values": v })
            }
            "set" => {
                let v: Vec<String> = redis_conn.smembers(key).await.unwrap_or_default();
                serde_json::json!({ "type": "set", "members": v })
            }
            "zset" => {
                let v: Vec<(String, f64)> = redis_conn
                    .zrange_withscores(key, 0, -1)
                    .await
                    .unwrap_or_default();
                let items: Vec<serde_json::Value> = v
                    .into_iter()
                    .map(|(member, score)| serde_json::json!({ "member": member, "score": score }))
                    .collect();
                serde_json::json!({ "type": "zset", "items": items })
            }
            "hash" => {
                let v: std::collections::HashMap<String, String> =
                    redis_conn.hgetall(key).await.unwrap_or_default();
                serde_json::json!({ "type": "hash", "fields": v })
            }
            _ => serde_json::json!({ "type": key_type }),
        };

        result.insert(key.clone(), value);
    }

    Ok(Json(serde_json::json!({
        "total_keys": keys.len(),
        "keys": result
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    /// 测试 GET /api/user/settings — 正常获取设置
    #[tokio::test]
    async fn test_get_user_settings() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_get_settings_user").await;
        let claims = test_claims(user_id, false);

        let result = get_user_settings(claims, Extension(pool.clone())).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.msg, "ok");
        let settings = resp.data.unwrap();
        assert_eq!(settings.language, "zh");
        assert_eq!(settings.flutter_theme, "dark");

        cleanup_test_user(&pool, user_id).await;
    }

    /// 测试 GET /api/user/settings — 用户不存在
    #[tokio::test]
    async fn test_get_user_settings_not_found() {
        let pool = test_pool().await;
        let fake_id = Uuid::now_v7();
        let claims = test_claims(fake_id, false);

        let result = get_user_settings(claims, Extension(pool.clone())).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::NotFound { .. } => {} // expected
            other => panic!("期望 NotFound, 得到: {:?}", other),
        }
    }

    /// 测试 PUT /api/user/settings — 更新语言
    #[tokio::test]
    async fn test_update_user_settings_language() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_update_lang_user").await;
        let claims = test_claims(user_id, false);

        let payload = UpdateUserSettings {
            language: Some("en".to_string()),
            flutter_theme: None,
        };
        let result = update_user_settings(claims, Extension(pool.clone()), Json(payload)).await;
        assert!(result.is_ok());

        let settings = result.unwrap().data.unwrap();
        assert_eq!(settings.language, "en");
        assert_eq!(settings.flutter_theme, "dark"); // 未修改

        cleanup_test_user(&pool, user_id).await;
    }

    /// 测试 PUT /api/user/settings — 更新主题
    #[tokio::test]
    async fn test_update_user_settings_theme() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_update_theme_user").await;
        let claims = test_claims(user_id, false);

        let payload = UpdateUserSettings {
            language: None,
            flutter_theme: Some("light".to_string()),
        };
        let result = update_user_settings(claims, Extension(pool.clone()), Json(payload)).await;
        assert!(result.is_ok());

        let settings = result.unwrap().data.unwrap();
        assert_eq!(settings.language, "zh"); // 未修改
        assert_eq!(settings.flutter_theme, "light");

        cleanup_test_user(&pool, user_id).await;
    }

    /// 测试 PUT /api/user/settings — 同时更新两个字段
    #[tokio::test]
    async fn test_update_user_settings_both() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_update_both_user").await;
        let claims = test_claims(user_id, false);

        let payload = UpdateUserSettings {
            language: Some("en".to_string()),
            flutter_theme: Some("light".to_string()),
        };
        let result = update_user_settings(claims, Extension(pool.clone()), Json(payload)).await;
        assert!(result.is_ok());

        let settings = result.unwrap().data.unwrap();
        assert_eq!(settings.language, "en");
        assert_eq!(settings.flutter_theme, "light");

        // 验证持久化：再次 GET
        let claims2 = test_claims(user_id, false);
        let get_result = get_user_settings(claims2, Extension(pool.clone())).await.unwrap();
        let s = get_result.data.unwrap();
        assert_eq!(s.language, "en");
        assert_eq!(s.flutter_theme, "light");

        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_user_login_success() {
        let pool = test_pool().await;
        // 创建一个有 bcrypt hash 的用户
        let user_id = Uuid::now_v7();
        let username = format!("test_login_{}", user_id.as_simple());
        let hash = bcrypt::hash("testpass123", 4).unwrap();
        sqlx::query(
            "INSERT INTO users (id, username, password, hash, language, flutter_theme) VALUES ($1, $2, '', $3, 'zh', 'dark')",
        )
        .bind(user_id)
        .bind(&username)
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();

        let req = LoginRequest {
            username: username.clone(),
            password: "testpass123".to_string(),
        };
        let result = user_login(
            Extension(pool.clone()),
            Extension(JwtSecret("test_secret".to_string())),
            Json(req),
        )
        .await;
        assert!(result.is_ok());
        let resp = result.unwrap().data.unwrap();
        assert!(!resp.token.is_empty());

        // 清理
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_user_login_wrong_password() {
        let pool = test_pool().await;
        let user_id = Uuid::now_v7();
        let username = format!("test_login_bad_{}", user_id.as_simple());
        let hash = bcrypt::hash("correctpass", 4).unwrap();
        sqlx::query(
            "INSERT INTO users (id, username, password, hash, language, flutter_theme) VALUES ($1, $2, '', $3, 'zh', 'dark')",
        )
        .bind(user_id)
        .bind(&username)
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();

        let req = LoginRequest {
            username,
            password: "wrongpass".to_string(),
        };
        let result = user_login(
            Extension(pool.clone()),
            Extension(JwtSecret("test_secret".to_string())),
            Json(req),
        )
        .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unauthorized { .. } => {}
            other => panic!("期望 Unauthorized, 得到: {:?}", other),
        }

        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_list_users_requires_admin() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_list_user_nonadmin").await;
        let claims = test_claims(user_id, false);

        let result = list_users(claims, Extension(pool.clone())).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Forbidden { .. } => {}
            other => panic!("期望 Forbidden, 得到: {:?}", other),
        }

        cleanup_test_user(&pool, user_id).await;
    }
}
