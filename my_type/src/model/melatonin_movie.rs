use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MelatoninMovie {
    pub id: Uuid,
    pub title: String,
    pub cover_path: String,
    pub video_paths: Vec<String>,
    pub nfo_json: Value,
    pub cover_url: Option<String>,
    pub video_urls: Vec<String>,
    pub preview_urls: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MelatoninMetadata {
    #[serde(rename = "title")]
    pub title: Option<String>,
    #[serde(rename = "plot")]
    pub plot: Option<String>,
    #[serde(rename = "outline")]
    pub outline: Option<String>,
    #[serde(rename = "tagline")]
    pub tagline: Option<String>,
    #[serde(rename = "runtime")]
    pub runtime: Option<String>,
    #[serde(rename = "thumb")]
    pub thumb: Option<String>,
    #[serde(rename = "fanart")]
    pub fanart: Option<String>,
    #[serde(rename = "premiered")]
    pub premiered: Option<String>,
    #[serde(rename = "year")]
    pub year: Option<String>,
    #[serde(rename = "rating")]
    pub rating: Option<String>,
    #[serde(rename = "votes")]
    pub votes: Option<String>,
    #[serde(rename = "mpaa")]
    pub mpaa: Option<String>,
    #[serde(rename = "playcount")]
    pub playcount: Option<String>,
    #[serde(rename = "lastplayed")]
    pub lastplayed: Option<String>,
    #[serde(rename = "genre")]
    pub genre: Option<Vec<String>>,
    #[serde(rename = "director")]
    pub director: Option<Vec<String>>,
    #[serde(rename = "actor")]
    pub actor: Option<Vec<Actor>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Actor {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "role")]
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MelatoninMovieList {
    pub id: Uuid,
    pub title: String,
    pub cover_path: String,
    pub video_paths: Vec<String>,
    pub cover_url: Option<String>,
    pub video_urls: Vec<String>,
}

crate::impl_display!(self, MelatoninMovie,
    "ID"   => self.id,
    "标题" => self.title
);

crate::impl_display!(self, MelatoninMetadata,
    "标题" => self.title.as_deref().unwrap_or("无"),
    "年份" => self.year.as_deref().unwrap_or("无"),
    "评分" => self.rating.as_deref().unwrap_or("无")
);

crate::impl_display!(self, Actor,
    "演员" => self.name,
    "角色" => self.role.as_deref().unwrap_or("无")
);

crate::impl_display!(self, MelatoninMovieList,
    "ID"       => self.id,
    "标题"     => self.title,
    "封面路径" => self.cover_path,
    "视频数量" => self.video_paths.len(),
    "封面URL"  => self.cover_url.as_deref().unwrap_or("无"),
    "视频URL"  => self.video_urls.len()
);
