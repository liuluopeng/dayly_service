//! local-agent — macOS 本地守护进程
//!
//! 子命令:
//!   monitor   监听剪贴板图片（默认）
//!   convert   将 MHTML 文件转换为 Markdown

mod convert;
mod pasteboard;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "local-agent", version, about = "macOS 本地守护进程")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 监听剪贴板，自动保存图片
    Monitor,
    /// 将 MHTML 文件/目录转换为 Markdown
    Convert {
        /// MHTML 文件或包含 .mhtml 的目录
        input: PathBuf,
        /// 输出目录（默认 /Volumes/six/MD）
        #[arg(short, long, default_value = "/Volumes/six/MD")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Monitor) {
        Commands::Monitor => run_monitor(),
        Commands::Convert { input, output } => {
            if !input.exists() {
                eprintln!("文件不存在: {}", input.display());
                std::process::exit(1);
            }

            let files: Vec<PathBuf> = if input.is_dir() {
                let readdir = match std::fs::read_dir(&input) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("读取目录失败: {}", e);
                        std::process::exit(1);
                    }
                };
                let mut list = Vec::new();
                for entry in readdir {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("读取目录项失败: {}", e);
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().map(|e| e == "mhtml").unwrap_or(false) {
                        list.push(path);
                    }
                }
                list.sort();
                list
            } else {
                vec![input.clone()]
            };

            if files.is_empty() {
                eprintln!("未找到 .mhtml 文件");
                std::process::exit(1);
            }

            let total = files.len();
            eprintln!("开始处理 {} 个文件...\n", total);

            use rayon::prelude::*;
            let results: Vec<(&PathBuf, Result<(), String>)> = files
                .par_iter()
                .map(|f| {
                    let result = convert::convert_mhtml(f, &output);
                    (f, result)
                })
                .collect();

            let ok = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail = results.iter().filter(|(_, r)| r.is_err()).count();
            for (f, result) in &results {
                if let Err(e) = result {
                    eprintln!("  ❌ {}: {}", f.display(), e);
                }
            }
            eprintln!("\n完成：{} 成功，{} 失败，共 {}", ok, fail, total);
        }
    }
}

fn run_monitor() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Duration;

    use arboard::Clipboard;
    use tracing::{error, info, warn};

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

    fn hash_image(bytes: &[u8], w: usize, h: usize) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        w.hash(&mut hasher);
        h.hash(&mut hasher);
        hasher.finish()
    }

    fn save_image(bytes: &[u8], w: usize, h: usize) -> Option<std::path::PathBuf> {
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

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("local_agent=info".parse().unwrap()),
        )
        .init();

    info!("local-agent monitor 启动");

    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut state = State::default();
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            error!("无法初始化剪贴板: {}", e);
            return;
        }
    };

    loop {
        let changed = match pasteboard::change_count() {
            Some(count) if count != state.last_change_count => {
                state.last_change_count = count;
                true
            }
            Some(_) => false,
            None => true,
        };

        if changed {
            if let Ok(img) = clipboard.get_image() {
                let (w, h) = (img.width as u32, img.height as u32);
                let bytes = &*img.bytes;

                let hash = hash_image(bytes, img.width, img.height);
                if hash == state.last_hash {
                    std::thread::sleep(POLL_INTERVAL);
                    continue;
                }
                state.last_hash = hash;

                info!("检测到新图片: {}x{}", w, h);

                if let Some(path) = save_image(bytes, w as usize, h as usize) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    rt.block_on(send_to_server(bytes, &filename));
                }
            }
        }

        std::thread::sleep(POLL_INTERVAL);
    }
}
