use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::model::images::Image;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageWithUrl {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub serve_url: Option<String>,
    pub folder_path: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
}

impl From<Image> for ImageWithUrl {
    fn from(img: Image) -> Self {
        Self {
            id: img.id,
            name: img.name,
            path: img.path,
            serve_url: None,
            folder_path: img.folder_path,
            size: img.size,
            width: img.width,
            height: img.height,
            format: img.format,
        }
    }
}

impl ImageWithUrl {
    pub fn set_serve_url(&mut self, base_url: &str, path: &str) {
        self.serve_url = Some(format!(
            "{}/api/files/serve?path={}",
            base_url,
            urlencoding::encode(path)
        ));
    }
}

crate::impl_display!(self, ImageWithUrl,
    "ID"   => self.id,
    "名称" => self.name,
    "大小" => self.size,
    "分辨率" => match (self.width, self.height) {
        (Some(w), Some(h)) => format!("{}x{}", w, h),
        _ => "未知".to_string(),
    }
);
