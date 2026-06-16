use crate::config::env::ServerConfig;
use crate::middleware::Claims;
use anyhow;
use axum::extract::{Extension, Path as AxumPath, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::api::base::{ApiError, ApiResult};
use my_type::model::songs::Song;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tracing::{debug, error, info, warn};

use common::api::songs::{LyricsResponse, SongWithUrl};
use my_type::dto::songs::AllLyricsResponse;

use common::api::base::{ApiResponse, PaginatedResponse};

use common::read_metadata;
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::fs;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct AlbumQuery {
    album: String,
}

#[derive(Debug, Deserialize)]
pub struct ArtistQuery {
    artist: String,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct ScanQuery {
    pub media_path_id: Option<Uuid>,
}

// 扫描歌曲 — 自动扫描用户所有 media_type='song' 的媒体路径
pub async fn scan_songs(
    claims: Claims,
    Query(query): Query<ScanQuery>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<Json<serde_json::Value>> {
    // 查询该用户 song 类型的媒体路径（可指定单个）
    let media_paths: Vec<(Uuid, String)> = if let Some(mp_id) = query.media_path_id {
        sqlx::query_as(
            "SELECT id, path FROM media_paths WHERE id = $1 AND $2 = ANY(allow_list) AND media_type = 'song'"
        )
        .bind(mp_id)
        .bind(&claims.username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?
    } else {
        sqlx::query_as(
            "SELECT id, path FROM media_paths WHERE $1 = ANY(allow_list) AND media_type = 'song'"
        )
        .bind(&claims.username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?
    };

    if media_paths.is_empty() {
        return Ok(Json(json!({
            "code": 200,
            "message": "未配置歌曲目录，请先添加媒体路径",
            "data": null
        })));
    }

    let pg_pool_clone = pg_pool.clone();

    tokio::spawn(async move {
        let (songs_to_insert, mut errors) = tokio::task::spawn_blocking(move || {
            let mut songs_to_insert = Vec::new();
            let mut errors = Vec::new();

            fn scan_directory(
                path: &Path,
                media_path_id: Uuid,
                songs: &mut Vec<(
                    Uuid, String, String, Uuid,
                    Option<String>, Option<String>, Option<String>, Option<Vec<u8>>,
                    Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
                )>,
                errors: &mut Vec<String>,
            ) {
                for entry in WalkDir::new(path)
                    .max_depth(100)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let current_path = entry.path();
                    if !current_path.is_file() {
                        continue;
                    }
                    let Some(ext) = current_path.extension() else { continue };
                    let ext = ext.to_string_lossy().to_lowercase();
                    if !matches!(ext.as_str(), "mp3"|"flac"|"wav"|"ogg"|"aac"|"m4a"|"wma"|"opus"|"webm") {
                        continue;
                    }

                    let filename = current_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let abs_path = current_path.to_string_lossy().to_string();

                    let metadata_result = read_metadata::read_metadata(&abs_path);

                    let (title, artist, album, cover_path, cover_data) = match metadata_result {
                        Ok(metadata) => {
                            let (cover_path, cover_data) = if let Some(picture) = metadata.picture {
                                (None, Some(picture))
                            } else {
                                (None, None)
                            };
                            (
                                metadata.title.unwrap_or(filename),
                                metadata.artist,
                                metadata.album,
                                cover_path,
                                cover_data,
                            )
                        }
                        Err(e) => {
                            errors.push(format!("读取元数据失败: {}, 文件: {}", e, abs_path));
                            (filename, None, None, None, None)
                        }
                    };

                    // 读取同名歌词文件
                    let read_lyric = |ext: &str| -> Option<String> {
                        let path = current_path.with_extension(ext);
                        fs::read_to_string(&path).ok()
                    };

                    let lrc_content = read_lyric("lrc");
                    let ttml_content = read_lyric("ttml");
                    let eslrc_content = read_lyric("eslrc");
                    let qrc_content = read_lyric("qrc");
                    let yrc_content = read_lyric("yrc");
                    let lys_content = read_lyric("lys");

                    songs.push((
                        Uuid::now_v7(),
                        title,
                        abs_path,
                        media_path_id,
                        artist,
                        album,
                        cover_path,
                        cover_data,
                        lrc_content,
                        ttml_content,
                        eslrc_content,
                        qrc_content,
                        yrc_content,
                        lys_content,
                    ));
                }
            }

            for (mp_id, mp_path) in &media_paths {
                let dir = Path::new(mp_path);
                if !dir.exists() || !dir.is_dir() {
                    errors.push(format!("媒体路径不存在或不是目录: {}", mp_path));
                    continue;
                }
                scan_directory(dir, *mp_id, &mut songs_to_insert, &mut errors);
            }

            (songs_to_insert, errors)
        })
        .await
        .expect("扫描目录失败");

        errors.iter().for_each(|error| error!("{}", error));

        let mut transaction = match pg_pool_clone.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("开始事务失败: {}", e);
                return;
            }
        };

        let mut inserted_count = 0;

        for (id, title, song_path, media_path_id, artist, album, cover_path, cover_data, lrc_content, ttml_content, eslrc_content, qrc_content, yrc_content, lys_content) in songs_to_insert {
            let song_path_clone = song_path.clone();
            if let Err(e) = sqlx::query(
                "INSERT INTO songs (id, title, path, artist, album, cover_path, cover_data, media_path_id, lrc, ttml, eslrc, qrc, yrc, lys)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                 ON CONFLICT (id) DO NOTHING"
            )
            .bind(id)
            .bind(title)
            .bind(song_path)
            .bind(artist)
            .bind(album)
            .bind(cover_path)
            .bind(cover_data)
            .bind(media_path_id)
            .bind(lrc_content)
            .bind(ttml_content)
            .bind(eslrc_content)
            .bind(qrc_content)
            .bind(yrc_content)
            .bind(lys_content)
            .execute(&mut *transaction)
            .await
            {
                errors.push(format!("数据库插入失败: {}, 文件: {}", e, song_path_clone));
            } else {
                inserted_count += 1;
            }
        }

        if let Err(e) = transaction.commit().await {
            error!("提交事务失败: {}", e);
            return;
        }

        if errors.is_empty() {
            info!("成功扫描并插入 {} 首歌曲", inserted_count);
        } else {
            warn!("成功插入 {} 首歌曲，{} 个错误", inserted_count, errors.len());
        }

        // 扫描完成后，自动生成逐字 TTML
        let pg_pool_for_ttml = pg_pool_clone.clone();
        tokio::spawn(async move {
            generate_auto_ttml(pg_pool_for_ttml).await;
        });
    });

    Ok(Json(json!({
        "code": 200,
        "message": "正在处理扫描请求，请稍后查看结果",
        "data": null
    })))
}

// ─── 自动 TTML 生成（已移除 Python 依赖，使用 dummy 占位）───

/// Dummy: 已移除 Demucs Python 依赖
fn run_demucs(_song_path: &str) -> Option<Vec<u8>> {
    None
}

/// Dummy: 已移除 align.py Python 依赖
fn run_align(_lrc_text: &str, _vocal_wav: &[u8]) -> Option<String> {
    None
}

/// 已移除 Python 依赖，不再自动生成 TTML
async fn generate_auto_ttml(_pg_pool: PgPool) {
    info!("自动 TTML 生成功能已禁用（已移除 Python 依赖）");
}

// 获取所有歌曲（分页，按用户媒体路径过滤）
pub async fn get_all_songs(
    claims: Claims,
    Query(query): Query<PageQuery>,
    Extension(pg_pool): Extension<PgPool>,
    Extension(server_config): Extension<ServerConfig>,
) -> ApiResult<ApiResponse<PaginatedResponse<SongWithUrl>>> {
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM songs WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))"
    )
    .bind(&claims.username)
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    let offset = (query.page - 1) * query.page_size;

    let songs = sqlx::query_as::<_, Song>(
        "SELECT id, title, path, album, artist, cover_path, media_path_id FROM songs
         WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
         ORDER BY (
            (CASE WHEN lrc IS NOT NULL THEN 1 ELSE 0 END) +
            (CASE WHEN ttml IS NOT NULL THEN 1 ELSE 0 END) +
            (CASE WHEN eslrc IS NOT NULL THEN 1 ELSE 0 END) +
            (CASE WHEN qrc IS NOT NULL THEN 1 ELSE 0 END) +
            (CASE WHEN yrc IS NOT NULL THEN 1 ELSE 0 END) +
            (CASE WHEN lys IS NOT NULL THEN 1 ELSE 0 END)
         ) DESC, title ASC LIMIT $2 OFFSET $3"
    )
    .bind(&claims.username)
    .bind(query.page_size as i64)
    .bind(offset as i64)
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    let mut songs_with_url: Vec<SongWithUrl> = songs.into_iter().map(SongWithUrl::from).collect();

    let base_url = server_config.get_base_url();
    for song in &mut songs_with_url {
        song.set_cover_url(&base_url);
    }

    let paginated_response = PaginatedResponse {
        data: songs_with_url,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages: (total + query.page_size as i64 - 1) / query.page_size as i64,
    };

    Ok(ApiResponse::ok(paginated_response))
}

