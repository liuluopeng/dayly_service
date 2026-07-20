use flutter_rust_bridge::frb;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};

pub use common::api::{
    base::ApiError,
    clipboard::{get_clipboard_history, ClipboardEntry},
};

pub async fn get_clipboard_history_for_dart(
    count: Option<i64>,
    type_filter: Option<String>,
    search: Option<String>,
) -> Result<Vec<ClipboardEntry>, ApiError> {
    log_to_dart(format!(
        "剪贴板历史: count={}, type={:?}, search={:?}",
        count.unwrap_or(20),
        type_filter,
        search,
    ));
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let entries = get_clipboard_history(
        &client,
        count.map(|c| c as usize),
        type_filter.as_deref(),
        search.as_deref(),
    )
    .await?;
    log_to_dart(format!("剪贴板历史: 返回 {} 条", entries.len()));
    Ok(entries)
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
