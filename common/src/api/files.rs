use serde::{Deserialize, Serialize};

use super::base::{ApiError, ApiResult};
use super::client::ApiClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: Option<String>,
    pub content_type: String,
}

/// 列出目录内容
pub async fn list_files(client: &ApiClient, path: &str, page: Option<usize>, page_size: Option<usize>) -> ApiResult<DirListing> {
    let mut url = format!("/api/files/list?path={}", urlencoding::encode(path));
    if let Some(p) = page {
        url.push_str(&format!("&page={}", p));
    }
    if let Some(ps) = page_size {
        url.push_str(&format!("&page_size={}", ps));
    }
    let response = client
        .get(&url)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))
}

/// 获取文件元数据
pub async fn get_file_info(client: &ApiClient, path: &str) -> ApiResult<FileInfo> {
    let url = format!("/api/files/info?path={}", urlencoding::encode(path));
    let response = client
        .get(&url)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))
}

/// 对目录生成树形文件
pub async fn generate_tree(client: &ApiClient, path: &str) -> ApiResult<serde_json::Value> {
    let body = serde_json::json!({ "path": path });
    let response = client
        .post("/api/files/tree", body)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))
}

/// 构建文件下载 URL（带 token）
pub fn build_file_url(client: &ApiClient, path: &str) -> String {
    let encoded = urlencoding::encode(path);
    match &client.token {
        Some(token) => format!(
            "{}/api/files/serve?path={}&token={}",
            client.base_url, encoded, token
        ),
        None => format!("{}/api/files/serve?path={}", client.base_url, encoded),
    }
}
