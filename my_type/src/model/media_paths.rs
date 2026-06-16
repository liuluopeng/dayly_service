use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct MediaPath {
    pub id: Uuid,
    pub directory_id: Uuid,
    pub media_type: String,
    pub path: String,
    pub label: Option<String>,
    pub allow_list: Option<Vec<String>>,
    pub scan_when_start: Option<bool>,
    pub scan_when_change: Option<bool>,
    pub last_scan_time: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

crate::impl_display!(self, MediaPath,
    "ID"   => self.id,
    "类型" => self.media_type,
    "路径" => self.path,
    "标签" => self.label.as_deref().unwrap_or("无")
);
