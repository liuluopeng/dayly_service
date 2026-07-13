use axum::extract::Multipart;
use axum::routing::post;
use axum::{Json, Router};
use common::api::base::{ApiError, ApiResult};
use serde_json::json;

/// OCR: 上传图片，返回识别的文字
pub async fn ocr_image(mut multipart: Multipart) -> ApiResult<Json<serde_json::Value>> {
    let mut image_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest { code: "MULTIPART_ERROR".into(), message: format!("读取上传失败: {}", e) })?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            image_data = Some(field.bytes().await.map_err(|e| {
                ApiError::BadRequest { code: "READ_ERROR".into(), message: format!("读取图片数据失败: {}", e) }
            })?.to_vec());
        }
    }

    let data = image_data.ok_or_else(|| ApiError::BadRequest { code: "NO_IMAGE".into(), message: "未找到图片，请使用 field name 'image' 上传".into() })?;

    // 用 image crate 解码为 RGBA
    let img = image::load_from_memory(&data)
        .map_err(|e| ApiError::BadRequest { code: "IMAGE_DECODE_ERROR".into(), message: format!("图片解码失败: {}", e) })?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels = rgba.into_raw();

    tracing::info!("[OCR] 开始识别 {}x{} 图片", w, h);

    let text = tokio::task::spawn_blocking(move || {
        common::ocr::detect_and_recognize(&pixels, w, h)
    })
    .await
    .map_err(|e| {
        tracing::error!("OCR 任务失败: {}", e);
        ApiError::Internal("OCR 处理失败".into())
    })?;

    tracing::info!("[OCR] 识别结果: {} 字", text.chars().count());

    Ok(Json(json!({
        "code": 200,
        "text": text,
    })))
}

pub fn ocr_routes() -> Router {
    Router::new().route("/ocr", post(ocr_image))
}