// 根据专辑获取歌曲
pub async fn get_songs_by_album(
    claims: Claims,
    Query(query): Query<AlbumQuery>,
    Extension(pg_pool): Extension<PgPool>,
    Extension(server_config): Extension<ServerConfig>,
) -> ApiResult<ApiResponse<Vec<SongWithUrl>>> {
    let songs = sqlx::query_as::<_, Song>(
        "SELECT * FROM songs WHERE album = $1 AND media_path_id IN (SELECT id FROM media_paths WHERE $2 = ANY(allow_list))"
    )
        .bind(query.album)
        .bind(&claims.username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    let mut songs_with_url: Vec<SongWithUrl> = songs.into_iter().map(SongWithUrl::from).collect();

    let base_url = server_config.get_base_url();
    for song in &mut songs_with_url {
        song.set_cover_url(&base_url);
    }

    Ok(ApiResponse::ok(songs_with_url))
}

// 根据艺术家获取歌曲
pub async fn get_songs_by_artist(
    claims: Claims,
    Query(query): Query<ArtistQuery>,
    Extension(pg_pool): Extension<PgPool>,
    Extension(server_config): Extension<ServerConfig>,
) -> ApiResult<ApiResponse<Vec<SongWithUrl>>> {
    let songs = sqlx::query_as::<_, Song>(
        "SELECT * FROM songs WHERE artist = $1 AND media_path_id IN (SELECT id FROM media_paths WHERE $2 = ANY(allow_list))"
    )
        .bind(query.artist)
        .bind(&claims.username)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    let mut songs_with_url: Vec<SongWithUrl> = songs.into_iter().map(SongWithUrl::from).collect();

    let base_url = server_config.get_base_url();
    for song in &mut songs_with_url {
        song.set_cover_url(&base_url);
    }

    Ok(ApiResponse::ok(songs_with_url))
}

// 获取封面图片
pub async fn get_song_cover(
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<Response, ApiError> {
    let song = sqlx::query("SELECT cover_data FROM songs WHERE id = $1")
        .bind(song_id)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    match song {
        Some(row) => {
            let cover_data: Option<Vec<u8>> = row
                .try_get("cover_data")
                .map_err(|e| ApiError::Internal(format!("获取封面数据失败: {}", e)))?;
            match cover_data {
                Some(data) => {
                    // 返回封面图片
                    Ok(([(header::CONTENT_TYPE, "image/jpeg")], data).into_response())
                }
                None => Err(ApiError::not_found(ApiError::SONG_COVER_NOT_FOUND, "封面图片不存在")),
            }
        }
        None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
    }
}

// 获取歌曲文件
pub async fn get_song_file(
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<Response, ApiError> {
    let song = sqlx::query("SELECT path FROM songs WHERE id = $1")
        .bind(song_id)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    match song {
        Some(row) => {
            let path: String = row
                .try_get("path")
                .map_err(|e| ApiError::Internal(format!("获取歌曲路径失败: {}", e)))?;

            let file = fs::read(&path)
                .map_err(|e| ApiError::Internal(format!("读取歌曲文件失败: {}", e)))?;

            let content_type = if path.ends_with(".mp3") {
                "audio/mpeg"
            } else if path.ends_with(".flac") {
                "audio/flac"
            } else if path.ends_with(".wav") {
                "audio/wav"
            } else if path.ends_with(".ogg") {
                "audio/ogg"
            } else {
                "application/octet-stream"
            };

            Ok(([(header::CONTENT_TYPE, content_type)], file).into_response())
        }
        None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
    }
}

// 获取歌词(LRC)
pub async fn get_song_lyrics(
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<LyricsResponse>> {
    let song = sqlx::query_as::<_, Song>("SELECT * FROM songs WHERE id = $1")
        .bind(song_id)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    match song {
        Some(song) => {
            let song_path = Path::new(&song.path);
            let song_stem = song_path
                .file_stem()
                .ok_or_else(|| ApiError::Internal("无法解析歌曲文件名".to_string()))?
                .to_string_lossy();
            let lrc_filename = format!("{}.lrc", song_stem);
            let lrc_path = song_path
                .parent()
                .ok_or_else(|| ApiError::Internal("无法解析歌曲目录".to_string()))?
                .join(&lrc_filename);

            let lyrics = fs::read_to_string(&lrc_path).map_err(|_| {
                ApiError::not_found(ApiError::LYRICS_NOT_FOUND, format!("歌词文件不存在: {}", lrc_filename))
            })?;

            Ok(ApiResponse::ok(LyricsResponse {
                song_id: song.id,
                title: song.title,
                artist: song.artist,
                lyrics,
            }))
        }
        None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
    }
}

// ─── TTML 逐字歌词 ────────────────────────────────────────

/// 获取 TTML 逐字歌词（从数据库 songs.ttml 列）
pub async fn get_song_ttml(
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<String>> {
    let row = sqlx::query("SELECT ttml FROM songs WHERE id = $1")
        .bind(song_id)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    match row {
        Some(row) => {
            let ttml: Option<String> = row.get("ttml");
            match ttml {
                Some(content) if !content.is_empty() => Ok(ApiResponse::ok(content)),
                _ => Err(ApiError::not_found(ApiError::NO_TTML_LYRICS, "该歌曲没有 TTML 歌词")),
            }
        }
        None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
    }
}

// 获取所有类型的歌词（从数据库）
pub async fn get_all_lyrics(
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<AllLyricsResponse>> {
    let song = sqlx::query_as::<_, Song>("SELECT * FROM songs WHERE id = $1")
        .bind(song_id)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    match song {
        Some(song) => {
            // 查询所有歌词列
            let row = sqlx::query(
                "SELECT lrc, ttml, eslrc, qrc, yrc, lys, auto_ttml FROM songs WHERE id = $1"
            )
            .bind(song_id)
            .fetch_optional(&pg_pool)
            .await
            .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

            match row {
                Some(row) => {
                    let lrc: Option<String> = row.get("lrc");
                    let ttml: Option<String> = row.get("ttml");
                    let eslrc: Option<String> = row.get("eslrc");
                    let qrc: Option<String> = row.get("qrc");
                    let yrc: Option<String> = row.get("yrc");
                    let lys: Option<String> = row.get("lys");
                    let auto_ttml: Option<String> = row.get("auto_ttml");

                    Ok(ApiResponse::ok(AllLyricsResponse {
                        song_id: song.id,
                        title: song.title,
                        artist: song.artist,
                        lrc,
                        ttml,
                        eslrc,
                        qrc,
                        yrc,
                        lys,
                        auto_ttml,
                    }))
                }
                None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
            }
        }
        None => Err(ApiError::not_found(ApiError::SONG_NOT_FOUND, "歌曲不存在")),
    }
}

// 记录播放历史
pub async fn record_play(
    claims: Claims,
    AxumPath(song_id): AxumPath<Uuid>,
    Extension(mut redis_conn): Extension<ConnectionManager>,
) -> impl IntoResponse {
    let key = format!("play_history:{}", claims.id);
    let now = chrono::Utc::now().timestamp();
    let member = song_id.to_string();

    // ZADD 添加到有序集合，score 为时间戳
    let _: Result<(), _> = redis_conn
        .zadd(&key, &member, now)
        .await;

    // 只保留最近 100 条
    let _: Result<(), _> = redis_conn
        .zremrangebyrank(&key, 0, -101)
        .await;

    StatusCode::OK
}

// 获取播放历史
pub async fn get_play_history(
    claims: Claims,
    Extension(mut redis_conn): Extension<ConnectionManager>,
    Extension(pg_pool): Extension<PgPool>,
    Query(query): Query<PageQuery>,
) -> ApiResult<ApiResponse<Vec<SongWithUrl>>> {
    let key = format!("play_history:{}", claims.id);
    let limit = query.page_size as isize;
    let offset = ((query.page - 1) * query.page_size) as isize;

    // ZREVRANGE 按时间倒序获取 song_id 列表
    let song_ids: Vec<String> = redis_conn
        .zrevrange(&key, offset, offset + limit - 1)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis 查询失败: {}", e)))?;

    if song_ids.is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    // 解析 UUID
    let uuids: Vec<Uuid> = song_ids
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();

    if uuids.is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    // 从数据库批量查询歌曲信息
    let songs = sqlx::query_as::<_, Song>(
        "SELECT * FROM songs WHERE id = ANY($1)"
    )
    .bind(&uuids)
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("数据库查询失败: {}", e)))?;

    // 按播放历史顺序排列
    let song_map: std::collections::HashMap<Uuid, Song> = songs
        .into_iter()
        .map(|s| (s.id, s))
        .collect();

    let base_url = std::env::var("DOMAIN").unwrap_or_else(|_| "192.168.31.58".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "23001".to_string());

    let result: Vec<SongWithUrl> = uuids
        .iter()
        .filter_map(|id| song_map.get(id).map(|song| {
            let cover_url = format!("http://{}:{}/api/songs/cover/{}", base_url, port, song.id);
            SongWithUrl {
                id: song.id,
                title: song.title.clone(),
                path: song.path.clone(),
                album: song.album.clone(),
                artist: song.artist.clone(),
                cover_path: song.cover_path.clone(),
                cover_url: Some(cover_url),
            }
        }))
        .collect();

    Ok(ApiResponse::ok(result))
}

// 歌曲相关路由（需要认证）
pub fn songs_routes() -> Router {
    Router::new()
        .route("/scan", post(scan_songs))
        .route("/all", get(get_all_songs))
        .route("/album", get(get_songs_by_album))
        .route("/artist", get(get_songs_by_artist))
        .route("/history", get(get_play_history))
        .route("/{song_id}/lyrics", get(get_song_lyrics))
        .route("/{song_id}/ttml", get(get_song_ttml))
        .route("/{song_id}/all-lyrics", get(get_all_lyrics))
        .route("/{song_id}/play", post(record_play))
        .route("/{song_id}", get(get_song_file))
}

// 封面图片路由（不需要认证）
pub fn songs_cover_route() -> Router {
    Router::new().route("/api/songs/cover/{song_id}", get(get_song_cover))
}

// 歌曲文件路由（不需要认证，用于音频流式播放）
pub fn songs_file_route() -> Router {
    Router::new().route("/api/songs/file/{song_id}", get(get_song_file))
}
