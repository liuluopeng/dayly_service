use regex::Regex;
use scraper::{Html, Selector};

use super::mhtml::MhtmlResource;

/// An image ready to be embedded in the EPUB.
#[derive(Debug, Clone)]
pub struct EmbeddedImage {
    pub local_name: String,
    pub data: Vec<u8>,
    pub mime_type: String,
}

/// CSS selectors for elements to strip from the article body.
const STRIP_SELECTORS: &[&str] = &[
    "div.subscribe-widget",
    "div.youtube-wrap",
    "div.header-anchor-parent",
    "div.header-anchor",
    "div.available-actions",
    "button",
    "svg",
];

/// Extract only the article body from the full page HTML.
///
/// Parses the DOM, finds `<div class="body markup">`, removes UI chrome
/// (subscribe widgets, buttons, anchor icons, etc.) via DOM detach, then
/// serializes the cleaned subtree.
pub fn extract_article_body(html: &str) -> Option<String> {
    let mut document = Html::parse_document(html);

    // First, remove all unwanted descendant nodes inside `div.body.markup`.
    // We do this on the full document so the tree handles nesting correctly.
    for sel_str in STRIP_SELECTORS {
        let selector = match Selector::parse(sel_str) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let ids: Vec<_> = document.select(&selector).map(|e| e.id()).collect();
        for id in ids {
            if let Some(mut node) = document.tree.get_mut(id) {
                node.detach();
            }
        }
    }

    // Now serialize the full document and extract just the body markup content.
    let full_html = document.html();
    extract_div_inner(&full_html, "div", "markup")
}

/// Extract the inner HTML of `<tag class="...class...">` from a string.
/// Handles nested tags of the same name by counting depth.
fn extract_div_inner(html: &str, tag: &str, class: &str) -> Option<String> {
    // Find the opening tag with the target class.
    let open_pattern = format!(
        r#"(?is)<{}\s[^>]*class="[^"]*\b{}\b[^"]*"[^>]*>"#,
        regex::escape(tag),
        regex::escape(class)
    );
    let open_re = Regex::new(&open_pattern).ok()?;
    let open_match = open_re.find(html)?;
    let inner_start = open_match.end();

    // Scan forward to find the matching closing tag, counting nesting depth.
    let tag_lower = tag.to_lowercase();
    let open_prefix = format!("<{}", tag_lower);
    let close_prefix = format!("</{}", tag_lower);

    let mut depth: u32 = 1;
    let mut pos = inner_start;
    let bytes = html.as_bytes();

    while pos < bytes.len() {
        // Find next '<' that could be a tag.
        let next_lt = match html[pos..].find('<') {
            Some(i) => pos + i,
            None => break,
        };

        let slice = &html[next_lt..].to_lowercase();
        if slice.starts_with(&close_prefix) {
            // 确认是真正的结束标签（`</dividend>` 这类不应计入）
            let after_prefix = &slice[close_prefix.len()..];
            if after_prefix.starts_with('>')
                || after_prefix.starts_with(' ')
                || after_prefix.starts_with('\t')
                || after_prefix.starts_with('\n')
                || after_prefix.starts_with('/')
            {
                depth -= 1;
                if depth == 0 {
                    return Some(html[inner_start..next_lt].to_string());
                }
            }
            // Skip past the closing tag name.
            pos = next_lt + close_prefix.len();
        } else if slice.starts_with(&open_prefix) {
            // 确认为真正的开始标签（非 `<dividend`、非自闭合 `<div/>`）
            let after_prefix = &slice[open_prefix.len()..];
            if after_prefix.starts_with('>')
                || after_prefix.starts_with(' ')
                || after_prefix.starts_with('\t')
                || after_prefix.starts_with('\n')
            {
                depth += 1;
            }
            pos = next_lt + 1;
        } else {
            pos = next_lt + 1;
        }
    }

    None
}

