use my_type::model::openai::{OpenAiMessage, OpenAiSession};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::base::{ApiResponse, ApiResult};
use super::client::ApiClient;

#[derive(Debug, Deserialize, Serialize)]
/// 创建会话的请求体
#[allow(missing_docs)]
pub struct CreateSessionRequest {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// 添加消息的请求体
#[allow(missing_docs)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    pub think: Option<String>,
    pub cite: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
/// 会话及消息列表
#[allow(missing_docs)]
pub struct SessionWithMessages {
    pub session: OpenAiSession,
    pub messages: Vec<OpenAiMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
/// 聊天补全请求体
#[allow(missing_docs)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
/// 聊天消息（角色+内容）
#[allow(missing_docs)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// 创建新的 OpenAI 会话
pub async fn create_session(
    client: &ApiClient,
    title: &str,
) -> ApiResult<ApiResponse<OpenAiSession>> {
    let req = CreateSessionRequest {
        title: title.to_string(),
    };
    let response = client
        .post("/api/openai/sessions", &req)
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<OpenAiSession>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 列出当前用户的会话
pub async fn list_sessions(client: &ApiClient) -> ApiResult<ApiResponse<Vec<OpenAiSession>>> {
    let response = client
        .get("/api/openai/sessions")
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<Vec<OpenAiSession>>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 获取会话详情
pub async fn get_session(
    client: &ApiClient,
    session_id: &Uuid,
) -> ApiResult<ApiResponse<SessionWithMessages>> {
    let response = client
        .get(&format!("/api/openai/sessions/{}", session_id))
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<SessionWithMessages>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 删除会话
pub async fn delete_session(client: &ApiClient, session_id: &Uuid) -> ApiResult<ApiResponse<()>> {
    let response = client
        .delete(&format!("/api/openai/sessions/{}", session_id))
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<()>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 向会话添加消息
pub async fn add_message(
    client: &ApiClient,
    session_id: &Uuid,
    role: &str,
    content: &str,
    think: Option<&str>,
    cite: Option<Value>,
) -> ApiResult<ApiResponse<OpenAiMessage>> {
    let req = AddMessageRequest {
        role: role.to_string(),
        content: content.to_string(),
        think: think.map(|s| s.to_string()),
        cite,
    };
    let response = client
        .post(
            &format!("/api/openai/sessions/{}/messages", session_id),
            &req,
        )
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<OpenAiMessage>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 获取会话的消息列表
pub async fn get_session_messages(
    client: &ApiClient,
    session_id: &Uuid,
) -> ApiResult<ApiResponse<Vec<OpenAiMessage>>> {
    let response = client
        .get(&format!("/api/openai/sessions/{}/messages", session_id))
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<Vec<OpenAiMessage>>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

/// 发起聊天补全请求
pub async fn chat_completion(
    client: &ApiClient,
    req: &ChatCompletionRequest,
) -> ApiResult<ApiResponse<serde_json::Value>> {
    let response = client
        .post("/api/openai/chat/completions", req)
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json::<ApiResponse<serde_json::Value>>()
        .await
        .map_err(|e| super::base::ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}
