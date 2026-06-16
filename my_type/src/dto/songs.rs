use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::model::songs::Song;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SongWithUrl {
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_path: Option<String>,
    pub cover_url: Option<String>,
}
impl From<Song> for SongWithUrl {
    fn from(song: Song) -> Self {
        Self {
            id: song.id,
            title: song.title,
            path: song.path,
            album: song.album,
            artist: song.artist,
            cover_path: song.cover_path,
            cover_url: None, // 初始化为 None，后续可以通过方法设置
        }
    }
}

impl SongWithUrl {
    pub fn set_cover_url(&mut self, base_url: &str) {
        // 封面现在存储在数据库中，生成访问封面的URL
        self.cover_url = Some(format!("{}/api/songs/cover/{}", base_url, self.id));
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LyricsResponse {
    pub song_id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub lyrics: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllLyricsResponse {
    pub song_id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub lrc: Option<String>,
    pub ttml: Option<String>,
    pub eslrc: Option<String>,
    pub qrc: Option<String>,
    pub yrc: Option<String>,
    pub lys: Option<String>,
    pub auto_ttml: Option<String>,
}

crate::impl_display!(self, SongWithUrl,
    "ID"       => self.id,
    "标题"     => self.title,
    "路径"     => self.path,
    "专辑"     => self.album.as_deref().unwrap_or("无"),
    "艺术家"   => self.artist.as_deref().unwrap_or("无"),
    "封面路径" => self.cover_path.as_deref().unwrap_or("无"),
    "封面URL"  => self.cover_url.as_deref().unwrap_or("无")
);

crate::impl_display!(self, LyricsResponse,
    "歌曲ID" => self.song_id,
    "标题"   => self.title,
    "艺术家" => self.artist.as_deref().unwrap_or("无"),
    "歌词长度" => format!("{} 字符", self.lyrics.len())
);

crate::impl_display!(self, AllLyricsResponse,
    "歌曲ID" => self.song_id,
    "标题"   => self.title,
    "艺术家" => self.artist.as_deref().unwrap_or("无"),
    "可用格式" => {
        let types = [
            ("LRC", &self.lrc),
            ("TTML", &self.ttml),
            ("ESLRC", &self.eslrc),
            ("QRC", &self.qrc),
            ("YRC", &self.yrc),
            ("LYS", &self.lys),
            ("自动TTML", &self.auto_ttml),
        ];
        let available: Vec<&str> = types.iter().filter(|(_, v)| v.is_some()).map(|(k, _)| *k).collect();
        available.join(", ")
    }
);
