//! MHTML → Markdown 转换命令

use std::path::Path;

use common::mhtml::html::{apply_image_replacements, build_image_map, clean_body_html, extract_article_body};
use common::mhtml::markdown::html_to_markdown;
use common::mhtml::mhtml::parse_mhtml;

/// 将 MHTML 文件转换为 index.md + images/
pub fn convert_mhtml(input: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(input)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 1. 解析 MHTML
    let doc = parse_mhtml(&content)
        .map_err(|e| format!("解析 MHTML 失败: {}", e))?;

    // 2. 提取正文（DOM 去广告）
    let body = extract_article_body(&doc.html)
        .unwrap_or_else(|| {
            eprintln!("  [警告] 未找到 `.body.markup`，使用完整 HTML 作为正文");
            doc.html.clone()
        });

    // 3. 图片映射（原 URL → 本地文件名）
    let mappings = build_image_map(&body, &doc.resources);
    eprintln!("  [信息] 找到 {} 张图片", mappings.len());

    // 4. 替换 HTML 中的图片 URL 为本地路径
    let processed = apply_image_replacements(body, &mappings);

    // 5. 清理 HTML
    let cleaned = clean_body_html(processed);

    // 6. 转 Markdown
    let markdown = html_to_markdown(&cleaned);

    // 7. 创建输出目录
    let stem = input.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let out_dir = input.parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}markdown", &stem));
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;
    std::fs::create_dir_all(out_dir.join("images"))
        .map_err(|e| format!("创建图片目录失败: {}", e))?;

    // 8. 保存图片
    for (original_url, local_name, data, _mime) in &mappings {
        let img_path = out_dir.join(local_name);
        // 确保 images/ 子目录存在
        if let Some(parent) = img_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&img_path, data)
            .map_err(|e| format!("保存图片 {} 失败: {}", local_name, e))?;
        eprintln!("  [保存] {} ← {}", local_name, original_url);
    }

    // 9. 写入 index.md
    let md_path = out_dir.join("index.md");
    std::fs::write(&md_path, &markdown)
        .map_err(|e| format!("写入 index.md 失败: {}", e))?;

    eprintln!("\n  ✅ 完成：{}", md_path.display());
    Ok(())
}