/// Clean a body-only HTML string for EPUB.
pub fn clean_body_html(html: String) -> String {
    // Remove any remaining script/style tags via regex (safe since we already
    // removed them from the DOM; this catches edge cases).
    let script_re = Regex::new(r#"(?is)<script\b[^>]*>.*?</script>"#).unwrap();
    let result = script_re.replace_all(&html, "").to_string();
    let style_re = Regex::new(r#"(?is)<style\b[^>]*>.*?</style>"#).unwrap();
    let result = style_re.replace_all(&result, "").to_string();

    // Remove dark background inline styles.
    let bg_re = Regex::new(r#"(?i)background(-color)?:\s*[^;]+;?"#).unwrap();
    let result = bg_re.replace_all(&result, "").to_string();
    let white_re = Regex::new(r#"(?i)color:\s*rgb\(255,\s*255,\s*255\)"#).unwrap();
    let result = white_re.replace_all(&result, "color: #333").to_string();

    // Expand self-closing <div/> and <span/> to <div></div> / <span></span>.
    // Some EPUB readers reject self-closing non-void HTML elements.
    let self_close = Regex::new(r#"(?i)<(div|span|p)\b([^>]*)/>"#).unwrap();
    let result = self_close.replace_all(&result, "<$1$2></$1>").to_string();

    // Remove empty pencraft/styling divs (no content, no children).
    let empty_pencraft = Regex::new(r#"(?is)<div\s+class="[^"]*pencraft[^"]*"\s*></div>"#).unwrap();
    let result = empty_pencraft.replace_all(&result, "").to_string();

    // Remove style attributes that became empty.
    let empty_style = Regex::new(r#" style="\s*""#).unwrap();
    empty_style.replace_all(&result, "").to_string()
}

/// Build a mapping from original image URL to local EPUB path.
pub fn build_image_map(
    body_html: &str,
    resources: &[MhtmlResource],
) -> Vec<(String, String, Vec<u8>, String)> {
    let document = Html::parse_fragment(body_html);
    let img_selector = Selector::parse("img").unwrap();

    let mut mappings = Vec::new();
    let mut img_idx: usize = 0;

    for element in document.select(&img_selector) {
        let src = match element.value().attr("src") {
            Some(s) => s.to_string(),
            None => continue,
        };

        if src.starts_with("data:") || src.starts_with("images/") {
            continue;
        }

        if let Some((data, mime)) = find_matching_resource(&src, resources) {
            let ext = mime_to_ext(mime);
            let local_name = format!("images/img_{:03}.{}", img_idx, ext);
            mappings.push((src, local_name, data.clone(), mime.to_string()));
            img_idx += 1;
        }
    }

    mappings
}

fn find_matching_resource<'a>(
    img_url: &str,
    resources: &'a [MhtmlResource],
) -> Option<(&'a Vec<u8>, &'a str)> {
    let img_id = extract_image_id(img_url);

    for res in resources {
        let res_id = extract_image_id(&res.content_location);
        if img_id.is_some() && img_id == res_id {
            return Some((&res.data, &res.content_type));
        }
    }

    for res in resources {
        if !res.content_location.is_empty()
            && img_url.contains(
                &res.content_location[..res
                    .content_location
                    .floor_char_boundary(res.content_location.len().min(80))],
            )
        {
            return Some((&res.data, &res.content_type));
        }
    }

    None
}

fn extract_image_id(url: &str) -> Option<String> {
    let re = Regex::new(r"\$s_![^!]+!").ok()?;
    re.find(url).map(|m| m.as_str().to_string())
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

/// Apply image URL replacements to HTML string.
pub fn apply_image_replacements(
    html: String,
    mappings: &[(String, String, Vec<u8>, String)],
) -> String {
    let mut result = html;
    for (original_url, local_name, _, _) in mappings {
        let escaped = regex::escape(original_url);
        let re = Regex::new(&format!(r#"src=["']{}["']"#, escaped)).unwrap();
        result = re
            .replace_all(&result, format!("src=\"{}\"", local_name))
            .to_string();
    }
    // Strip srcset, loading, data-src, data-srcset, sizes attributes.
    let strip_re =
        Regex::new(r#"\s+(loading|srcset|data-src|data-srcset|sizes)=["'][^"']*["']"#).unwrap();
    result = strip_re.replace_all(&result, "").to_string();
    result
}
