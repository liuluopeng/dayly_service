use image::{DynamicImage, Rgba};
use qrcode::QrCode;
use qrcode::render::unicode;
use std::io::Cursor;

#[derive(Debug)]
pub enum QrError {
    Encode(String),
    Image(String),
}

impl std::fmt::Display for QrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QrError::Encode(s) => write!(f, "编码失败: {}", s),
            QrError::Image(s) => write!(f, "生成图片失败: {}", s),
        }
    }
}

/// 生成 QR 码 PNG 图片
/// `text` — 要编码的文本
/// `scale` — 每个模块的像素大小（建议 8-20）
/// `margin` — 外边距模块数（默认 2）
pub fn generate_qr_png(text: &str, scale: u32, margin: u32) -> Result<Vec<u8>, QrError> {
    let scale = scale.max(1).min(100);
    let margin = margin.min(10);

    let code = QrCode::new(text).map_err(|e| QrError::Encode(e.to_string()))?;

    let raw_pixels = code
        .render::<Rgba<u8>>()
        .dark_color(Rgba([0, 0, 0, 255]))
        .light_color(Rgba([255, 255, 255, 255]))
        .quiet_zone(margin > 0)
        .module_dimensions(scale as u32, scale as u32)
        .build();

    let w = raw_pixels.width();
    let h = raw_pixels.height();
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let p = raw_pixels[(x, y)];
            rgba.extend_from_slice(&[p.0[0], p.0[1], p.0[2], p.0[3]]);
        }
    }

    let img = DynamicImage::ImageRgba8(
        image::ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, rgba)
            .ok_or_else(|| QrError::Image("创建图像缓冲区失败".into()))?,
    );

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| QrError::Image(e.to_string()))?;

    Ok(buf.into_inner())
}

/// 生成 QR 码的 Unicode 字符表示（终端预览用）
pub fn generate_qr_unicode(text: &str) -> Result<String, QrError> {
    let code = QrCode::new(text).map_err(|e| QrError::Encode(e.to_string()))?;
    let string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    Ok(string)
}

/// 获取 QR 码版本和尺寸信息
pub fn qr_info(text: &str) -> Result<(i16, usize), QrError> {
    use qrcode::types::Version;
    let code = QrCode::new(text).map_err(|e| QrError::Encode(e.to_string()))?;
    let version = match code.version() {
        Version::Normal(v) => v,
        Version::Micro(v) => -v,
    };
    let size = code.width() as usize;
    Ok((version, size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_png() {
        let png = generate_qr_png("Hello, QR!", 10, 2).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[..8], [137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_generate_qr_scaled() {
        let small = generate_qr_png("test", 5, 2).unwrap();
        let big = generate_qr_png("test", 20, 2).unwrap();
        assert!(big.len() > small.len());
    }

    #[test]
    fn test_generate_qr_zero_margin() {
        let png = generate_qr_png("no margin text", 8, 0).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_generate_qr_unicode() {
        let string = generate_qr_unicode("中文测试").unwrap();
        assert!(!string.is_empty());
    }

    #[test]
    fn test_qr_info() {
        let (version, size) = qr_info("test data").unwrap();
        assert_eq!(version, 1);
        assert_eq!(size, 21);
    }

    #[test]
    fn test_qr_long_text() {
        let long = "A".repeat(100);
        let png = generate_qr_png(&long, 8, 2).unwrap();
        assert!(!png.is_empty());
        let (version, _) = qr_info(&long).unwrap();
        assert!(version > 1);
    }

    #[test]
    fn test_qr_empty_string() {
        // 空字符串可以编码为 QR 码
        let result = generate_qr_png("", 10, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_qr_very_long_text() {
        let long = "B".repeat(200);
        let result = generate_qr_png(&long, 6, 1);
        assert!(result.is_ok());
    }
}
