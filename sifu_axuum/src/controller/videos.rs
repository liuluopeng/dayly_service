use crate::config::env::ServerConfig;
use crate::middleware::Claims;
use axum::body::Body;
use axum::extract::{Extension, Path as AxumPath, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::api::base::{ApiError, ApiResponse, ApiResult, PaginatedResponse};
use fast_image_resize::{images::Image as FirImage, Resizer};
use image::GenericImageView;
use my_type::dto::videos::VideoWithUrl;
use my_type::model::videos::Video;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct VideoListQuery {
    folder: Option<String>,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_page_size")]
    page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

/// 用 ffmpeg 提取视频第 1 秒的帧，生成 200x200 方形预览图，返回 JPEG 字节
fn generate_video_preview(path: &Path) -> Option<Vec<u8>> {
    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args([
            "-ss", "1",
            "-i", path.to_str()?,
            "-frames:v", "1",
            "-f", "image2pipe",
            "-vcodec", "mjpeg",
            "pipe:1",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }

    let img = image::load_from_memory(&output.stdout).ok()?;
    let (w, h) = img.dimensions();
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    let cropped = img.crop_imm(x, y, side, side).to_rgba8();

    let src_image = FirImage::from_vec_u8(side, side, cropped.into_raw(), fast_image_resize::PixelType::U8x4).ok()?;
    let mut dst_image = FirImage::new(200, 200, fast_image_resize::PixelType::U8x4);
    let mut resizer = Resizer::new();
    let mut opts = fast_image_resize::ResizeOptions::new();
    opts.algorithm = fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear);
    resizer.resize(&src_image, &mut dst_image, &opts).ok()?;

    let rgba = dst_image.into_vec();
    let dst_img = image::RgbaImage::from_raw(200, 200, rgba)?;
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(dst_img)
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .ok()?;
    Some(buf.into_inner())
}

// 扫描视频 — 自动扫描用户所有 media_type='video' 的媒体路径
pub async fn scan_videos(
    claims: Claims,
    Extension(pg_pool): Extension<PgPool>,
) -> ApiResult<Json<serde_json::Value>> {
    let media_paths: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, path FROM media_paths WHERE $1 = ANY(allow_list) AND media_type = 'video'",
    )
    .bind(&claims.username)
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("查询媒体路径失败: {}", e)))?;

    if media_paths.is_empty() {
        return Ok(Json(json!({
            "code": 200,
            "message": "未配置视频目录，请先添加媒体路径",
            "data": null
        })));
    }

    let pg_pool_clone = pg_pool.clone();

    // 使用 channel 实现边扫描边插入
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<(Uuid, String, String, String, Uuid, i64, Option<String>, Option<Vec<u8>>)>>(4);

    use rayon::prelude::*;

    // 生产者：边遍历边用 rayon 并行生成预览图，每 50 个立即发送
    tokio::task::spawn_blocking(move || {
        let batch_size = 50;
        let mut file_entries: Vec<(String, String, String, Uuid, i64)> = Vec::new();
        let mut total_scanned = 0u64;

        for (mp_id, mp_path) in &media_paths {
            let dir = Path::new(mp_path);
            if !dir.exists() || !dir.is_dir() {
                error!("媒体路径不存在或不是目录: {}", mp_path);
                continue;
            }
            info!("开始扫描视频目录: {}", mp_path);

            for entry in WalkDir::new(dir)
                .max_depth(100)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let current_path = entry.path();
                if !current_path.is_file() {
                    continue;
                }
                let Some(ext) = current_path.extension() else {
                    continue;
                };
                let ext = ext.to_string_lossy().to_lowercase();
                if !matches!(
                    ext.as_str(),
                    "mp4" | "mkv" | "avi" | "mov" | "flv" | "wmv"
                        | "m4v" | "webm" | "ts" | "mpg" | "mpeg" | "3gp"
                ) {
                    continue;
                }

                let abs_path = current_path.to_string_lossy().to_string();
                let folder_path = current_path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let size = fs::metadata(current_path)
                    .map(|m| m.len() as i64)
                    .unwrap_or(0);

                file_entries.push((abs_path, folder_path, ext, *mp_id, size));

                // 攒够一批就立即并行生成预览图并发送
                if file_entries.len() >= batch_size {
                    let results: Vec<_> = file_entries.par_iter().map(|(abs_path, folder_path, ext, mp_id, size)| {
                        let path = Path::new(abs_path);
                        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        let preview = generate_video_preview(path);
                        (Uuid::now_v7(), name, abs_path.clone(), folder_path.clone(), *mp_id, *size, Some(ext.clone()), preview)
                    }).collect();

                    let count = results.len();
                    if tx.blocking_send(results).is_err() {
                        return;
                    }
                    total_scanned += count as u64;
                    info!("视频扫描进度: 已处理 {} 个", total_scanned);
                    file_entries.clear();
                }
            }
        }

        // 处理剩余不足一批的文件
        if !file_entries.is_empty() {
            let results: Vec<_> = file_entries.par_iter().map(|(abs_path, folder_path, ext, mp_id, size)| {
                let path = Path::new(abs_path);
                let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let preview = generate_video_preview(path);
                (Uuid::now_v7(), name, abs_path.clone(), folder_path.clone(), *mp_id, *size, Some(ext.clone()), preview)
            }).collect();

            let count = results.len();
            let _ = tx.blocking_send(results);
            total_scanned += count as u64;
        }

        info!("视频扫描完成，共处理 {} 个", total_scanned);
    });

    // 消费者：接收批次并插入数据库
    tokio::spawn(async move {
        let mut inserted_total = 0;

        while let Some(batch) = rx.recv().await {
            let count = batch.len();
            let mut transaction = match pg_pool_clone.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    error!("开始事务失败: {}", e);
                    continue;
                }
            };

            let mut batch_inserted = 0;
            for (id, name, path, folder_path, media_path_id, size, format, preview) in batch {
                if let Err(e) = sqlx::query(
                    "INSERT INTO videos (id, name, path, folder_path, media_path_id, size, format, preview)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                     ON CONFLICT (path) DO NOTHING",
                )
                .bind(id)
                .bind(name)
                .bind(path)
                .bind(folder_path)
                .bind(media_path_id)
                .bind(size)
                .bind(format)
                .bind(preview)
                .execute(&mut *transaction)
                .await
                {
                    error!("数据库插入失败: {}", e);
                } else {
                    batch_inserted += 1;
                }
            }

            if let Err(e) = transaction.commit().await {
                error!("提交事务失败: {}", e);
                continue;
            }

            inserted_total += batch_inserted;
            info!("视频扫描进度: 已插入 {} 个 (本批 {} 个)", inserted_total, count);
        }

        info!("视频扫描完成，共插入 {} 个", inserted_total);
    });

    Ok(Json(json!({
        "code": 200,
        "message": "正在处理扫描请求，请稍后查看结果",
        "data": null
    })))
}

