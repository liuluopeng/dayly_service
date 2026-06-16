use my_type::model::openai::{OpenAiMessage, OpenAiSession};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use anyhow;

use super::base::{ApiResponse, ApiResult};
use super::client::ApiClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    pub think: Option<String>,
    pub cite: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionWithMessages {
    pub session: OpenAiSession,
    pub messages: Vec<OpenAiMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

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
