use common::mhtml::{html::extract_article_body, markdown::html_to_markdown, mhtml::parse_mhtml};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mhtml2md <input.mhtml> [output.md]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        Path::new(input_path).with_extension("md").to_string_lossy().to_string()
    };

    let content = fs::read_to_string(input_path).unwrap_or_else(|e| {
        eprintln!("读取文件失败: {}", e);
        std::process::exit(1);
    });

    let doc = parse_mhtml(&content).unwrap_or_else(|e| {
        eprintln!("解析 MHTML 失败: {}", e);
        std::process::exit(1);
    });

    let title = extract_title(&doc.html);
    let body = extract_article_body(&doc.html).unwrap_or(doc.html);
    let md = html_to_markdown(&body);

    let mut output = String::new();
    if let Some(ref t) = title {
        output.push_str(&format!("---\ntitle: {}\n---\n\n", t));
    }
    output.push_str(&md);

    fs::write(&output_path, &output).unwrap_or_else(|e| {
        eprintln!("写入文件失败: {}", e);
        std::process::exit(1);
    });

    println!("→ {}", output_path);
}

fn extract_title(html: &str) -> Option<String> {
    regex::Regex::new(r"<title>(.*?)</title>").ok()
        .and_then(|re| re.captures(html).map(|c| c[1].to_string()))
}
