use super::base::{ApiError, ApiResult};
use super::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClipboardEntry {
    pub id: i64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub text_content: Option<String>,
    pub content_hash: String,
    pub created_at: String,
    pub image_url: Option<String>,
    pub image_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClipboardResponse {
    pub code: i32,
    pub data: Option<Vec<ClipboardEntry>>,
    pub message: Option<String>,
}

pub async fn get_clipboard_history(
    client: &ApiClient,
    count: Option<usize>,
    type_filter: Option<&str>,
    search: Option<&str>,
) -> ApiResult<Vec<ClipboardEntry>> {
    let mut path = format!("/api/clipboard/history?count={}", count.unwrap_or(20));
    if let Some(t) = type_filter {
        path.push_str(&format!("&type={}", t));
    }
    if let Some(s) = search {
        path.push_str(&format!("&search={}", s));
    }

    let response = client
        .get(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!("HTTP {}: {}", status, body)));
    }

    let resp: ClipboardResponse = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("{}", e)))?;

    if resp.code != 200 {
        return Err(ApiError::Internal(
            resp.message.unwrap_or_else(|| "未知错误".into()),
        ));
    }

    Ok(resp.data.unwrap_or_default())
}
