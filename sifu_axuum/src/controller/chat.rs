use crate::middleware::Claims;
use axum::extract::Query;
use axum::{
    Extension, Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::{get, post},
};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use futures::{SinkExt, StreamExt};
use my_type::model::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    after: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ChatMessageWithUsername {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Local>,
}

async fn send_message(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Extension(chat_tx): Extension<broadcast::Sender<String>>,
    Json(req): Json<SendMessageRequest>,
) -> ApiResult<ApiResponse<ChatMessage>> {
    let sender_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let content = req.content.trim().to_string();
    if content.is_empty() {
        return Err(ApiError::bad_request("EMPTY_CONTENT", "消息不能为空"));
    }

    let message = sqlx::query_as::<_, ChatMessage>(
        r#"INSERT INTO chat_messages (sender_id, content) VALUES ($1, $2) RETURNING *"#,
    )
    .bind(sender_id)
    .bind(&content)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    // 广播消息给所有 WebSocket 客户端
    let ws_msg = ChatMessageWithUsername {
        id: message.id,
        sender_id: message.sender_id,
        username: claims.username.clone(),
        content: message.content.clone(),
        created_at: message.created_at,
    };
    if let Ok(json) = serde_json::to_string(&ws_msg) {
        let _ = chat_tx.send(json);
    }

    Ok(ApiResponse::ok(message))
}

/// .
///
/// # Errors
///
/// This function will return an error if .
async fn get_messages(
    _claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<GetMessagesQuery>,
) -> ApiResult<ApiResponse<Vec<ChatMessageWithUsername>>> {
    let messages = if let Some(after) = &query.after {
        sqlx::query_as::<_, ChatMessageWithUsername>(
            r#"SELECT m.id, m.sender_id, u.username, m.content, m.created_at
               FROM chat_messages m
               JOIN users u ON m.sender_id = u.id
               WHERE m.created_at > $1
               ORDER BY m.created_at ASC
               LIMIT 100"#,
        )
        .bind(after)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, ChatMessageWithUsername>(
            r#"SELECT m.id, m.sender_id, u.username, m.content, m.created_at
               FROM chat_messages m
               JOIN users u ON m.sender_id = u.id
               ORDER BY m.created_at DESC
               LIMIT 50"#,
        )
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(ApiResponse::ok(messages))
}

/// .
async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(chat_tx): Extension<broadcast::Sender<String>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, chat_tx))
}

async fn handle_socket(socket: WebSocket, chat_tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = chat_tx.subscribe();

    // 转发广播消息到 WebSocket 客户端
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // 接收客户端消息（目前只做心跳/忽略）
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(_) => {
                    // 客户端发来的文本消息，目前忽略
                    // 后续可用于发送消息（替代 POST）
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
    info!("WebSocket 连接关闭");
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct RecentContact {
    pub user_id: Uuid,
    pub username: String,
    pub last_message: String,
    pub last_message_at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ContactItem {
    pub id: Uuid,
    pub username: String,
}

async fn recent_contacts(
    _claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<RecentContact>>> {
    let contacts = sqlx::query_as::<_, RecentContact>(
        r#"SELECT DISTINCT ON (u.id)
               u.id AS user_id,
               u.username,
               m.content AS last_message,
               m.created_at AS last_message_at
           FROM chat_messages m
           JOIN users u ON m.sender_id = u.id
           ORDER BY u.id, m.created_at DESC"#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    let mut sorted = contacts;
    sorted.sort_by(|a, b| b.last_message_at.cmp(&a.last_message_at));
    Ok(ApiResponse::ok(sorted))
}

async fn contacts(
    _claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<ContactItem>>> {
    let users =
        sqlx::query_as::<_, ContactItem>("SELECT id, username FROM users ORDER BY username")
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                error!("Database error: {:?}", e);
                ApiError::Internal(e.to_string())
            })?;

    Ok(ApiResponse::ok(users))
}

pub fn chat_routes() -> Router {
    Router::new()
        .route("/messages", post(send_message))
        .route("/messages", get(get_messages))
        .route("/recent-contacts", get(recent_contacts))
        .route("/contacts", get(contacts))
        .route("/ws", get(ws_handler))
}
