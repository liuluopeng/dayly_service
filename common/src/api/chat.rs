use my_type::model::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::base::{ApiResponse, ApiResult};
use super::client::ApiClient;

#[derive(Debug, Serialize)]
/// 发送聊天消息的请求体
pub struct SendMessageRequest {
    /// 消息内容
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// 聊天消息（附带发送者用户名）
pub struct ChatMessageWithUsername {
    /// 消息 ID
    pub id: Uuid,
    /// 发送者用户 ID
    pub sender_id: Uuid,
    /// 发送者用户名
    pub username: String,
    /// 消息内容
    pub content: String,
    /// 发送时间
    pub created_at: chrono::DateTime<chrono::Local>,
}

/// 发送一条聊天消息
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

/// 获取聊天消息（可按时间游标增量拉取）
pub async fn get_messages(
    client: &ApiClient,
    after: Option<&str>,
) -> ApiResult<ApiResponse<Vec<ChatMessageWithUsername>>> {
    let url = match after {
        Some(ts) => format!("/api/chat/messages?after={}", urlencoding::encode(ts)),
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
/// 最近联系人条目
#[allow(missing_docs)]
pub struct RecentContact {
    pub user_id: Uuid,
    pub username: String,
    pub last_message: String,
    pub last_message_at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize, Serialize)]
/// 联系人条目
#[allow(missing_docs)]
pub struct ContactItem {
    pub id: Uuid,
    pub username: String,
}

/// 获取最近联系人列表
pub async fn get_recent_contacts(client: &ApiClient) -> ApiResult<ApiResponse<Vec<RecentContact>>> {
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

/// 获取全部联系人列表
pub async fn get_contacts(client: &ApiClient) -> ApiResult<ApiResponse<Vec<ContactItem>>> {
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
