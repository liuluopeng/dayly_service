use crate::middleware::Claims;
use axum::Router;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::{Engine as _, engine::general_purpose};
use common::api::base::{ApiError, ApiResponse};
use my_type::dto::NoteSummary;
use my_type::model::note::Note;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SearchQuery {
    query: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct SaveNoteRequest {
    id: Option<Uuid>,
    text: String,
    filename: Option<String>,
}

#[derive(Serialize)]
pub struct SaveNoteResponse {
    id: Uuid,
    message: String,
}

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    text: String,
    filename: Option<String>,
}

#[derive(Serialize)]
pub struct CreateNoteResponse {
    id: Uuid,
    message: String,
}

pub async fn save_note(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::Json(req): axum::Json<SaveNoteRequest>,
) -> Result<axum::Json<ApiResponse<SaveNoteResponse>>, ApiError> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    match req.id {
        Some(id) => {
            let simple_text = req.text.chars().take(500).collect::<String>();

            let result = sqlx::query(
                r#"UPDATE notes SET text = $1, simple_text = $2, updated_at = NOW() WHERE id = $3 AND user_id = $4"#,
            )
            .bind(&req.text)
            .bind(&simple_text)
            .bind(id)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(|e| {
                error!("Database update error: {:?}", e);
                ApiError::Internal(e.to_string())
            })?;

            if result.rows_affected() == 0 {
                let filename = req.filename.unwrap_or_else(|| format!("{}.md", id));

                sqlx::query(
                    r#"
                    INSERT INTO notes (id, text, simple_text, filename, user_id)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(id)
                .bind(&req.text)
                .bind(&simple_text)
                .bind(&filename)
                .bind(user_id)
                .execute(&pool)
                .await
                .map_err(|e| {
                    error!("Database insert error: {:?}", e);
                    ApiError::Internal(e.to_string())
                })?;

                Ok(axum::Json(ApiResponse::ok(SaveNoteResponse {
                    id,
                    message: "Note created successfully".to_string(),
                })))
            } else {
                Ok(axum::Json(ApiResponse::ok(SaveNoteResponse {
                    id,
                    message: "Note updated successfully".to_string(),
                })))
            }
        }
        None => {
            let id = Uuid::new_v4();
            let filename = req.filename.unwrap_or_else(|| format!("{}.md", id));
            let simple_text = req.text.chars().take(500).collect::<String>();

            sqlx::query(
                r#"
                INSERT INTO notes (id, text, simple_text, filename, user_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(id)
            .bind(&req.text)
            .bind(&simple_text)
            .bind(&filename)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(|e| {
                error!("Database insert error: {:?}", e);
                ApiError::Internal(e.to_string())
            })?;

            Ok(axum::Json(ApiResponse::ok(SaveNoteResponse {
                id,
                message: "Note created successfully".to_string(),
            })))
        }
    }
}

pub async fn create_note(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    axum::Json(req): axum::Json<CreateNoteRequest>,
) -> Result<axum::Json<ApiResponse<CreateNoteResponse>>, ApiError> {
    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let filename = req.filename.unwrap_or_else(|| format!("{}.md", id));
    let simple_text = req.text.chars().take(500).collect::<String>();

    sqlx::query(
        r#"
        INSERT INTO notes (id, text, simple_text, filename, user_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(&req.text)
    .bind(&simple_text)
    .bind(&filename)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Database insert error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(axum::Json(ApiResponse::ok(CreateNoteResponse {
        id,
        message: "Note created successfully".to_string(),
    })))
}

pub async fn list_notes(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<ListQuery>,
) -> Result<axum::Json<ApiResponse<Vec<NoteSummary>>>, ApiError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let notes = sqlx::query_as::<_, NoteSummary>(
        r#"
        SELECT id, text, simple_text, filepath, filename, file_info, created_at, updated_at, sha256
        FROM notes
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database query error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(axum::Json(ApiResponse::ok(notes)))
}

pub async fn get_note(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Path(uuid): Path<Uuid>,
) -> Result<axum::Json<ApiResponse<NoteSummary>>, ApiError> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let note = sqlx::query_as::<_, NoteSummary>(
        r#"
        SELECT id, text, simple_text, filepath, filename, file_info, created_at, updated_at, sha256
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(uuid)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Database query error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    let note = note.ok_or_else(|| ApiError::not_found(ApiError::NOTE_NOT_FOUND, "Note not found"))?;

    Ok(axum::Json(ApiResponse::ok(note)))
}

pub async fn search_notes(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<SearchQuery>,
) -> Result<axum::Json<ApiResponse<Vec<NoteSummary>>>, ApiError> {
    let query = params.query;
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let notes = sqlx::query_as::<_, NoteSummary>(
        r#"
        SELECT id, text, simple_text, filepath, filename, file_info, created_at, updated_at, sha256
        FROM notes
        WHERE user_id = $1 AND (text ILIKE $2 OR simple_text ILIKE $2)
        LIMIT 50
        "#,
    )
    .bind(user_id)
    .bind(format!("%{query}%"))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Database query error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(axum::Json(ApiResponse::ok(notes)))
}

pub async fn preview_note(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Path(uuid): Path<Uuid>,
) -> Result<Response, ApiError> {
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();
    let note = sqlx::query_as::<_, Note>(
        r#"
        SELECT id, text, simple_text, raw_content, filepath, filename, file_info, created_at, updated_at, sha256
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#
    )
    .bind(uuid)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        eprintln!("Database query error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    let note = note.ok_or_else(|| ApiError::not_found(ApiError::NOTE_NOT_FOUND, "Note not found"))?;

    let raw_content = note
        .raw_content
        .ok_or_else(|| ApiError::not_found(ApiError::NOTE_CONTENT_NOT_FOUND, "No content found"))?;

    let filename = note.filename.clone().unwrap_or_else(|| "file".to_string());

    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let (content_type, content) = if extension.to_lowercase() == "mhtml" {
        let mhtml_str = String::from_utf8_lossy(&raw_content);
        let html_content = mhtml_to_html(&mhtml_str);
        (
            "text/html; charset=utf-8",
            axum::body::Body::from(html_content),
        )
    } else {
        let content_type = match extension.to_lowercase().as_str() {
            "pdf" => "application/pdf",
            "epub" => "application/epub+zip",
            "html" | "htm" => "text/html; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        };
        (content_type, axum::body::Body::from(raw_content))
    };

    let encoded_filename = utf8_percent_encode(&filename, NON_ALPHANUMERIC).to_string();
    let content_disposition = format!(
        "inline; filename=\"{}\"; filename*=UTF-8''{}",
        filename, encoded_filename
    );

    let mut response = Response::new(content);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(content_type),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        axum::http::HeaderValue::from_str(&content_disposition).unwrap(),
    );
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("public, max-age=31536000"),
    );
    Ok(response)
}

pub fn mhtml_to_html(mhtml: &str) -> String {
    let boundary = match extract_boundary(mhtml) {
        Some(b) => b,
        None => return mhtml.to_string(),
    };

    let delimiter = format!("--{}", boundary);
    let parts: Vec<&str> = mhtml.split(&delimiter).collect();

    let mut resources: HashMap<String, String> = HashMap::new();
    let mut html_content = String::new();

    for part in parts.iter().skip(1) {
        if part.trim() == "--" || part.trim().is_empty() {
            continue;
        }

        let (headers, body) = match split_header_body(part) {
            Some(hb) => hb,
            None => continue,
        };

        let mime_type = extract_header_value(headers, "Content-Type")
            .unwrap_or_default()
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();

        let encoding = extract_header_value(headers, "Content-Transfer-Encoding")
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        let location = extract_header_value(headers, "Content-Location")
            .unwrap_or_default()
            .trim()
            .to_string();

        let content_id = extract_header_value(headers, "Content-ID")
            .unwrap_or_default()
            .trim()
            .trim_matches('<')
            .trim_matches('>')
            .to_string();

        let body_clean = body.trim_end_matches("--").trim();

        if mime_type == "text/html" && html_content.is_empty() {
            html_content = decode_body(body_clean, &encoding);
        } else if !mime_type.is_empty() {
            let data_uri = encode_to_data_uri(body_clean, &mime_type, &encoding);
            if !location.is_empty() {
                resources.insert(location.clone(), data_uri.clone());
            }
            if !content_id.is_empty() {
                resources.insert(format!("cid:{}", content_id), data_uri);
            }
        }
    }

    for (reference, data_uri) in &resources {
        html_content = html_content.replace(reference, data_uri);
    }

    html_content
}

fn extract_boundary(mhtml: &str) -> Option<String> {
    for line in mhtml.lines() {
        let line = line.trim();
        if line.to_lowercase().contains("boundary=") {
            let boundary = if let Some(pos) = line.find("boundary=\"") {
                let start = pos + 10;
                let end = line[start..].find('"')? + start;
                line[start..end].to_string()
            } else if let Some(pos) = line.find("boundary=") {
                let start = pos + 9;
                line[start..].split(';').next()?.trim().to_string()
            } else {
                continue;
            };
            if !boundary.is_empty() {
                return Some(boundary);
            }
        }
    }
    None
}

fn split_header_body<'a>(part: &'a str) -> Option<(&'a str, &'a str)> {
    if let Some(pos) = part.find("\r\n\r\n") {
        Some((&part[..pos], &part[pos + 4..]))
    } else if let Some(pos) = part.find("\n\n") {
        Some((&part[..pos], &part[pos + 2..]))
    } else {
        None
    }
}

fn extract_header_value<'a>(headers: &'a str, key: &str) -> Option<String> {
    let key_lower = key.to_lowercase();
    let mut result = String::new();

    for line in headers.lines() {
        if line.to_lowercase().starts_with(&format!("{}:", key_lower)) {
            result = line[key.len() + 1..].trim().to_string();
        } else if !result.is_empty() && (line.starts_with(' ') || line.starts_with('\t')) {
            result.push(' ');
            result.push_str(line.trim());
        } else if !result.is_empty() {
            break;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn decode_body(body: &str, encoding: &str) -> String {
    match encoding {
        "base64" => {
            let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
            match general_purpose::STANDARD.decode(&cleaned) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                Err(_) => body.to_string(),
            }
        }
        "quoted-printable" => {
            match quoted_printable::decode(body.as_bytes(), quoted_printable::ParseMode::Robust) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                Err(_) => body.to_string(),
            }
        }
        _ => body.to_string(),
    }
}

fn encode_to_data_uri(body: &str, mime_type: &str, encoding: &str) -> String {
    let b64 = match encoding {
        "base64" => body
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>(),
        "quoted-printable" => {
            match quoted_printable::decode(body.as_bytes(), quoted_printable::ParseMode::Robust) {
                Ok(bytes) => general_purpose::STANDARD.encode(&bytes),
                Err(_) => general_purpose::STANDARD.encode(body.as_bytes()),
            }
        }
        _ => general_purpose::STANDARD.encode(body.as_bytes()),
    };

    format!("data:{};base64,{}", mime_type, b64)
}

pub fn note_routes() -> Router {
    Router::new()
        .route("/", get(list_notes).post(create_note))
        .route("/save", post(save_note))
        .route("/{uuid}", get(get_note))
        .route("/preview/{uuid}", get(preview_note))
        .route("/search", get(search_notes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::Json;
    use axum::extract::Query;

    // ---- 纯函数单元测试（不需要数据库）----

    #[test]
    fn test_extract_boundary_quoted() {
        let mhtml = "Content-Type: multipart/related; boundary=\"----=_NextPart_000\"\n\n";
        assert_eq!(extract_boundary(mhtml), Some("----=_NextPart_000".to_string()));
    }

    #[test]
    fn test_extract_boundary_unquoted() {
        let mhtml = "Content-Type: multipart/related; boundary=myboundary\n\n";
        assert_eq!(extract_boundary(mhtml), Some("myboundary".to_string()));
    }

    #[test]
    fn test_extract_boundary_none() {
        assert_eq!(extract_boundary("no boundary here"), None);
    }

    #[test]
    fn test_split_header_body_crlf() {
        let part = "Content-Type: text/html\r\n\r\n<html>body</html>";
        let (headers, body) = split_header_body(part).unwrap();
        assert_eq!(headers, "Content-Type: text/html");
        assert_eq!(body, "<html>body</html>");
    }

    #[test]
    fn test_split_header_body_lf() {
        let part = "Content-Type: text/html\n\n<html>body</html>";
        let (headers, body) = split_header_body(part).unwrap();
        assert_eq!(headers, "Content-Type: text/html");
        assert_eq!(body, "<html>body</html>");
    }

    #[test]
    fn test_extract_header_value_simple() {
        let headers = "Content-Type: text/html\nContent-Location: http://example.com";
        assert_eq!(
            extract_header_value(headers, "Content-Location"),
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn test_extract_header_value_case_insensitive() {
        let headers = "content-type: text/html";
        assert_eq!(
            extract_header_value(headers, "Content-Type"),
            Some("text/html".to_string())
        );
    }

    #[test]
    fn test_extract_header_value_missing() {
        let headers = "Content-Type: text/html";
        assert_eq!(extract_header_value(headers, "Content-Location"), None);
    }

    // ---- Handler 测试（需要数据库）----

    #[tokio::test]
    async fn test_create_and_get_note() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_note_user").await;
        let claims = test_claims(user_id, false);

        let req = CreateNoteRequest {
            text: "# 测试笔记\n\n这是测试内容。".to_string(),
            filename: None,
        };
        let result = create_note(claims.clone(), Extension(pool.clone()), Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap().0;
        assert_eq!(resp.msg, "ok");
        let note_id = resp.data.unwrap().id;

        // 获取
        let result2 = get_note(claims, Extension(pool.clone()), Path(note_id)).await;
        assert!(result2.is_ok());
        let note = result2.unwrap().0.data.unwrap();
        assert!(note.text.as_ref().unwrap().contains("测试笔记"));

        // 清理
        sqlx::query("DELETE FROM notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_list_notes() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_note_list_user").await;
        let claims = test_claims(user_id, false);

        // 创建几条笔记
        for i in 0..3 {
            let req = CreateNoteRequest {
                text: format!("笔记 {}", i),
                filename: None,
            };
            create_note(claims.clone(), Extension(pool.clone()), Json(req))
                .await
                .unwrap();
        }

        let params = ListQuery {
            page: Some(1),
            limit: Some(10),
        };
        let result = list_notes(claims, Extension(pool.clone()), Query(params)).await;
        assert!(result.is_ok());
        let notes = result.unwrap().0.data.unwrap();
        assert!(notes.len() >= 3);

        // 清理
        sqlx::query("DELETE FROM notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_search_notes() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_note_search_user").await;
        let claims = test_claims(user_id, false);

        let req = CreateNoteRequest {
            text: "Rust是一门系统编程语言".to_string(),
            filename: None,
        };
        create_note(claims.clone(), Extension(pool.clone()), Json(req))
            .await
            .unwrap();

        let params = SearchQuery {
            query: "系统编程".to_string(),
        };
        let result = search_notes(claims, Extension(pool.clone()), Query(params)).await;
        assert!(result.is_ok());
        let notes = result.unwrap().0.data.unwrap();
        assert!(!notes.is_empty());
        assert!(notes[0].simple_text.as_ref().unwrap().contains("系统编程"));

        // 清理
        sqlx::query("DELETE FROM notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_note_not_found() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_note_404_user").await;
        let claims = test_claims(user_id, false);
        let fake_id = Uuid::now_v7();

        let result = get_note(claims, Extension(pool.clone()), Path(fake_id)).await;
        assert!(result.is_err());

        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_save_note_create_and_update() {
        let pool = test_pool().await;
        let user_id = create_test_user(&pool, "test_note_save_user").await;
        let claims = test_claims(user_id, false);

        // 首次保存（无 id，创建新笔记）
        let req = SaveNoteRequest {
            id: None,
            text: "初始内容".to_string(),
            filename: None,
        };
        let result = save_note(claims.clone(), Extension(pool.clone()), Json(req)).await;
        assert!(result.is_ok());
        let note_id = result.unwrap().0.data.unwrap().id;

        // 更新已有笔记
        let req2 = SaveNoteRequest {
            id: Some(note_id),
            text: "更新后内容".to_string(),
            filename: None,
        };
        let result2 = save_note(claims.clone(), Extension(pool.clone()), Json(req2)).await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().0.data.unwrap().message.contains("updated"));

        // 清理
        sqlx::query("DELETE FROM notes WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_test_user(&pool, user_id).await;
    }
}
