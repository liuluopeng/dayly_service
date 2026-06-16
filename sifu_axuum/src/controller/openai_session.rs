use crate::middleware::Claims;
use axum::{
    Extension, Json, Router,
    routing::{delete, get, post},
};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::model::openai::{OpenAiMessage, OpenAiSession};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    role: String,
    content: String,
    think: Option<String>,
    cite: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct SessionWithMessages {
    session: OpenAiSession,
    messages: Vec<OpenAiMessage>,
}

// 创建新会话
async fn create_session(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Json(req): Json<CreateSessionRequest>,
) -> ApiResult<ApiResponse<OpenAiSession>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let session = sqlx::query_as::<_, OpenAiSession>(
        r#"INSERT INTO openai_sessions (title, user_id) VALUES ($1, $2) RETURNING *"#,
    )
    .bind(&req.title)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(session))
}

// 获取会话列表
async fn list_sessions(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<OpenAiSession>>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let sessions = sqlx::query_as::<_, OpenAiSession>(
        r#"SELECT * FROM openai_sessions WHERE user_id = $1 ORDER BY updated_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(sessions))
}

// 获取会话详情
async fn get_session(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> ApiResult<ApiResponse<SessionWithMessages>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let session =
        sqlx::query_as::<_, OpenAiSession>(r#"SELECT * FROM openai_sessions WHERE id = $1 AND user_id = $2"#)
            .bind(&session_id)
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                error!("Database error: {:?}", e);
                ApiError::not_found(ApiError::SESSION_NOT_FOUND, "会话不存在")
            })?;

    let messages = sqlx::query_as::<_, OpenAiMessage>(
        r#"SELECT * FROM openai_messages WHERE session_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(&session_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(SessionWithMessages { session, messages }))
}

// 删除会话
async fn delete_session(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> ApiResult<ApiResponse<()>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    sqlx::query(r#"DELETE FROM openai_sessions WHERE id = $1 AND user_id = $2"#)
        .bind(&session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            ApiError::Internal(e.to_string())
        })?;

    Ok(ApiResponse::ok(()))
}

// 在会话中添加消息
async fn add_message(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<AddMessageRequest>,
) -> ApiResult<ApiResponse<OpenAiMessage>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    // 验证会话属于当前用户
    let _session = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM openai_sessions WHERE id = $1 AND user_id = $2"#,
    )
    .bind(&session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::not_found(ApiError::SESSION_NOT_FOUND, "会话不存在"))?;

    let message = sqlx::query_as::<_, OpenAiMessage>(
        r#"INSERT INTO openai_messages (session_id, role, content, think, cite) VALUES ($1, $2, $3, $4, $5) RETURNING *"#
    )
    .bind(&session_id)
    .bind(&req.role)
    .bind(&req.content)
    .bind(&req.think)
    .bind(&req.cite)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    // 更新会话的 updated_at
    sqlx::query(r#"UPDATE openai_sessions SET updated_at = NOW() WHERE id = $1"#)
        .bind(&session_id)
        .execute(&pool)
        .await
        .ok();

    Ok(ApiResponse::ok(message))
}

// 获取会话的消息历史
async fn get_session_messages(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> ApiResult<ApiResponse<Vec<OpenAiMessage>>> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    // 验证会话属于当前用户
    let _session = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM openai_sessions WHERE id = $1 AND user_id = $2"#,
    )
    .bind(&session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::not_found(ApiError::SESSION_NOT_FOUND, "会话不存在"))?;

    let messages = sqlx::query_as::<_, OpenAiMessage>(
        r#"SELECT * FROM openai_messages WHERE session_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(&session_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(messages))
}

pub fn openai_session_routes() -> Router {
    Router::new()
        .route("/", post(create_session))
        .route("/", get(list_sessions))
        .route("/{id}", get(get_session))
        .route("/{id}", delete(delete_session))
        .route("/{id}/messages", post(add_message))
        .route("/{id}/messages", get(get_session_messages))
}
