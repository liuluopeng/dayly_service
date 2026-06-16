use crate::middleware::Claims;
use axum::Router;
use axum::extract::{Extension, Json, Path, Query};
use axum::routing::{get, post};
use common::api::base::{ApiError, ApiResult};
use common::api::melatonin::{ActorMovieQuery, MelatoninListQuery};
use my_type::model::melatonin_movie::{MelatoninMetadata, MelatoninMovie, MelatoninMovieList};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::fs;
use std::path::Path as StdPath;
use tracing::{debug, error, info};
use uuid::Uuid;

use common::api::base::{ApiResponse, PaginatedResponse};


/// 获取用户的所有授权目录
async fn get_user_directories(pool: &PgPool, username: &str) -> Result<Vec<String>, ApiError> {
    let directories: Vec<String> = sqlx::query_scalar(
        "SELECT path FROM user_directories WHERE $1 = ANY(allow_list)"
    )
    .bind(username)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(format!("查询用户目录失败: {}", e)))?;

    if directories.is_empty() {
        return Err(ApiError::bad_request(ApiError::NO_AUTHORIZED_DIR, "用户未配置授权目录"));
    }

    Ok(directories)
}

/// 从用户的某个授权目录计算文件的 API URL
fn build_file_url(full_path: &str, user_dirs: &[String]) -> Option<String> {
    for dir in user_dirs {
        let dir = dir.trim_end_matches('/');
        if full_path.starts_with(dir) {
            return Some(format!("/api/files/serve?path={}", urlencoding::encode(full_path)));
        }
    }
    None
}

pub async fn scan_melatonin(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    // 查询该用户所有 dmm 类型的媒体路径
    let mut video_paths: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, path FROM media_paths WHERE $1 = ANY(allow_list) AND media_type = 'melatonin'"
    )
    .bind(&claims.username)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?;

    // 默认 Melatonin 目录
    if video_paths.is_empty() {
        let default_path = "/Volumes/six/vedio/nfo".to_string();
        if StdPath::new(&default_path).exists() {
            video_paths.push((Uuid::nil(), default_path));
        }
    }

    if video_paths.is_empty() {
        return Ok(Json(json!({
            "code": 200,
            "message": "未配置 Melatonin 目录，请先添加媒体路径",
            "data": { "scanned": 0, "added": 0 }
        })));
    }

    let mut movies_scanned = 0;
    let mut movies_added = 0;

    for (media_path_id, base_path_str) in video_paths {
        let base_path = StdPath::new(&base_path_str);
        if !base_path.exists() || !base_path.is_dir() {
            continue;
        }

        // 加载该 media_path 下已扫描的目录集合（用于增量跳过）
        let existing_dirs = load_existing_dirs(&pool, media_path_id).await;

        let mut dir_stack = vec![base_path.to_path_buf()];

        while let Some(dir_path) = dir_stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                            if let Ok(movie) = check_and_add_melatonin_movie(&path, media_path_id, &pool, &existing_dirs).await {
                                if movie.is_some() {
                                    movies_added += 1;
                                }
                                movies_scanned += 1;
                            }

                            dir_stack.push(path);
                        }
                    }
                }
            }
        }
    }

    Ok(Json(json!({
        "code": 200,
        "message": "扫描完成",
        "data": {
            "scanned": movies_scanned,
            "added": movies_added
        }
    })))
}

/// 已扫描目录信息（用于增量更新）
struct ExistingRecord {
    id: Uuid,
    has_video: bool,
    cover_is_empty: bool,
}

