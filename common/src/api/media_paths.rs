use super::base::{ApiError, ApiResult};
use super::client::ApiClient;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct AddMediaPathRequest {
    pub directory_id: String,
    pub media_type: String,
    pub path: String,
    pub label: Option<String>,
}

pub async fn list_media_paths(
    client: &ApiClient,
    media_type: Option<&str>,
) -> ApiResult<Value> {
    let url = match media_type {
        Some(mt) => format!("/api/media_paths?media_type={}", mt),
        None => "/api/media_paths".to_string(),
    };
    let resp = client
        .get(&url)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn add_media_path(
    client: &ApiClient,
    directory_id: &str,
    media_type: &str,
    path: &str,
    label: Option<&str>,
) -> ApiResult<Value> {
    let body = AddMediaPathRequest {
        directory_id: directory_id.to_string(),
        media_type: media_type.to_string(),
        path: path.to_string(),
        label: label.map(|s| s.to_string()),
    };
    let resp = client
        .post("/api/media_paths", &body)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn delete_media_path(client: &ApiClient, path_id: &str) -> ApiResult<Value> {
    let resp = client
        .delete(&format!("/api/media_paths/{}", path_id))
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}
