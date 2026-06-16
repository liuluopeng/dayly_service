use crate::api::base::{ApiError, ApiResponse, ApiResult};
use crate::api::client::ApiClient;
use my_type::model::short_notes::ShortNote;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建短笔记请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShortNoteRequest {
    pub content: Option<String>,
    pub view_name: Option<String>,
}

/// 更新短笔记请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateShortNoteRequest {
    pub content: Option<String>,
    pub view_name: Option<String>,
}

/// 获取短笔记列表
pub async fn list_short_notes(
    client: &ApiClient,
    page: Option<u32>,
    page_size: Option<u32>,
) -> ApiResult<ApiResponse<Vec<ShortNote>>> {
    let mut path = "/api/short_notes/list".to_string();

    // 构建查询参数
    let mut params = Vec::new();
    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(ps) = page_size {
        params.push(format!("page_size={}", ps));
    }

    if !params.is_empty() {
        path.push_str("?");
        path.push_str(&params.join("&"));
    }

    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status,
            body
        )));
    }

    let api_response: ApiResponse<Vec<ShortNote>> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

/// 获取单个短笔记
pub async fn get_short_note(client: &ApiClient, id: Uuid) -> ApiResult<ApiResponse<ShortNote>> {
    let path = format!("/api/short_notes/get/{}", id);

    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status,
            body
        )));
    }

    let api_response: ApiResponse<ShortNote> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

/// 创建短笔记
pub async fn create_short_note(
    client: &ApiClient,
    req: CreateShortNoteRequest,
) -> ApiResult<ApiResponse<ShortNote>> {
    let response = client
        .post("/api/short_notes/create", req)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status,
            body
        )));
    }

    let api_response: ApiResponse<ShortNote> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

/// 更新短笔记
pub async fn update_short_note(
    client: &ApiClient,
    id: Uuid,
    req: UpdateShortNoteRequest,
) -> ApiResult<ApiResponse<ShortNote>> {
    let path = format!("/api/short_notes/update/{}", id);

    let response = client
        .put(&path, req)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status,
            body
        )));
    }

    let api_response: ApiResponse<ShortNote> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

/// 删除短笔记
pub async fn delete_short_note(client: &ApiClient, id: Uuid) -> ApiResult<ApiResponse<()>> {
    let path = format!("/api/short_notes/delete/{}", id);

    let response = client
        .delete(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status,
            body
        )));
    }

    let api_response: ApiResponse<()> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}
