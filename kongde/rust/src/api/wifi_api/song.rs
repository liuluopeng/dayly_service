use flutter_rust_bridge::frb;
use uuid::Uuid;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};

pub use common::api::{
    base::{ApiError, ApiResponse, PaginatedResponse},
    client::ApiClient,
    songs::{SongWithUrl, get_all_songs, get_all_lyrics, get_song_file},
};

pub struct SongWithUrlForDart {
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_path: Option<String>,
    pub cover_url: Option<String>,
}

pub struct SongPaginatedResponseForDart {
    pub data: Vec<SongWithUrlForDart>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: i64,
}

pub async fn get_all_songs_for_dart(
    page: u32,
    page_size: u32,
) -> Result<SongPaginatedResponseForDart, ApiError> {
    log_to_dart(format!("song 启动 page {} page_size {}", page, page_size));

    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_all_songs(&client, Some(page), Some(page_size)).await {
        Ok(songs) => {
            if let Some(songs) = songs.data {
                let total = songs.total;
                let data = songs
                    .data
                    .into_iter()
                    .map(|song| SongWithUrlForDart {
                        id: song.id,
                        title: song.title,
                        path: song.path,
                        album: song.album,
                        artist: song.artist,
                        cover_path: song.cover_path,
                        cover_url: song.cover_url,
                    })
                    .collect();
                log_to_dart(format!("歌曲列表: page={}, 共 {} 首", page, total));
                Ok(SongPaginatedResponseForDart {
                    data,
                    total: songs.total,
                    page: songs.page,
                    page_size: songs.page_size,
                    total_pages: songs.total_pages,
                })
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_song_file_for_dart(song_id: Uuid) -> Result<Vec<u8>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_song_file(&client, &song_id).await {
        Ok(res) => {
            if let Some(data) = res.data {
                Ok(data)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub struct AllLyricsResponseForDart {
    pub song_id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub lrc: Option<String>,
    pub ttml: Option<String>,
    pub eslrc: Option<String>,
    pub qrc: Option<String>,
    pub yrc: Option<String>,
    pub lys: Option<String>,
}

pub async fn get_song_lyrics_for_dart(song_id: Uuid) -> Result<AllLyricsResponseForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_all_lyrics(&client, &song_id).await {
        Ok(res) => {
            if let Some(data) = res.data {
                Ok(AllLyricsResponseForDart {
                    song_id: data.song_id,
                    title: data.title,
                    artist: data.artist,
                    lrc: data.lrc,
                    ttml: data.ttml,
                    eslrc: data.eslrc,
                    qrc: data.qrc,
                    yrc: data.yrc,
                    lys: data.lys,
                })
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}
