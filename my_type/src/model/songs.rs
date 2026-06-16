use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_path: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub cover_data: Option<Vec<u8>>,
    pub media_path_id: Option<Uuid>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub lrc: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub ttml: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub eslrc: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub qrc: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub yrc: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub lys: Option<String>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub vocal: Option<Vec<u8>>,
    #[cfg_attr(not(target_arch = "wasm32"), sqlx(default))]
    pub auto_ttml: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SongInfo {
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub size: u64,
}

crate::impl_display!(self, Song,
    "ID"     => self.id,
    "标题"   => self.title,
    "专辑"   => self.album.as_deref().unwrap_or("无"),
    "艺术家" => self.artist.as_deref().unwrap_or("无")
);

crate::impl_display!(self, SongInfo,
    "ID"     => self.id,
    "标题"   => self.title,
    "大小"   => self.size
);
