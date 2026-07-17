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
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    get_clipboard_history(
        &client,
        count.map(|c| c as usize),
        type_filter.as_deref(),
        search.as_deref(),
    )
    .await
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
