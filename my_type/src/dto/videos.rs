use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::model::videos::Video;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoWithUrl {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub serve_url: Option<String>,
    pub folder_path: String,
    pub size: i64,
    pub duration_ms: Option<i64>,
    pub format: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl From<Video> for VideoWithUrl {
    fn from(vid: Video) -> Self {
        Self {
            id: vid.id,
            name: vid.name,
            path: vid.path,
            serve_url: None,
            folder_path: vid.folder_path,
            size: vid.size,
            duration_ms: vid.duration_ms,
            format: vid.format,
            width: vid.width,
            height: vid.height,
        }
    }
}

impl VideoWithUrl {
    pub fn set_serve_url(&mut self, base_url: &str, path: &str) {
        self.serve_url = Some(format!(
            "{}/api/files/serve?path={}",
            base_url,
            urlencoding::encode(path)
        ));
    }
}

crate::impl_display!(self, VideoWithUrl,
    "ID"   => self.id,
    "名称" => self.name,
    "大小" => self.size,
    "分辨率" => match (self.width, self.height) {
        (Some(w), Some(h)) => format!("{}x{}", w, h),
        _ => "未知".to_string(),
    },
    "时长" => match self.duration_ms {
        Some(d) => format!("{}ms", d),
        None => "未知".to_string(),
    }
);