/// 加载已扫描目录 → (db_id, has_video, cover_is_empty)
async fn load_existing_dirs(pool: &PgPool, media_path_id: Uuid) -> std::collections::HashMap<String, ExistingRecord> {
    let rows: Vec<(Uuid, Vec<String>, String)> = sqlx::query_as(
        "SELECT id, video_paths, cover_path FROM melatonin_movies WHERE media_path_id = $1"
    )
    .bind(media_path_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .filter_map(|(id, video_paths, cover_path)| {
            let dir_key = if !cover_path.is_empty() {
                StdPath::new(&cover_path).parent().map(|p| p.to_string_lossy().to_string())
            } else if let Some(first_vid) = video_paths.first() {
                StdPath::new(first_vid).parent().map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };
            dir_key.map(|k| (k, ExistingRecord { id, has_video: !video_paths.is_empty(), cover_is_empty: cover_path.is_empty() }))
        })
        .collect()
}

async fn check_and_add_melatonin_movie(
    dir_path: &StdPath,
    media_path_id: Uuid,
    pool: &PgPool,
    existing_dirs: &std::collections::HashMap<String, ExistingRecord>,
) -> Result<Option<MelatoninMovie>, sqlx::Error> {
    debug!("检查目录: {:?}", dir_path);
    let files = match fs::read_dir(dir_path) {
        Ok(files) => files,
        Err(e) => {
            error!("读取目录失败: {:?}", e);
            return Ok(None);
        }
    };

    let mut video_files: Vec<String> = Vec::new();
    let mut nfo_file: Option<String> = None;
    let mut cover_file: Option<String> = None;

    for file in files {
        if let Ok(file) = file {
            match file.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        continue;
                    }

                    let file_name = file.file_name().to_string_lossy().to_string();
                    let lower_name = file_name.to_lowercase();

                    if is_video_file(&lower_name) {
                        video_files.push(file.path().to_string_lossy().to_string());
                    } else if lower_name.ends_with(".nfo") {
                        nfo_file = Some(file.path().to_string_lossy().to_string());
                    } else if is_image_file(&lower_name) {
                        // 优先选择名称含 "cover" 的图片
                        if lower_name.contains("cover") {
                            cover_file = Some(file.path().to_string_lossy().to_string());
                        } else if cover_file.is_none() {
                            cover_file = Some(file.path().to_string_lossy().to_string());
                        }
                    } else {
                        debug!("跳过未知文件类型: {:?}", file.path());
                    }
                }
                Err(e) => {
                    error!("获取文件类型失败: {:?}", e);
                }
            }
        } else if let Err(e) = file {
            error!("读取文件失败: {:?}", e);
        }
    }

    // 只要有 nfo 文件就收录
    let Some(nfo) = nfo_file else {
        return Ok(None);
    };

    let nfo_content = match fs::read_to_string(&nfo) {
        Ok(content) => {
            debug!("成功读取 NFO 文件: {:?}", nfo);
            content
        }
        Err(e) => {
            error!("读取 NFO 文件失败: {:?}, 错误: {:?}", nfo, e);
            return Ok(None);
        }
    };

    let dmm_metadata: MelatoninMetadata = match quick_xml::de::from_str(&nfo_content) {
        Ok(meta) => {
            debug!("成功解析 NFO 文件");
            meta
        }
        Err(e) => {
            error!("解析 NFO 文件失败: {:?}", e);
            return Ok(None);
        }
    };

    let title = dmm_metadata.title.clone().unwrap_or_else(|| {
        dir_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let nfo_json = serde_json::to_value(&dmm_metadata).unwrap_or_default();
    let cover = cover_file.unwrap_or_default();

    // 增量扫描：目录已存在时检查是否有新视频/封面可以补充
    let dir_key = dir_path.to_string_lossy().to_string();
    if let Some(rec) = existing_dirs.get(&dir_key) {
        let mut need_update = false;

        if !rec.has_video && !video_files.is_empty() {
            need_update = true;
        }
        if rec.cover_is_empty && !cover.is_empty() {
            need_update = true;
        }

        if need_update {
            let first_vid = video_files.first().map(|s| s.as_str()).unwrap_or("");
            sqlx::query("UPDATE melatonin_movies SET video_path = CASE WHEN video_path = '' THEN $1 ELSE video_path END, video_paths = CASE WHEN cardinality(video_paths) = 0 THEN $2 ELSE video_paths END, cover_path = CASE WHEN cover_path = '' THEN $3 ELSE cover_path END WHERE id = $4")
                .bind(first_vid).bind(&video_files).bind(&cover).bind(rec.id)
                .execute(pool).await?;
            info!("增量更新: {}", dir_key);
        }
        return Ok(None);
    }

    let id = Uuid::now_v7();
    let movie = MelatoninMovie {
        id,
        title: title.clone(),
        cover_path: cover.clone(),
        video_paths: video_files.clone(),
        nfo_json,
        cover_url: None,
        video_urls: vec![],
        preview_urls: vec![],
    };

    let first_video = video_files.first().cloned().unwrap_or_default();
    let result = sqlx::query(
        "INSERT INTO melatonin_movies (id, title, cover_path, video_path, video_paths, nfo_json, media_path_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&movie.id)
    .bind(&movie.title)
    .bind(&movie.cover_path)
    .bind(&first_video)
    .bind(&movie.video_paths)
    .bind(&movie.nfo_json)
    .bind(media_path_id)
    .execute(pool)
    .await;

    if result.is_err() {
        error!("插入数据库失败: {:?}", result.err());
        return Ok(None);
    }

    Ok(Some(movie))
}

/// 扫描目录中数字命名的预览图，排除 cover 文件
fn scan_preview_urls(dir_path: &StdPath, cover_path: &str, user_dirs: &[String]) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir_path) else {
        return vec![];
    };
    let mut previews: Vec<(u32, String)> = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            // 排除 cover 文件
            if path == cover_path { continue; }
            if let Some(stem) = StdPath::new(&file_name).file_stem() {
                if let Ok(n) = stem.to_string_lossy().parse::<u32>() {
                    let lower = file_name.to_lowercase();
                    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp") {
                        if let Some(url) = build_file_url(&path, user_dirs) {
                            previews.push((n, url));
                        }
                    }
                }
            }
        }
    }
    previews.sort_by_key(|(n, _)| *n);
    previews.into_iter().map(|(_, url)| url).collect()
}

