use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct Image {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub folder_path: String,
    pub media_path_id: Uuid,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

crate::impl_display!(self, Image,
    "ID"   => self.id,
    "名称" => self.name,
    "大小" => self.size
);
