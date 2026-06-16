use super::client::ApiClient;
use my_type::dto::NoteSummary;
use serde::{Deserialize, Serialize};
use urlencoding;

use crate::api::base::{ApiError, ApiResponse, ApiResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveNoteRequest {
    pub id: Option<String>,
    pub text: String,
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveNoteResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub text: String,
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteResponse {
    pub id: String,
    pub message: String,
}

/// 保存笔记（支持创建和更新）
pub async fn save_note(
    client: &ApiClient,
    req: &SaveNoteRequest,
) -> ApiResult<ApiResponse<SaveNoteResponse>> {
    let response = client
        .post("/api/note/save", req)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let result = response
        .json::<ApiResponse<SaveNoteResponse>>()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(result)
}

/// 创建笔记
pub async fn create_note(
    client: &ApiClient,
    req: &CreateNoteRequest,
) -> ApiResult<ApiResponse<CreateNoteResponse>> {
    let response = client
        .post("/api/note", req)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let result = response
        .json::<ApiResponse<CreateNoteResponse>>()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(result)
}

/// 获取笔记列表
pub async fn list_notes(
    client: &ApiClient,
    page: Option<u32>,
    limit: Option<u32>,
) -> ApiResult<ApiResponse<Vec<NoteSummary>>> {
    let mut path = "/api/note".to_string();
    let mut params = Vec::new();

    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }

    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }

    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let notes = response
        .json::<ApiResponse<Vec<NoteSummary>>>()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(notes)
}

/// 获取笔记详情
pub async fn get_note(client: &ApiClient, uuid: &str) -> ApiResult<ApiResponse<NoteSummary>> {
    let path = format!("/api/note/{}", uuid);
    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let note = response
        .json::<ApiResponse<NoteSummary>>()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(note)
}

/// 搜索笔记
pub async fn search_notes(
    client: &ApiClient,
    query: &str,
) -> ApiResult<ApiResponse<Vec<NoteSummary>>> {
    let path = format!("/api/note/search?query={}", urlencoding::encode(query));
    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let notes = response
        .json::<ApiResponse<Vec<NoteSummary>>>()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(notes)
}