fn is_video_file(name: &str) -> bool {
    name.ends_with(".mp4")
        || name.ends_with(".avi")
        || name.ends_with(".mov")
        || name.ends_with(".mkv")
        || name.ends_with(".flv")
        || name.ends_with(".wmv")
        || name.ends_with(".m4v")
}

fn is_image_file(name: &str) -> bool {
    name.ends_with(".jpg")
        || name.ends_with(".jpeg")
        || name.ends_with(".png")
        || name.ends_with(".gif")
        || name.ends_with(".bmp")
        || name.ends_with(".webp")
}

/// 从数据库行构建 MelatoninMovieList，附带 cover_url/video_urls
fn row_to_movie_list(row: &sqlx::postgres::PgRow, user_dirs: &[String]) -> MelatoninMovieList {
    let cover_path: String = row.get("cover_path");
    let video_paths: Vec<String> = row.get("video_paths");

    MelatoninMovieList {
        id: row.get("id"),
        title: row.get("title"),
        cover_url: build_file_url(&cover_path, user_dirs),
        video_urls: video_paths.iter().filter_map(|p| build_file_url(p, user_dirs)).collect(),
        cover_path,
        video_paths,
    }
}

/// 从数据库行构建 MelatoninMovie，附带 cover_url/video_urls 和预览图
fn row_to_movie(row: &sqlx::postgres::PgRow, user_dirs: &[String]) -> MelatoninMovie {
    let cover_path: String = row.get("cover_path");
    let video_paths: Vec<String> = row.get("video_paths");

    let preview_urls = if !cover_path.is_empty() {
        if let Some(parent) = StdPath::new(&cover_path).parent() {
            scan_preview_urls(parent, &cover_path, user_dirs)
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    MelatoninMovie {
        id: row.get("id"),
        title: row.get("title"),
        nfo_json: row.get("nfo_json"),
        cover_url: build_file_url(&cover_path, user_dirs),
        video_urls: video_paths.iter().filter_map(|p| build_file_url(p, user_dirs)).collect(),
        cover_path,
        video_paths,
        preview_urls,
    }
}

pub async fn get_melatonin_movies(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<MelatoninListQuery>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<MelatoninMovieList>>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let user_dirs = get_user_directories(&pool, &claims.username).await?;

    let rows = sqlx::query(
        "SELECT id, title, cover_path, video_paths FROM melatonin_movies
         WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
         ORDER BY title LIMIT $2 OFFSET $3"
    )
        .bind(&claims.username)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let movies: Vec<MelatoninMovieList> = rows
        .iter()
        .map(|row| row_to_movie_list(row, &user_dirs))
        .collect();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM melatonin_movies
         WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))"
    )
        .bind(&claims.username)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total_pages = (total + page_size as i64 - 1) / page_size as i64;

    let paginated_response = PaginatedResponse {
        data: movies,
        total,
        page,
        page_size,
        total_pages,
    };

    Ok(Json(ApiResponse::ok(paginated_response)))
}

