use my_type::model::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::base::{ApiResponse, ApiResult};
use super::client::ApiClient;

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessageWithUsername {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Local>,
}

pub async fn send_message(
    client: &ApiClient,
    content: &str,
) -> ApiResult<ApiResponse<ChatMessage>> {
    let req = SendMessageRequest {
        content: content.to_string(),
    };
    let response = client
        .post("/api/chat/messages", &req)
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<ChatMessage>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn get_messages(
    client: &ApiClient,
    after: Option<&str>,
) -> ApiResult<ApiResponse<Vec<ChatMessageWithUsername>>> {
    let url = match after {
        Some(ts) => format!("/api/chat/messages?after={}", ts),
        None => "/api/chat/messages".to_string(),
    };
    let response = client
        .get(&url)
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<Vec<ChatMessageWithUsername>>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecentContact {
    pub user_id: Uuid,
    pub username: String,
    pub last_message: String,
    pub last_message_at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContactItem {
    pub id: Uuid,
    pub username: String,
}

pub async fn get_recent_contacts(
    client: &ApiClient,
) -> ApiResult<ApiResponse<Vec<RecentContact>>> {
    let response = client
        .get("/api/chat/recent-contacts")
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<Vec<RecentContact>>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn get_contacts(
    client: &ApiClient,
) -> ApiResult<ApiResponse<Vec<ContactItem>>> {
    let response = client
        .get("/api/chat/contacts")
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<Vec<ContactItem>>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}
