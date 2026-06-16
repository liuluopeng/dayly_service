use super::base::{ApiError, ApiResult};
use super::client::ApiClient;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct AddUserDirRequest {
    pub path: String,
    pub label: Option<String>,
    pub allow_list: Option<Vec<String>>,
}

pub async fn list_users(client: &ApiClient) -> ApiResult<Value> {
    let resp = client
        .get("/api/user/list")
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn list_user_directories(client: &ApiClient, username: Option<&str>) -> ApiResult<Value> {
    let url = match username {
        Some(u) => format!("/api/admin/user-directories?username={}", u),
        None => "/api/admin/user-directories".to_string(),
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

pub async fn add_user_directory(
    client: &ApiClient,
    path: &str,
    label: Option<&str>,
    allow_list: Option<Vec<String>>,
) -> ApiResult<Value> {
    let body = AddUserDirRequest {
        path: path.to_string(),
        label: label.map(|s| s.to_string()),
        allow_list,
    };
    let resp = client
        .post("/api/admin/user-directories", &body)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}

pub async fn delete_user_directory(client: &ApiClient, dir_id: &str) -> ApiResult<Value> {
    let resp = client
        .delete(&format!("/api/admin/user-directories/{}", dir_id))
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    let data = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    Ok(data)
}
