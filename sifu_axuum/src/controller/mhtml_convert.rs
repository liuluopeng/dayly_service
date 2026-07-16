use axum::{Json, Router, extract::Json as ExtractJson, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use common::mhtml::{mhtml::parse_mhtml, html::extract_article_body, markdown::html_to_markdown};

#[derive(Deserialize)]
struct MhtmlRequest {
    content: String,
}

#[derive(Serialize)]
struct MhtmlResponse {
    markdown: String,
    title: Option<String>,
}

async fn convert_mhtml(ExtractJson(req): ExtractJson<MhtmlRequest>) -> impl IntoResponse {
    match parse_mhtml(&req.content) {
        Ok(doc) => {
            let title = extract_title(&doc.html);
            let body = extract_article_body(&doc.html).unwrap_or(doc.html);
            let md = html_to_markdown(&body);
            Json(MhtmlResponse { markdown: md, title }).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("解析 MHTML 失败: {}", e)).into_response(),
    }
}

fn extract_title(html: &str) -> Option<String> {
    regex::Regex::new(r"<title>(.*?)</title>").ok()
        .and_then(|re| re.captures(html).map(|c| c[1].to_string()))
}

pub fn mhtml_routes() -> Router {
    Router::new().route("/to-markdown", post(convert_mhtml))
}
