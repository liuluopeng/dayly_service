use anyhow::{Context, Result};
use regex::Regex;

/// A single resource extracted from an MHTML archive.
#[derive(Debug, Clone)]
pub struct MhtmlResource {
    pub content_type: String,
    pub content_location: String,
    pub data: Vec<u8>,
}

/// The parsed result of an MHTML file.
#[derive(Debug)]
pub struct MhtmlDocument {
    pub html: String,
    pub resources: Vec<MhtmlResource>,
}

/// Parse an MHTML file into HTML content and embedded resources.
pub fn parse_mhtml(input: &str) -> Result<MhtmlDocument> {
    // Extract the MIME boundary from the Content-Type header.
    let boundary = extract_boundary(input).context("Could not find MIME boundary in MHTML file")?;

    // Split into parts by boundary.
    let boundary_delim = format!("--{}", boundary);
    let parts: Vec<&str> = input.split(&boundary_delim).collect();

    let mut html: Option<String> = None;
    let mut resources = Vec::new();

    for part in &parts[1..] {
        // Skip the final "--" boundary.
        let part = part.trim_end();
        if part == "--" || part.is_empty() {
            continue;
        }

        // Separate headers from body (split on first blank line).
        let (headers_str, body) = match part.split_once("\r\n\r\n") {
            Some(pair) => pair,
            None => match part.split_once("\n\n") {
                Some(pair) => pair,
                None => continue,
            },
        };

        let content_type = extract_header(headers_str, "Content-Type").unwrap_or_default();
        let _content_id = extract_header(headers_str, "Content-ID")
            .unwrap_or_default()
            .trim_matches(|c| c == '<' || c == '>')
            .to_string();
        let content_location = extract_header(headers_str, "Content-Location").unwrap_or_default();
        let transfer_encoding = extract_header(headers_str, "Content-Transfer-Encoding")
            .unwrap_or_default()
            .to_lowercase();

        // Decode body based on transfer encoding.
        let data = match transfer_encoding.as_str() {
            "quoted-printable" => decode_quoted_printable(body),
            "base64" => decode_base64(body),
            _ => body.as_bytes().to_vec(),
        };

        if content_type.starts_with("text/html") && html.is_none() {
            html = Some(String::from_utf8_lossy(&data).to_string());
        } else if content_type.starts_with("image/") {
            resources.push(MhtmlResource {
                content_type,
                content_location,
                data,
            });
        }
    }

    let html = html.context("No HTML content found in MHTML file")?;
    Ok(MhtmlDocument { html, resources })
}

fn extract_boundary(input: &str) -> Option<String> {
    // 支持带引号与不带引号的 boundary 参数
    let re = Regex::new(r#"(?i)boundary\s*=\s*"([^"]+)""#).ok()?;
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].to_string());
    }
    let re = Regex::new(r#"(?i)boundary\s*=\s*([^;\s]+)"#).ok()?;
    let caps = re.captures(input)?;
    let b = caps[1].trim();
    if b.is_empty() {
        None
    } else {
        Some(b.to_string())
    }
}

fn extract_header(headers: &str, name: &str) -> Option<String> {
    let prefix = format!("{}:", name);
    for line in headers.lines() {
        if line.to_lowercase().starts_with(&prefix.to_lowercase()) {
            let value = line[prefix.len()..].trim().to_string();
            // Handle folded headers (continuation lines starting with whitespace).
            return Some(value);
        }
    }
    None
}

fn decode_quoted_printable(body: &str) -> Vec<u8> {
    // The quoted-printable crate expects ASCII input.
    let body_clean = body.trim_end();
    quoted_printable::decode(body_clean, quoted_printable::ParseMode::Robust)
        .unwrap_or_else(|_| body_clean.as_bytes().to_vec())
}

fn decode_base64(body: &str) -> Vec<u8> {
    use base64::Engine;
    let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .unwrap_or_else(|_| body.as_bytes().to_vec())
}
