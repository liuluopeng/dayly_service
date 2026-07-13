//! MHTML → Markdown 转换命令

use std::path::{Path, PathBuf};

use common::mhtml::html::{apply_image_replacements, build_image_map, clean_body_html, extract_article_body};
use common::mhtml::markdown::html_to_markdown;
use common::mhtml::mhtml::parse_mhtml;

const OUTPUT_DIR: &str = "/Volumes/six/MD";

pub fn output_dir() -> PathBuf {
    PathBuf::from(OUTPUT_DIR)
}

/// 将 MHTML 文件转换为 MD + 图片，保存到固定目录
pub fn convert_mhtml(input: &Path, out_dir: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(input)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 1. 解析 MHTML
    let doc = parse_mhtml(&content)
        .map_err(|e| format!("解析 MHTML 失败: {}", e))?;

    // 2. 提取正文（DOM 去广告）
    let body = extract_article_body(&doc.html)
        .unwrap_or_else(|| {
            eprintln!("  [警告] 使用完整 HTML 作为正文");
            doc.html.clone()
        });

    // 3. 构建原始图片映射（原 URL → 图片数据 + MIME）
    let raw_mappings = build_image_map(&body, &doc.resources);
    eprintln!("  [信息] 找到 {} 张图片", raw_mappings.len());

    // 4. 生成时间戳文件名，构建最终映射
    let mut img_idx = 0usize;
    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d_%H%M%S");

    let mut mappings: Vec<(String, String, Vec<u8>, String)> = Vec::new();
    for (orig_url, _old_name, data, mime) in &raw_mappings {
        let ext = mime_to_ext(mime);
        let local_name = format!("{}_{}.{}", ts, img_idx, ext);
        mappings.push((orig_url.clone(), local_name, data.clone(), mime.clone()));
        img_idx += 1;
    }

    // 5. 替换 HTML 中的图片 URL
    let processed = apply_image_replacements(body, &mappings);

    // 6. 清理 HTML
    let cleaned = clean_body_html(processed);

    // 7. 转 Markdown
    let markdown = html_to_markdown(&cleaned);

    // 8. 确保输出目录存在
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;

    // 10. 保存图片到输出目录
    for (_orig_url, local_name, data, _mime) in &mappings {
        let img_path = out_dir.join(local_name);
        std::fs::write(&img_path, data)
            .map_err(|e| format!("保存图片 {} 失败: {}", local_name, e))?;
        eprintln!("  [保存] {} ← {}", local_name, _orig_url);
    }

    // 11. 写入 .md
    let stem = input.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let md_path = out_dir.join(format!("{}.md", stem));
    std::fs::write(&md_path, &markdown)
        .map_err(|e| format!("写入 {} 失败: {}", md_path.display(), e))?;

    eprintln!("\n  ✅ {} → {}", input.display(), md_path.display());
    Ok(())
}

fn mime_to_ext(mime: &str) -> &str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}
