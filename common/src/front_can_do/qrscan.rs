use quircs::Quirc;

#[derive(Debug)]
pub enum ScanError {
    Decode(String),
    NoQrFound,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::Decode(s) => write!(f, "图片解码失败: {}", s),
            ScanError::NoQrFound => write!(f, "未检测到 QR 码"),
        }
    }
}

/// 从图片中扫描并解码 QR 码
pub fn scan_qr_from_image(data: &[u8]) -> Result<String, ScanError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ScanError::Decode(e.to_string()))?;

    let grey = img.to_luma8();
    let (w, h) = grey.dimensions();

    let mut quirc = Quirc::new();
    let codes: Vec<_> = quirc
        .identify(w as usize, h as usize, &grey)
        .collect::<Result<_, _>>()
        .map_err(|e| ScanError::Decode(e.to_string()))?;

    for code in &codes {
        if let Ok(data) = code.decode() {
            let text = String::from_utf8_lossy(&data.payload).to_string();
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }

    Err(ScanError::NoQrFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::front_can_do::qrcode::generate_qr_png;

    #[test]
    fn test_scan_generated_qr() {
        let png = generate_qr_png("Hello from Rust QR!", 10, 2).unwrap();
        let text = scan_qr_from_image(&png).unwrap();
        assert_eq!(text, "Hello from Rust QR!");
    }

    #[test]
    fn test_scan_qr_url() {
        let png = generate_qr_png("https://example.com/qr/test", 10, 2).unwrap();
        let text = scan_qr_from_image(&png).unwrap();
        assert_eq!(text, "https://example.com/qr/test");
    }

    #[test]
    fn test_scan_qr_chinese() {
        let png = generate_qr_png("中文内容测试", 12, 2).unwrap();
        let text = scan_qr_from_image(&png).unwrap();
        assert_eq!(text, "中文内容测试");
    }

    #[test]
    fn test_scan_no_qr() {
        let img = image::DynamicImage::new_luma8(100, 100);
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let result = scan_qr_from_image(&buf.into_inner());
        assert!(matches!(result, Err(ScanError::NoQrFound)));
    }

    #[test]
    fn test_scan_invalid_image() {
        let result = scan_qr_from_image(b"not an image");
        assert!(matches!(result, Err(ScanError::Decode(_))));
    }
}