// 获取视频列表（分页，可选文件夹过滤）
pub async fn list_videos(
    claims: Claims,
    Query(query): Query<VideoListQuery>,
    Extension(pg_pool): Extension<PgPool>,
    Extension(server_config): Extension<ServerConfig>,
) -> ApiResult<ApiResponse<PaginatedResponse<VideoWithUrl>>> {
    let (total, videos) = if let Some(ref folder) = query.folder {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM videos
             WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
             AND folder_path = $2",
        )
        .bind(&claims.username)
        .bind(folder)
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询总数失败: {}", e)))?;

        let offset = ((query.page - 1) * query.page_size) as i64;
        let videos = sqlx::query_as::<_, Video>(
            "SELECT id, name, path, folder_path, media_path_id, size, duration_ms, format, width, height, NULL::bytea AS preview, created_at
             FROM videos
             WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
             AND folder_path = $2
             ORDER BY name ASC
             LIMIT $3 OFFSET $4",
        )
        .bind(&claims.username)
        .bind(folder)
        .bind(query.page_size as i64)
        .bind(offset)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询视频失败: {}", e)))?;

        (total, videos)
    } else {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM videos
             WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))",
        )
        .bind(&claims.username)
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询总数失败: {}", e)))?;

        let offset = ((query.page - 1) * query.page_size) as i64;
        let videos = sqlx::query_as::<_, Video>(
            "SELECT id, name, path, folder_path, media_path_id, size, duration_ms, format, width, height, NULL::bytea AS preview, created_at
             FROM videos
             WHERE media_path_id IN (SELECT id FROM media_paths WHERE $1 = ANY(allow_list))
             ORDER BY name ASC
             LIMIT $2 OFFSET $3",
        )
        .bind(&claims.username)
        .bind(query.page_size as i64)
        .bind(offset)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("查询视频失败: {}", e)))?;

        (total, videos)
    };

    let base_url = server_config.get_base_url();
    let videos_with_url: Vec<VideoWithUrl> = videos
        .into_iter()
        .map(|vid| {
            let mut dto = VideoWithUrl::from(vid.clone());
            dto.set_serve_url(&base_url, &vid.path);
            dto
        })
        .collect();

    let paginated_response = PaginatedResponse {
        data: videos_with_url,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages: (total + query.page_size as i64 - 1) / query.page_size as i64,
    };

    Ok(ApiResponse::ok(paginated_response))
}

// 获取视频预览图
pub async fn get_video_preview(
    Extension(pg_pool): Extension<PgPool>,
    AxumPath(video_id): AxumPath<String>,
) -> Response {
    let id = match Uuid::parse_str(&video_id) {
        Ok(id) => id,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let result: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT preview FROM videos WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pg_pool)
    .await
    .ok()
    .flatten();

    match result {
        Some(data) if !data.is_empty() => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/jpeg")
            .header("Cache-Control", "public, max-age=86400")
            .body(Body::from(data))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn videos_routes() -> Router {
    Router::new()
        .route("/scan", post(scan_videos))
        .route("/list", get(list_videos))
        .route("/preview/{video_id}", get(get_video_preview))
}
