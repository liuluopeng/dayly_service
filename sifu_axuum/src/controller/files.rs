use axum::{
    Json,
    body::Body,
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use common::api::base::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::DateTime;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::{error, info, warn};

use crate::middleware::Claims;

#[derive(Deserialize)]
struct FilePath {
    path: String,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Serialize, Clone)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    last_modified: Option<String>,
}

#[derive(Serialize)]
struct DirListing {
    path: String,
    entries: Vec<FileEntry>,
    total: usize,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    last_modified: Option<String>,
    content_type: String,
}

/// 从 JWT Claims 获取用户的所有授权目录
async fn get_user_directories(claims: &Claims, pool: &PgPool) -> Result<Vec<String>, ApiError> {
    let username = &claims.username;

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

/// 验证并解析请求路径，确保在用户的某个授权目录内
fn resolve_path(roots: &[String], request_path: &str) -> Result<PathBuf, ApiError> {
    let is_absolute = Path::new(request_path).is_absolute();

    for root in roots {
        let root_path = match Path::new(root).canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let target = if is_absolute {
            // 绝对路径: 直接使用
            PathBuf::from(request_path)
        } else {
            let clean_path = request_path.trim_start_matches('/');
            if clean_path.is_empty() {
                root_path.clone()
            } else {
                root_path.join(clean_path)
            }
        };

        let canonical = match target.canonicalize() {
            Ok(c) => c,
            Err(_) => continue,
        };

        if canonical.starts_with(&root_path) {
            return Ok(canonical);
        }
    }

    Err(ApiError::bad_request(ApiError::PATH_NOT_IN_DIR, "路径不在任何授权目录内"))
}

/// 检测文件的 Content-Type
fn detect_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        "m4v" => "video/mp4",
        "webm" => "video/webm",
        "ts" => "video/mp2t",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "epub" => "application/epub+zip",
        "txt" | "log" | "md" => "text/plain; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "nfo" => "text/xml; charset=utf-8",
        "srt" | "ass" | "ssa" | "sub" => "text/plain; charset=utf-8",
        "ttml" => "application/xml; charset=utf-8",
        "zip" => "application/zip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => "application/octet-stream",
    }
}

fn format_system_time(time: SystemTime) -> String {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "未知".to_string())
}

// ─── handlers ──────────────────────────────────────────────

/// 列出目录内容
async fn list_files(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<FilePath>,
) -> ApiResult<Json<DirListing>> {
    let roots = get_user_directories(&claims, &pool).await?;

    // 空路径或根路径 → 返回授权目录列表
    if query.path.is_empty() || query.path == "/" {
        let entries: Vec<FileEntry> = roots.iter().map(|dir| {
            let name = std::path::Path::new(dir)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.clone());
            FileEntry {
                name,
                path: dir.clone(),
                is_dir: true,
                size: 0,
                last_modified: None,
            }
        }).collect();
        let total = entries.len();
        return Ok(Json(DirListing {
            path: String::new(),
            entries,
            total,
        }));
    }

    let full_path = resolve_path(&roots, &query.path)?;

    if !full_path.is_dir() {
        return Err(ApiError::bad_request(ApiError::NOT_A_DIRECTORY, "路径不是目录"));
    }

    let mut entries = Vec::new();
    let dir_entries = fs::read_dir(&full_path)
        .map_err(|e| ApiError::Internal(format!("读取目录失败: {}", e)))?;

    for entry in dir_entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            let is_dir = metadata.is_dir();
            // 使用完整绝对路径，确保多根目录场景下路径唯一
            let entry_path = entry.path().to_string_lossy().to_string();

            entries.push(FileEntry {
                name,
                path: entry_path,
                is_dir,
                size: if is_dir { 0 } else { metadata.len() },
                last_modified: metadata.modified().ok().map(format_system_time),
            });
        }
    }

    // 目录在前，文件在后，按名称排序
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    let total = entries.len();

    // 分页
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(200).min(1000);
    let start = ((page - 1) * page_size).min(total);
    let end = (start + page_size).min(total);
    let paged = entries[start..end].to_vec();

    Ok(Json(DirListing {
        path: query.path,
        entries: paged,
        total,
    }))
}

