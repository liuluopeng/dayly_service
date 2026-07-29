use flutter_rust_bridge::frb;

use crate::api::wifi_api::init::get_client_clone;

pub use common::api::{
    base::ApiError,
    clipboard::{get_clipboard_history, ClipboardEntry},
};

pub async fn get_clipboard_history_for_dart(
    count: Option<i64>,
    type_filter: Option<String>,
    search: Option<String>,
) -> Result<Vec<ClipboardEntry>, ApiError> {
    tracing::info!(
        count = count.unwrap_or(20),
        type_filter = ?type_filter,
        search = ?search,
        "查询剪贴板历史"
    );
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_clipboard_history(
        &client,
        count.map(|c| c as usize),
        type_filter.as_deref(),
        search.as_deref(),
    )
    .await
    {
        Ok(entries) => {
            tracing::info!(count = entries.len(), "剪贴板历史返回");
            Ok(entries)
        }
        Err(e) => {
            tracing::error!(error = %e, "剪贴板历史获取失败");
            Err(ApiError::Internal(format!("{}", e)))
        }
    }
}

#[frb(mirror(ClipboardEntry))]
pub struct _ClipboardEntry {
    pub id: i64,
    pub entry_type: String,
    pub text_content: Option<String>,
    pub content_hash: String,
    pub created_at: String,
    pub image_url: Option<String>,
    pub image_path: Option<String>,
}
