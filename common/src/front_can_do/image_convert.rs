use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;

#[derive(Debug)]
pub enum ConvertError {
    Decode(String),
    Encode(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::Decode(s) => write!(f, "解码失败: {}", s),
            ConvertError::Encode(s) => write!(f, "编码失败: {}", s),
            ConvertError::UnsupportedFormat(s) => write!(f, "不支持格式: {}", s),
        }
    }
}

/// 检测图片格式
pub fn detect_format(data: &[u8]) -> Option<&'static str> {
    let format = image::guess_format(data).ok()?;
    match format {
        ImageFormat::Png => Some("png"),
        ImageFormat::Jpeg => Some("jpeg"),
        ImageFormat::WebP => Some("webp"),
        ImageFormat::Gif => Some("gif"),
        ImageFormat::Bmp => Some("bmp"),
        _ => None,
    }
}

/// 缩放图片
/// `width`, `height` 为目标尺寸，`filter` 为插值算法
pub fn resize_image(
    input: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ConvertError> {
    let img = image::load_from_memory(input)
        .map_err(|e| ConvertError::Decode(e.to_string()))?;

    if width == 0 || height == 0 {
        return Err(ConvertError::Encode("目标尺寸不能为 0".into()));
    }

    let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
    let mut buf = Cursor::new(Vec::new());
    resized
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// 裁剪图片
/// `x`, `y` 为左上角坐标，`w`, `h` 为裁剪区域宽高
pub fn crop_image(
    input: &[u8],
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, ConvertError> {
    let img = image::load_from_memory(input)
        .map_err(|e| ConvertError::Decode(e.to_string()))?;

    if x + w > img.width() || y + h > img.height() {
        return Err(ConvertError::Encode(format!(
            "裁剪区域超出图片边界 (图片 {}x{}, 裁剪 {}x{} 偏移 ({},{}))",
            img.width(), img.height(), w, h, x, y
        )));
    }

    if w == 0 || h == 0 {
        return Err(ConvertError::Encode("裁剪区域宽高不能为 0".into()));
    }

    let cropped = img.crop_imm(x, y, w, h);
    let mut buf = Cursor::new(Vec::new());
    cropped
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// 将图片数据转换为指定格式（不缩放）
pub fn convert_image(
    input: &[u8],
    output_format: &str,
    quality: u8,
) -> Result<Vec<u8>, ConvertError> {
    convert_image_with_size(input, output_format, quality, 0, 0)
}

/// 将图片数据转换为指定格式，先缩放到目标尺寸（0 表示不缩放）
pub fn convert_image_with_size(
    input: &[u8],
    output_format: &str,
    quality: u8,
    resize_w: u32,
    resize_h: u32,
) -> Result<Vec<u8>, ConvertError> {
    let img = image::load_from_memory(input)
        .map_err(|e| ConvertError::Decode(e.to_string()))?;

    let img = if resize_w > 0 && resize_h > 0 {
        img.resize_exact(resize_w, resize_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    match output_format {
        "jpeg" | "jpg" => encode_jpeg(&img, quality),
        "png" => encode_png(&img),
        "webp" => encode_webp(&img, quality),
        "bmp" => encode_bmp(&img),
        "gif" => encode_gif(&img),
        f => Err(ConvertError::UnsupportedFormat(f.to_string())),
    }
}
/// 解码图片并返回其 RGBA 像素数据及尺寸
pub fn decode_image_info(
    data: &[u8],
) -> Result<(Vec<u8>, u32, u32), ConvertError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ConvertError::Decode(e.to_string()))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buf);
    img.write_with_encoder(encoder)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

fn encode_webp(img: &DynamicImage, _quality: u8) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::WebP)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

fn encode_bmp(img: &DynamicImage) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Bmp)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

fn encode_gif(img: &DynamicImage) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Gif)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// 生成一个简单的渐变色测试图片（RGBA）
fn make_test_image(w: u32, h: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let r = (x as f64 / w as f64 * 255.0) as u8;
            let g = (y as f64 / h as f64 * 255.0) as u8;
            let b = ((x + y) as f64 / (w + h) as f64 * 255.0) as u8;
            buf.extend_from_slice(&[r, g, b, 255]);
        }
    }
    buf
}

/// 用指定格式编码测试图片，返回 bytes
fn encode_raw(w: u32, h: u32, fmt: ImageFormat) -> Vec<u8> {
    let rgba = make_test_image(w, h);
    let img = DynamicImage::ImageRgba8(
        ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, rgba).unwrap(),
    );
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, fmt).unwrap();
    buf.into_inner()
}

// ─── 文件头魔数 ────────────────────────────────────────────────

const PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const JPEG_HEADER: [u8; 3] = [0xFF, 0xD8, 0xFF];
const GIF_HEADER: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61]; // GIF89a
const WEBP_HEADER: [u8; 4] = [0x52, 0x49, 0x46, 0x46]; // RIFF
const WEBP_SUBTYPE: [u8; 4] = [0x57, 0x45, 0x42, 0x50]; // WEBP
const BMP_HEADER: [u8; 2] = [0x42, 0x4D];

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_format ─────────────────────────────────────────

    #[test]
    fn test_detect_png() {
        let data = encode_raw(4, 4, ImageFormat::Png);
        assert_eq!(detect_format(&data), Some("png"));
    }

    #[test]
    fn test_detect_jpeg() {
        let data = encode_raw(4, 4, ImageFormat::Jpeg);
        assert_eq!(detect_format(&data), Some("jpeg"));
    }

    #[test]
    fn test_detect_webp() {
        let data = encode_raw(4, 4, ImageFormat::WebP);
        assert_eq!(detect_format(&data), Some("webp"));
    }

    #[test]
    fn test_detect_gif() {
        let data = encode_raw(4, 4, ImageFormat::Gif);
        assert_eq!(detect_format(&data), Some("gif"));
    }

    #[test]
    fn test_detect_bmp() {
        let data = encode_raw(4, 4, ImageFormat::Bmp);
        assert_eq!(detect_format(&data), Some("bmp"));
    }

    // ── convert_image ─────────────────────────────────────────

    #[test]
    fn test_png_to_jpeg() {
        let png = encode_raw(16, 16, ImageFormat::Png);
        let result = convert_image(&png, "jpeg", 85).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("jpeg"));
        // JPEG 文件头 0xFFD8FF
        assert_eq!(&result[..3], JPEG_HEADER);
    }

    #[test]
    fn test_png_to_webp() {
        let png = encode_raw(16, 16, ImageFormat::Png);
        let result = convert_image(&png, "webp", 80).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("webp"));
        // WebP: RIFF + ... + WEBP
        assert_eq!(&result[..4], WEBP_HEADER);
        assert_eq!(&result[8..12], WEBP_SUBTYPE);
    }

    #[test]
    fn test_png_to_bmp() {
        let png = encode_raw(16, 16, ImageFormat::Png);
        let result = convert_image(&png, "bmp", 0).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("bmp"));
        assert_eq!(&result[..2], BMP_HEADER);
    }

    #[test]
    fn test_png_to_gif() {
        let png = encode_raw(16, 16, ImageFormat::Png);
        let result = convert_image(&png, "gif", 0).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("gif"));
        assert_eq!(&result[..3], &GIF_HEADER[..3]);
    }

    #[test]
    fn test_jpeg_to_png() {
        let jpeg = encode_raw(16, 16, ImageFormat::Jpeg);
        let result = convert_image(&jpeg, "png", 0).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("png"));
        assert_eq!(&result[..8], PNG_HEADER);
    }

    #[test]
    fn test_webp_to_png() {
        let webp = encode_raw(16, 16, ImageFormat::WebP);
        let result = convert_image(&webp, "png", 0).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("png"));
    }

    #[test]
    fn test_jpeg_to_webp() {
        let jpeg = encode_raw(16, 16, ImageFormat::Jpeg);
        let result = convert_image(&jpeg, "webp", 75).unwrap();
        assert!(!result.is_empty());
        assert_eq!(detect_format(&result), Some("webp"));
    }

    #[test]
    fn test_unsupported_format() {
        let png = encode_raw(4, 4, ImageFormat::Png);
        let result = convert_image(&png, "tiff", 0);
        assert!(matches!(result, Err(ConvertError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_invalid_input() {
        let result = convert_image(b"not an image", "png", 0);
        assert!(matches!(result, Err(ConvertError::Decode(_))));
    }

    // ── crop_image ───────────────────────────────────────────

    #[test]
    fn test_crop_basic() {
        let png = encode_raw(20, 20, ImageFormat::Png);
        let cropped = crop_image(&png, 2, 3, 10, 8).unwrap();
        let (_, w, h) = decode_image_info(&cropped).unwrap();
        assert_eq!((w, h), (10, 8));
        assert_eq!(&cropped[..8], PNG_HEADER);
    }

    #[test]
    fn test_crop_full_image() {
        let png = encode_raw(16, 16, ImageFormat::Png);
        let cropped = crop_image(&png, 0, 0, 16, 16).unwrap();
        let (_, w, h) = decode_image_info(&cropped).unwrap();
        assert_eq!((w, h), (16, 16));
    }

    #[test]
    fn test_crop_out_of_bounds() {
        let png = encode_raw(10, 10, ImageFormat::Png);
        let result = crop_image(&png, 8, 8, 5, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_crop_zero_area() {
        let png = encode_raw(10, 10, ImageFormat::Png);
        let result = crop_image(&png, 0, 0, 0, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_crop_preserves_content() {
        // 创建纯红 10x10，裁剪中间 2x2，确保像素值正确
        let mut buf = Vec::with_capacity(10 * 10 * 4);
        for _ in 0..10 * 10 {
            buf.extend_from_slice(&[255, 0, 0, 255]); // RGBA 纯红
        }
        let img = DynamicImage::ImageRgba8(
            ImageBuffer::<Rgba<u8>, _>::from_raw(10, 10, buf).unwrap(),
        );
        let mut png_buf = Cursor::new(Vec::new());
        img.write_to(&mut png_buf, ImageFormat::Png).unwrap();

        let cropped = crop_image(&png_buf.into_inner(), 4, 4, 2, 2).unwrap();
        let (pixels, w, h) = decode_image_info(&cropped).unwrap();
        assert_eq!((w, h), (2, 2));
        // 所有像素应为纯红
        assert!(pixels.chunks(4).all(|p| p[0] == 255 && p[1] == 0 && p[2] == 0 && p[3] == 255));
    }

    #[test]
    fn test_crop_on_jpeg_input() {
        let jpeg = encode_raw(15, 20, ImageFormat::Jpeg);
        let cropped = crop_image(&jpeg, 2, 2, 10, 10).unwrap();
        let (_, w, h) = decode_image_info(&cropped).unwrap();
        assert_eq!((w, h), (10, 10));
    }

    // ── resize_image / convert_image_with_size ─────────────────

    #[test]
    fn test_resize_downscale() {
        let png = encode_raw(40, 30, ImageFormat::Png);
        let resized = resize_image(&png, 20, 15).unwrap();
        let (_, w, h) = decode_image_info(&resized).unwrap();
        assert_eq!((w, h), (20, 15));
    }

    #[test]
    fn test_convert_with_resize() {
        let png = encode_raw(100, 50, ImageFormat::Png);
        let result = convert_image_with_size(&png, "jpeg", 85, 50, 25).unwrap();
        assert_eq!(detect_format(&result), Some("jpeg"));
        let (_, w, h) = decode_image_info(&result).unwrap();
        assert_eq!((w, h), (50, 25));
    }

    #[test]
    fn test_resize_upscale() {
        let png = encode_raw(4, 4, ImageFormat::Png);
        let resized = resize_image(&png, 16, 16).unwrap();
        let (_, w, h) = decode_image_info(&resized).unwrap();
        assert_eq!((w, h), (16, 16));
    }

    #[test]
    fn test_resize_zero() {
        let png = encode_raw(10, 10, ImageFormat::Png);
        assert!(resize_image(&png, 0, 10).is_err());
        assert!(resize_image(&png, 10, 0).is_err());
    }

    #[test]
    fn test_convert_with_size_zero_keeps_original() {
        // convert_image_with_size(w=0,h=0) 应等同于 convert_image
        let png = encode_raw(16, 16, ImageFormat::Png);
        let result = convert_image_with_size(&png, "jpeg", 85, 0, 0).unwrap();
        let (_, w, h) = decode_image_info(&result).unwrap();
        assert_eq!((w, h), (16, 16));
    }

    #[test]
    fn test_jpeg_quality_affects_size() {
        let png = encode_raw(64, 64, ImageFormat::Png);
        let high = convert_image(&png, "jpeg", 95).unwrap();
        let low = convert_image(&png, "jpeg", 5).unwrap();
        assert!(high.len() > low.len(), "high quality should be larger");
    }

    // ── decode_image_info ─────────────────────────────────────

    #[test]
    fn test_decode_image_info_png() {
        let png = encode_raw(10, 20, ImageFormat::Png);
        let (pixels, w, h) = decode_image_info(&png).unwrap();
        assert_eq!(w, 10);
        assert_eq!(h, 20);
        assert_eq!(pixels.len(), 10 * 20 * 4);
    }

    #[test]
    fn test_decode_image_info_jpeg() {
        let jpeg = encode_raw(8, 12, ImageFormat::Jpeg);
        let (pixels, w, h) = decode_image_info(&jpeg).unwrap();
        assert_eq!(w, 8);
        assert_eq!(h, 12);
        assert_eq!(pixels.len(), 8 * 12 * 4);
    }

    // ── 文件头魔数完整校验 ────────────────────────────────────

    #[test]
    fn test_png_magic_bytes() {
        let data = encode_raw(2, 2, ImageFormat::Png);
        assert_eq!(&data[..8], PNG_HEADER);
    }

    #[test]
    fn test_jpeg_magic_bytes() {
        let data = encode_raw(2, 2, ImageFormat::Jpeg);
        assert_eq!(&data[..3], JPEG_HEADER);
    }

    #[test]
    fn test_gif_magic_bytes() {
        let data = encode_raw(2, 2, ImageFormat::Gif);
        assert_eq!(&data[..6], GIF_HEADER);
    }

    #[test]
    fn test_bmp_magic_bytes() {
        let data = encode_raw(2, 2, ImageFormat::Bmp);
        assert_eq!(&data[..2], BMP_HEADER);
    }

    #[test]
    fn test_webp_magic_bytes() {
        let data = encode_raw(2, 2, ImageFormat::WebP);
        assert_eq!(&data[..4], WEBP_HEADER);
        assert_eq!(&data[8..12], WEBP_SUBTYPE);
    }
}


