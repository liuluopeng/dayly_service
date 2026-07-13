//! local-agent — macOS 本地守护进程
//!
//! - 剪贴板监听：检测到新图片时，保存到 ~/Pictures/clipboard/

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Duration;

use arboard::Clipboard;
use tracing::{error, info, warn};

mod pasteboard;

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const SAVE_DIR: &str = "Pictures/clipboard";

struct State {
    last_change_count: i64,
    last_hash: u64,
}

impl Default for State {
    fn default() -> Self {
        Self { last_change_count: 0, last_hash: 0 }
    }
}

/// 计算图片 RGBA 数据的哈希值，用于去重
fn hash_image(bytes: &[u8], w: usize, h: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    w.hash(&mut hasher);
    h.hash(&mut hasher);
    hasher.finish()
}

/// 保存 RGBA 图片到本地磁盘
fn save_image(bytes: &[u8], w: usize, h: usize) -> Option<PathBuf> {
    let save_dir = dirs::home_dir().map(|p| p.join(SAVE_DIR))?;
    std::fs::create_dir_all(&save_dir).ok()?;

    let now = chrono::Local::now();
    let filename = format!("{}.png", now.format("%Y-%m-%d_%H-%M-%S"));
    let path = save_dir.join(&filename);

    match image::RgbaImage::from_raw(w as u32, h as u32, bytes.to_vec()) {
        Some(img) => {
            if img.save(&path).is_ok() {
                info!("已保存: {}", path.display());
                Some(path)
            } else {
                warn!("保存失败: {}", path.display());
                None
            }
        }
        None => {
            warn!("RGBA 图像创建失败 {}x{}", w, h);
            None
        }
    }
}

/// 发送图片到服务器进行 OCR 识别
async fn send_to_server(bytes: &[u8], filename: &str) {
    let server_url = std::env::var("SERVER_URL")
        .unwrap_or_else(|_| "http://localhost:23000/api/ocr".to_string());

    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(bytes.to_vec())
        .file_name(filename.to_string())
        .mime_str("image/png")
        .unwrap();

    let form = reqwest::multipart::Form::new().part("image", part);

    match client.post(&server_url).multipart(form).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(text) = body.get("text").and_then(|t| t.as_str()) {
                        info!("OCR 结果: {}", text);
                    }
                }
            } else {
                warn!("服务器返回: {}", resp.status());
            }
        }
        Err(e) => {
            error!("请求失败: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("local_agent=info".parse().unwrap()),
        )
        .init();

    info!("local-agent 启动");

    let mut state = State::default();
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            error!("无法初始化剪贴板: {}", e);
            return;
        }
    };

    loop {
        // macOS: 先检查 changeCount（轻量，不复制数据）
        let changed = match pasteboard::change_count() {
            Some(count) if count != state.last_change_count => {
                state.last_change_count = count;
                true
            }
            Some(_) => false,
            None => true, // 非 macOS 平台：每次都尝试
        };

        if changed {
            if let Ok(img) = clipboard.get_image() {
                let (w, h) = (img.width as u32, img.height as u32);
                let bytes = &*img.bytes;

                let hash = hash_image(bytes, img.width, img.height);
                if hash == state.last_hash {
                    tokio::time::sleep(POLL_INTERVAL).await;
                    continue;
                }
                state.last_hash = hash;

                info!("检测到新图片: {}x{}", w, h);

                if let Some(path) = save_image(bytes, w as usize, h as usize) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    send_to_server(bytes, &filename).await;
                }
            }
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