/// 获取文件元数据
async fn file_info(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<FilePath>,
) -> ApiResult<Json<FileInfo>> {
    let roots = get_user_directories(&claims, &pool).await?;
    let full_path = resolve_path(&roots, &query.path)?;

    if !full_path.exists() {
        return Err(ApiError::not_found(ApiError::PATH_NOT_FOUND, "路径不存在"));
    }

    let metadata = fs::metadata(&full_path)
        .map_err(|e| ApiError::Internal(format!("获取元数据失败: {}", e)))?;

    let name = full_path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(Json(FileInfo {
        name,
        path: query.path,
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        last_modified: metadata.modified().ok().map(format_system_time),
        content_type: if metadata.is_dir() {
            "inode/directory".to_string()
        } else {
            detect_content_type(&full_path).to_string()
        },
    }))
}

const BUF_SIZE: usize = 65536; // 64KB, 与 dufs 一致

/// 读取文件，支持 Range 请求、ETag 缓存
async fn serve_file(
    Extension(pool): Extension<PgPool>,
    claims: Claims,
    Query(query): Query<FilePath>,
    headers: HeaderMap,
) -> Response {
    let roots = match get_user_directories(&claims, &pool).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let full_path = match resolve_path(&roots, &query.path) {
        Ok(p) => p,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    if !full_path.exists() || full_path.is_dir() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let metadata = match fs::metadata(&full_path) {
        Ok(m) => m,
        Err(e) => {
            error!("获取文件元数据失败: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let file_size = metadata.len();
    let content_type = detect_content_type(&full_path);
    let filename = full_path.file_name().unwrap_or_default().to_string_lossy();

    // ETag + Last-Modified (参照 dufs)
    let mtime = metadata.modified().ok().or_else(|| metadata.created().ok());
    let mtime_ts = mtime.map(|t| {
        t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
    }).unwrap_or(0);
    let etag = format!(r#""{}-{}""#, mtime_ts, file_size);
    let last_modified = mtime.and_then(|t| {
        let dt: DateTime<chrono::Utc> = t.into();
        Some(dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
    });

    // 条件请求：If-None-Match / If-Modified-Since → 304
    if let (Some(etag_val), Some(lm_val)) = (Some(&etag), last_modified.as_ref()) {
        if let Some(if_none_match) = headers.get("if-none-match") {
            if if_none_match.to_str().unwrap_or("") == etag_val {
                return StatusCode::NOT_MODIFIED.into_response();
            }
        }
        if let Some(if_modified_since) = headers.get("if-modified-since") {
            if if_modified_since.to_str().unwrap_or("") == lm_val.as_str() {
                return StatusCode::NOT_MODIFIED.into_response();
            }
        }
    }

    // If-Range：文件已变则忽略 Range，返回完整文件
    let use_range = if let Some(if_range) = headers.get("if-range") {
        if_range.to_str().unwrap_or("") == etag
    } else {
        true // 没有 If-Range，正常处理 Range
    };

    // 处理 Range 请求
    if use_range {
        if let Some(range) = headers.get("Range") {
            if let Ok(range_str) = range.to_str() {
                if let Some((start, end)) = parse_range_header(range_str, file_size) {
                    return handle_range_request(&full_path, start, end, file_size, &content_type, &etag, last_modified.as_deref()).await;
                } else {
                    let mut resp = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
                    resp.headers_mut().insert("content-range", format!("bytes */{}", file_size).parse().unwrap());
                    return resp;
                }
            }
        }
    }

    // HEAD 请求只返回头
    if headers.get("method").map(|v| v == "HEAD").unwrap_or(false) {
        let mut resp = StatusCode::OK.into_response();
        let h = resp.headers_mut();
        h.insert(axum::http::header::CONTENT_TYPE, content_type.parse().unwrap());
        h.insert(axum::http::header::ACCEPT_RANGES, "bytes".parse().unwrap());
        h.insert(axum::http::header::CONTENT_LENGTH, file_size.to_string().parse().unwrap());
        h.insert("etag", etag.parse().unwrap());
        if let Some(lm) = last_modified { h.insert("last-modified", lm.parse().unwrap()); }
        return resp;
    }

    // 完整文件响应（流式 64KB 缓冲，参照 dufs）
    match File::open(&full_path).await {
        Ok(file) => {
            let stream = ReaderStream::with_capacity(file, BUF_SIZE);
            let mut resp = (
                StatusCode::OK,
                axum::http::HeaderMap::new(),
                Body::from_stream(stream),
            ).into_response();
            let h = resp.headers_mut();
            h.insert(axum::http::header::CONTENT_TYPE, content_type.parse().unwrap());
            h.insert(axum::http::header::ACCEPT_RANGES, "bytes".parse().unwrap());
            h.insert(axum::http::header::CONTENT_LENGTH, file_size.to_string().parse().unwrap());
            h.insert(axum::http::header::CONTENT_DISPOSITION,
                format!(r#"inline; filename="{}""#, filename)
                    .parse()
                    .unwrap_or(axum::http::HeaderValue::from_static("inline")));
            h.insert("etag", etag.parse().unwrap());
            if let Some(lm) = last_modified { h.insert("last-modified", lm.parse().unwrap()); }
            resp
        }
        Err(e) => {
            error!("打开文件失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn parse_range_header(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..];
    let parts: Vec<&str> = range_part.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start: u64 = if parts[0].is_empty() {
        0
    } else {
        parts[0].parse().ok()?
    };

    let end: u64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    if start >= file_size || end >= file_size || start > end {
        return None;
    }

    Some((start, end))
}

async fn handle_range_request(
    path: &PathBuf,
    start: u64,
    end: u64,
    file_size: u64,
    content_type: &str,
    etag: &str,
    last_modified: Option<&str>,
) -> Response {
    let content_length = end - start + 1;
    info!("Range: bytes={}-{} ({} bytes), path: {:?}", start, end, content_length, path);

    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(e) => {
            error!("打开文件失败: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
        error!("seek 失败: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let limited = file.take(content_length);
    let stream = ReaderStream::with_capacity(limited, BUF_SIZE);
    let content_range = format!("bytes {}-{}/{}", start, end, file_size);

    let mut resp = (
        StatusCode::PARTIAL_CONTENT,
        axum::http::HeaderMap::new(),
        Body::from_stream(stream),
    ).into_response();
    let h = resp.headers_mut();
    h.insert(axum::http::header::CONTENT_TYPE, content_type.parse().unwrap());
    h.insert(axum::http::header::ACCEPT_RANGES, "bytes".parse().unwrap());
    h.insert(axum::http::header::CONTENT_RANGE, content_range.parse().unwrap());
    h.insert(axum::http::header::CONTENT_LENGTH, content_length.to_string().parse().unwrap());
    h.insert("etag", etag.parse().unwrap());
    if let Some(lm) = last_modified { h.insert("last-modified", lm.parse().unwrap()); }
    resp
}

fn build_tree_string(path: &Path, prefix: &str, is_last: bool) -> String {
    let mut result = String::new();
    let connector = if is_last { "└── " } else { "├── " };
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    result.push_str(&format!("{}{}{}\n", prefix, connector, name));

    if path.is_dir() {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        if let Ok(entries) = fs::read_dir(path) {
            let mut dirs: Vec<PathBuf> = Vec::new();
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    dirs.push(p);
                } else {
                    files.push(p);
                }
            }
            dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            let mut all = dirs;
            all.extend(files);
            let len = all.len();
            for (i, child) in all.iter().enumerate() {
                result.push_str(&build_tree_string(child, &new_prefix, i == len - 1));
            }
        }
    }
    result
}

#[derive(Deserialize)]
struct TreeRequest {
    path: String,
}

async fn generate_tree(Json(params): Json<TreeRequest>) -> Result<Json<serde_json::Value>, ApiError> {
    let target = PathBuf::from(&params.path);
    if !target.is_dir() {
        return Err(ApiError::Internal("路径不是目录".to_string()));
    }

    let dir_name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "root".to_string());
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("树_{}_{}.txt", dir_name, timestamp);

    let parent = target.parent().unwrap_or(Path::new("/"));
    let output_path = parent.join(&filename);

    let mut content = format!("{}\n", dir_name);
    if let Ok(entries) = fs::read_dir(&target) {
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() { dirs.push(p); } else { files.push(p); }
        }
        dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        let mut all = dirs;
        all.extend(files);
        let len = all.len();
        for (i, child) in all.iter().enumerate() {
            content.push_str(&build_tree_string(child, "", i == len - 1));
        }
    }

    fs::write(&output_path, &content).map_err(|e| ApiError::Internal(format!("写入文件失败: {}", e)))?;
    info!("生成目录树: {} -> {}", params.path, output_path.display());

    Ok(Json(serde_json::json!({
        "success": true,
        "file_path": output_path.to_string_lossy().to_string(),
        "filename": filename,
    })))
}

pub fn file_routes() -> Router {
    Router::new()
        .route("/serve", get(serve_file))
        .route("/list", get(list_files))
        .route("/info", get(file_info))
        .route("/tree", post(generate_tree))
}
