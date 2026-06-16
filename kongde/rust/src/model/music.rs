use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Music {
    pub id: Option<i64>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub path: String,
    pub created_at: i64,
    pub cover_path: Option<String>,
    pub background_color: Option<i64>,
    pub secondary_color: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Singer {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistWithSongs {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub songs: Vec<Music>,
}