pub async fn get_melatonin_movie_by_id(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> ApiResult<Json<ApiResponse<MelatoninMovie>>> {
    let user_dirs = get_user_directories(&pool, &claims.username).await?;

    let row = sqlx::query(
        "SELECT id, title, cover_path, video_paths, nfo_json FROM melatonin_movies
         WHERE id = $1 AND media_path_id IN (SELECT id FROM media_paths WHERE $2 = ANY(allow_list))"
    )
        .bind(id)
        .bind(&claims.username)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let movie = row_to_movie(&row, &user_dirs);

    Ok(Json(ApiResponse::ok(movie)))
}

pub async fn get_movies_by_actor(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<ActorMovieQuery>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<MelatoninMovieList>>>> {
    let actor_name = query.actor;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let user_dirs = get_user_directories(&pool, &claims.username).await?;

    let all_movies = sqlx::query(
        "SELECT id, title, cover_path, video_paths, nfo_json FROM melatonin_movies
         WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
         ORDER BY title"
    )
        .bind(&claims.username)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut filtered_movies = Vec::new();
    for movie in all_movies {
        if let Ok(dmm_metadata) = serde_json::from_value::<MelatoninMetadata>(
            movie
                .try_get::<serde_json::Value, _>("nfo_json")
                .unwrap_or_default(),
        ) {
            if let Some(actors) = dmm_metadata.actor {
                if actors
                    .iter()
                    .any(|a| a.name.eq_ignore_ascii_case(&actor_name))
                {
                    filtered_movies.push(movie);
                }
            }
        }
    }

    let total = filtered_movies.len() as i64;

    let paginated_movies = filtered_movies
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    let movies: Vec<MelatoninMovieList> = paginated_movies
        .iter()
        .map(|row| row_to_movie_list(row, &user_dirs))
        .collect();

    let total_pages = (total + page_size as i64 - 1) / page_size as i64;

    let paginated_response = PaginatedResponse {
        data: movies,
        total,
        page,
        page_size,
        total_pages,
    };

    Ok(Json(ApiResponse::ok(paginated_response)))
}

pub async fn get_movies_by_genre(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<ActorMovieQuery>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<MelatoninMovieList>>>> {
    let genre = &query.actor; // 复用 actor 查询结构，actor 字段即 genre
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let user_dirs = get_user_directories(&pool, &claims.username).await?;

    // 用 PostgreSQL JSONB 查询匹配 genre
    let all_movies = sqlx::query(
        "SELECT id, title, cover_path, video_paths, nfo_json FROM melatonin_movies
         WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
         AND nfo_json->'genre' ? $2
         ORDER BY title"
    )
        .bind(&claims.username)
        .bind(genre)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total = all_movies.len() as i64;
    let paginated_movies = all_movies
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    let movies: Vec<MelatoninMovieList> = paginated_movies
        .iter()
        .map(|row| row_to_movie_list(row, &user_dirs))
        .collect();

    let total_pages = (total + page_size as i64 - 1) / page_size as i64;
    Ok(Json(ApiResponse::ok(PaginatedResponse {
        data: movies, total, page, page_size, total_pages,
    })))
}

/// 读取 NFO 目录下的 bt_list.csv，返回磁力链接列表
pub async fn get_bt_list(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> ApiResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let row = sqlx::query("SELECT cover_path, video_paths FROM melatonin_movies WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let cover_path: String = row.get("cover_path");
    let video_paths: Vec<String> = row.get("video_paths");

    let dir = if !cover_path.is_empty() {
        StdPath::new(&cover_path).parent().map(|p| p.to_path_buf())
    } else if let Some(v) = video_paths.first() {
        StdPath::new(v).parent().map(|p| p.to_path_buf())
    } else {
        None
    };

    let dir = dir.ok_or_else(|| ApiError::bad_request("NO_DIR", "找不到电影目录"))?;
    let csv_path = dir.join("bt_list.csv");

    if !csv_path.exists() {
        return Ok(Json(ApiResponse::ok(vec![])));
    }

    let content = fs::read_to_string(&csv_path)
        .map_err(|e| ApiError::Internal(format!("读取 CSV 失败: {}", e)))?;

    let mut lines = content.lines();
    let header = lines.next().unwrap_or("");
    let headers: Vec<&str> = header.split(',').collect();
    let name_idx = headers.iter().position(|&h| h == "name");
    let tags_idx = headers.iter().position(|&h| h == "tags");
    let size_idx = headers.iter().position(|&h| h == "file_size");
    let magnet_idx = headers.iter().position(|&h| h == "magnet");

    let items: Vec<serde_json::Value> = lines
        .filter_map(|line| {
            if line.trim().is_empty() { return None; }
            let cols: Vec<&str> = line.split(',').collect();
            let name = name_idx.and_then(|i| cols.get(i)).unwrap_or(&"");
            let tags = tags_idx.and_then(|i| cols.get(i)).unwrap_or(&"");
            let size = size_idx.and_then(|i| cols.get(i)).unwrap_or(&"");
            let magnet = magnet_idx.and_then(|i| cols.get(i)).unwrap_or(&"");
            Some(serde_json::json!({
                "name": name,
                "tags": tags,
                "size": size,
                "magnet": magnet,
            }))
        })
        .collect();

    Ok(Json(ApiResponse::ok(items)))
}

pub fn melatonin_routes() -> Router {
    Router::new()
        .route("/scan", post(scan_melatonin))
        .route("/list", get(get_melatonin_movies))
        .route("/actor", get(get_movies_by_actor))
        .route("/genre", get(get_movies_by_genre))
        .route("/detail/{id}", get(get_melatonin_movie_by_id))
        .route("/bt_list/{id}", get(get_bt_list))
}
