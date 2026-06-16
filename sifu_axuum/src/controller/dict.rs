use crate::middleware::Claims;
use axum::extract::{Extension, Path, Query};
use axum::{Json, response::Response};
use common::api::base::ApiError;
use common::api::base::ApiResponse;
use common::api::base::ApiResult;
use common::api::dict::{DictSearchQuery, RecentHistoryQuery};
use my_type::model::dict::DictWord;
use my_type::model::dict::ModernChineseWord;
use my_type::model::dict::WordHistory;
use my_type::model::dict::{DictResource, Word};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

static DICT_POOLS: OnceLock<Mutex<HashMap<String, sqlx::SqlitePool>>> = OnceLock::new();
static STATIC_BASE: OnceLock<String> = OnceLock::new();

pub fn set_static_dir(dir: &str) {
    STATIC_BASE.set(dir.to_string()).ok();
}

fn static_path(subdir: &str, resource: &str) -> std::path::PathBuf {
    let base = STATIC_BASE.get().map(|s| s.as_str()).unwrap_or("static");
    std::path::Path::new(base).join(subdir).join(resource)
}

/// 初始化词典 SQLite — 按表名映射到独立 DB 文件
pub async fn init_dict_db(base_dir: &str) {
    let map: Vec<(String, sqlx::SqlitePool)> = futures::future::join_all([
        ("collins_words", format!("{}/collins_words.db", base_dir)),
        ("collins_resources", format!("{}/collins_resources.db", base_dir)),
        ("ldoce_words", format!("{}/ldoce_words.db", base_dir)),
        ("ldoce_resources", format!("{}/ldoce_resources.db", base_dir)),
        ("modern_chinese_words", format!("{}/modern_chinese_words.db", base_dir)),
        ("ggtt_codes", format!("{}/ggtt_codes.db", base_dir)),
    ].iter().map(|(table, path)| async move {
        let pool = SqlitePoolOptions::new().max_connections(2).connect(path).await
            .unwrap_or_else(|e| panic!("连接 {} 失败: {}", path, e));
        (table.to_string(), pool)
    })).await;
    DICT_POOLS.set(Mutex::new(map.into_iter().collect())).ok();
    info!("词典 SQLite 已初始化 ({} 文件)", 5);
}

async fn lookup_word(table: &str, word: &str) -> Option<String> {
    let pools = DICT_POOLS.get()?.lock().await;
    let pool = pools.get(table)?;
    let sql = format!("SELECT explanation FROM {} WHERE word = ? LIMIT 1", table);
    sqlx::query_scalar::<_, String>(&sql).bind(word).fetch_optional(pool).await.ok().flatten()
}

pub async fn lookup_ggtt_char(ch: &str) -> Option<(String, Option<String>, Option<String>, Option<String>, Option<String>, bool)> {
    let pools = DICT_POOLS.get()?.lock().await;
    let pool = pools.get("ggtt_codes")?;
    sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>, bool)>(
        "SELECT code_86, svg1, svg2, svg3, svg4, has_diagram FROM ggtt_codes WHERE char = ?"
    ).bind(ch).fetch_optional(pool).await.ok().flatten()
}

async fn lookup_resource(table: &str, path: &str) -> Option<Vec<u8>> {
    let pools = DICT_POOLS.get()?.lock().await;
    let pool = pools.get(table)?;
    let sql = format!("SELECT resource_data FROM {} WHERE resource_path = ? LIMIT 1", table);
    sqlx::query_scalar::<_, Vec<u8>>(&sql).bind(path).fetch_optional(pool).await.ok().flatten()
}
// 定义响应结构体
fn get_server_base_url() -> String {
    let host = std::env::var("DOMAIN").unwrap_or_else(|_| "192.168.31.58".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "23000".to_string());
    format!("http://{}:{}", host, port)
}

fn process_entry_links(record: String, base_url: &str, dict_type: &str) -> String {
    let regex_entry = Regex::new(r#"<a\s+([^>]*?)href="?entry://([^"\s>]+)"?([^>]*?)>"#).unwrap();
    let api_url = format!("{}/api/dict/{}", base_url, dict_type);

    regex_entry
        .replace_all(&record, |caps: &regex::Captures| {
            let before_href = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let word = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let after_href = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            let new_href = format!("href=\"{}?search={}\"", api_url, word);

            format!("<a {}{}{}>", before_href, new_href, after_href)
        })
        .to_string()
}

fn process_sound_links(record: String, base_url: &str) -> String {
    let regex_a_tag = Regex::new(
        r#"<a\s+([^>]*?)addr="?([^"\s>]+)"?([^>]*?)href="?sound://([^"\s>]+)"?([^>]*?)>"#,
    )
    .unwrap();
    let sound_resource_url = format!("{}/collins_resources", base_url);

    regex_a_tag
        .replace_all(&record, |caps: &regex::Captures| {
            let before_addr = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let addr_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let middle = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let href_value = caps.get(4).map(|m| m.as_str()).unwrap_or("");
            let after_href = caps.get(5).map(|m| m.as_str()).unwrap_or("");

            let data_sound_url =
                format!(" data-sound-url=\"{}/{}\"", sound_resource_url, href_value);

            format!(
                "<a {}addr=\"{}\"{}href=sound://{}{}{}>",
                before_addr, addr_value, middle, href_value, data_sound_url, after_href
            )
        })
        .to_string()
}

fn process_ldoce_sound_links(record: String, base_url: &str) -> String {
    let regex_a_tag = Regex::new(r#"<a\s+([^>]*?)href="?sound://([^"\s>]+)"?([^>]*?)>"#).unwrap();
    let sound_resource_url = format!("{}/ldoce_resources", base_url);

    regex_a_tag
        .replace_all(&record, |caps: &regex::Captures| {
            let before_href = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let href_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let after_href = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            let data_sound_url =
                format!(" data-sound-url=\"{}/{}\"", sound_resource_url, href_value);

            format!(
                "<a {}href=sound://{}{}{}>",
                before_href, href_value, data_sound_url, after_href
            )
        })
        .to_string()
}

fn process_collins_resources(record: String, base_url: &str) -> String {
    let collins_resources_url = format!("{}/collins_resources", base_url);

    let record = record.replace(
        "href=\"colcobuildstyle.css\"",
        &format!("href=\"{}/colcobuildstyle.css\"", collins_resources_url),
    );
    let record = record.replace(
        "href='colcobuildstyle.css'",
        &format!("href='{}/colcobuildstyle.css'", collins_resources_url),
    );
    let record = record.replace(
        "href=colcobuildstyle.css",
        &format!("href={}/colcobuildstyle.css", collins_resources_url),
    );
    let record = record.replace(
        "href=\"colcobuildstyle_upgrade.css\"",
        &format!(
            "href=\"{}/colcobuildstyle_upgrade.css\"",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "href='colcobuildstyle_upgrade.css'",
        &format!(
            "href='{}/colcobuildstyle_upgrade.css'",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "href=colcobuildstyle_upgrade.css",
        &format!("href={}/colcobuildstyle_upgrade.css", collins_resources_url),
    );
    let record = record.replace(
        "href=\"colcobuildstyle_switch.css\"",
        &format!(
            "href=\"{}/colcobuildstyle_switch.css\"",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "href='colcobuildstyle_switch.css'",
        &format!(
            "href='{}/colcobuildstyle_switch.css'",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "href=colcobuildstyle_switch.css",
        &format!("href={}/colcobuildstyle_switch.css", collins_resources_url),
    );
    let record = record.replace(
        "href=\"colcobuildstyle_show.css\"",
        &format!(
            "href=\"{}/colcobuildstyle_show.css\"",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "href='colcobuildstyle_show.css'",
        &format!("href='{}/colcobuildstyle_show.css'", collins_resources_url),
    );
    let record = record.replace(
        "href=colcobuildstyle_show.css",
        &format!("href={}/colcobuildstyle_show.css", collins_resources_url),
    );
    let record = record.replace(
        "src=\"colcobuildoverhaul_switch.js\"",
        &format!(
            "src=\"{}/colcobuildoverhaul_switch.js\"",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "src='colcobuildoverhaul_switch.js'",
        &format!(
            "src='{}/colcobuildoverhaul_switch.js'",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "src=colcobuildoverhaul_switch.js",
        &format!("src={}/colcobuildoverhaul_switch.js", collins_resources_url),
    );
    let record = record.replace(
        "src=\"colcobuildoverhaul_config.ini\"",
        &format!(
            "src=\"{}/colcobuildoverhaul_config.ini\"",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "src='colcobuildoverhaul_config.ini'",
        &format!(
            "src='{}/colcobuildoverhaul_config.ini'",
            collins_resources_url
        ),
    );
    let record = record.replace(
        "src=colcobuildoverhaul_config.ini",
        &format!(
            "src={}/colcobuildoverhaul_config.ini",
            collins_resources_url
        ),
    );

    record
}

fn process_ldoce_resources(record: String, base_url: &str) -> String {
    let ldoce_resources_url = format!("{}/ldoce_resources", base_url);

    let record = Regex::new(r#"href=("[^"]*"|'[^']*'|[^\s">]+)"#)
        .unwrap()
        .replace_all(&record, |caps: &regex::Captures| {
            let href_value = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let has_quotes = href_value.starts_with('"') || href_value.starts_with("'");
            let clean_value = if has_quotes {
                &href_value[1..href_value.len() - 1]
            } else {
                href_value
            };

            if clean_value.starts_with("sound://")
                || clean_value.starts_with("entry://")
                || clean_value.starts_with("http://")
                || clean_value.starts_with("https://")
            {
                format!("href={}", href_value)
            } else {
                if has_quotes {
                    format!("href=\"{}/{}\"", ldoce_resources_url, clean_value)
                } else {
                    format!("href={}/{}", ldoce_resources_url, clean_value)
                }
            }
        })
        .to_string();

    let record = Regex::new(r#"src=("[^"]*"|'[^']*'|[^\s">]+)"#)
        .unwrap()
        .replace_all(&record, |caps: &regex::Captures| {
            let src_value = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let has_quotes = src_value.starts_with('"') || src_value.starts_with("'");
            let clean_value = if has_quotes {
                &src_value[1..src_value.len() - 1]
            } else {
                src_value
            };

            if clean_value.starts_with("sound://")
                || clean_value.starts_with("http://")
                || clean_value.starts_with("https://")
            {
                format!("src={}", src_value)
            } else {
                if has_quotes {
                    format!("src=\"{}/{}\"", ldoce_resources_url, clean_value)
                } else {
                    format!("src={}/{}", ldoce_resources_url, clean_value)
                }
            }
        })
        .to_string();

    record
}

fn process_xiandaihanyu_resources(record: String, base_url: &str) -> String {
    let xiandaihanyu_resources_url = format!("{}/xiandaihanyu_resources", base_url);

    let record = record.replace(
        "href=\"hycd.css\"",
        &format!("href=\"{}/hycd.css\"", xiandaihanyu_resources_url),
    );
    let record = record.replace(
        "href='hycd.css'",
        &format!("href='{}/hycd.css'", xiandaihanyu_resources_url),
    );
    let record = record.replace(
        "href=hycd.css",
        &format!("href={}/hycd.css", xiandaihanyu_resources_url),
    );

    record
}

fn process_xiandaihanyu_html(record: String, base_url: &str) -> String {
    let record = process_xiandaihanyu_resources(record, &base_url);
    let record = process_entry_links(record, &base_url, "xiandaihanyu");
    record
}

async fn record_word_history(pool: &PgPool, word: &str, user_id: Uuid) -> ApiResult<()> {
    let word = word.to_string();

    // 更新 words 表中的搜索次数
    sqlx::query(
        "INSERT INTO words (word, has_searched_times) VALUES ($1, 1)
         ON CONFLICT (word) DO UPDATE SET has_searched_times = words.has_searched_times + 1",
    )
    .bind(&word)
    .execute(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 向 word_histories 表中插入历史记录
    sqlx::query("INSERT INTO word_histories (word, time, user_id) VALUES ($1, NOW(), $2)")
        .bind(&word)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn search_xiandaihanyu(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<DictSearchQuery>,
) -> ApiResult<Response> {
    let word = query.search.trim();
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let explanation = lookup_word("modern_chinese_words", word).await;
    let xiandaihanyu_result = explanation.map(|e| ModernChineseWord { word: word.to_string(), explanation: e });

    let base_url = get_server_base_url();

    let html_content = if let Some(result) = xiandaihanyu_result {
        record_word_history(&pool, word, user_id).await?;
        process_xiandaihanyu_html(result.explanation, &base_url)
    } else {
        String::new()
    };

    let mut response = Response::new(axum::body::Body::from(html_content));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(response)
}

pub async fn search_collins(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<DictSearchQuery>,
) -> ApiResult<Response> {
    let word = query.search.trim().to_lowercase();
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let collins_result = lookup_word("collins_words", &word).await
        .map(|e| DictWord { word: word.clone(), explanation: e });

    let base_url = get_server_base_url();

    let js_code = r#"<script>
(function() {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initSoundLinks);
    } else {
        initSoundLinks();
    }

    function initSoundLinks() {
        var soundLinks = document.querySelectorAll('a[href^="sound://"]');
        soundLinks.forEach(function(element) {
            var newElement = element.cloneNode(true);
            element.parentNode.replaceChild(newElement, element);
            newElement.addEventListener('click', function(event) {
                event.preventDefault();
                event.stopPropagation();
                event.stopImmediatePropagation();
                var url = newElement.getAttribute('data-sound-url');
                if (url) {
                    var audio = new Audio(url);
                    audio.play().catch(function(error) {
                        console.error('Audio play failed:', error);
                    });
                }
            });
        });
    }
})();
</script>"#;

    let html_content = if let Some(result) = collins_result {
        record_word_history(&pool, &word, user_id).await?;
        let record = result.explanation;
        let record = process_sound_links(record, &base_url);
        let record = process_collins_resources(record, &base_url);
        let record = process_entry_links(record, &base_url, "collins");

        let head_regex = Regex::new(r#"<head\s*([^>]*?)>"#).unwrap();
        let processed_html = if head_regex.is_match(&record) {
            head_regex
                .replace(&record, |caps: &regex::Captures| {
                    let head_attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    format!(
                        "<head {}>{}{}",
                        head_attrs,
                        js_code,
                        &record[caps.get(0).unwrap().as_str().len()..]
                    )
                })
                .to_string()
        } else {
            format!("{}{}", js_code, record)
        };

        format!(
            "{}<script src={}/collins_resources/jquery-3.5.1/jquery.min.js></script><script src={}/collins_resources/bootstrap-4.5.0/js/bootstrap.bundle.min.js></script>",
            processed_html, base_url, base_url
        )
    } else {
        String::new()
    };

    let mut response = Response::new(axum::body::Body::from(html_content));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(response)
}

pub async fn collins_resource(
    // dict functions use SQLite, not PgPool
    Path(resource_path): Path<String>,
) -> ApiResult<Response> {
    let mut resource_path = resource_path.trim().to_string();

    // 去除末尾的斜杠
    while resource_path.ends_with('/') {
        resource_path.pop();
    }

    let disk_path = static_path("collins_resources", &resource_path);

    if disk_path.exists() {
        if let Ok(contents) = std::fs::read(&disk_path) {
            let content_type = get_content_type(&resource_path);
            let mut response = Response::new(axum::body::Body::from(contents));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=31536000"),
            );
            return Ok(response);
        }
    }

    if let Some(data) = lookup_resource("collins_resources", &resource_path).await {
            let content_type = get_content_type(&resource_path);
            let mut response = Response::new(axum::body::Body::from(data));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=31536000"),
            );
            return Ok(response);
        }
    Err(ApiError::not_found(ApiError::RESOURCE_NOT_FOUND, "Resource not found"))
}

pub async fn search_ldoce(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<DictSearchQuery>,
) -> ApiResult<Response> {
    let word = query.search.trim().to_lowercase();
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let ldoce_result = lookup_word("ldoce_words", &word).await
        .map(|e| DictWord { word: word.clone(), explanation: e });

    let base_url = get_server_base_url();

    let js_code = r#"<script>
(function() {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initSoundLinks);
    } else {
        initSoundLinks();
    }

    function initSoundLinks() {
        var soundLinks = document.querySelectorAll('a[href^="sound://"]');
        soundLinks.forEach(function(element) {
            var newElement = element.cloneNode(true);
            element.parentNode.replaceChild(newElement, element);
            newElement.addEventListener('click', function(event) {
                event.preventDefault();
                event.stopPropagation();
                var url = newElement.getAttribute('data-sound-url');
                if (url) {
                    var audio = new Audio(url);
                    audio.play().catch(function(error) {
                        console.error('Audio play failed:', error);
                    });
                }
            });
        });
    }
})();
</script>"#;

    let html_content = if let Some(result) = ldoce_result {
        record_word_history(&pool, &word, user_id).await?;
        let record = result.explanation;
        let record = process_ldoce_sound_links(record, &base_url);
        let record = process_ldoce_resources(record, &base_url);
        let record = process_entry_links(record, &base_url, "ldoce");

        let head_regex = Regex::new(r#"<head\s*([^>]*?)>"#).unwrap();
        let processed_html = if head_regex.is_match(&record) {
            head_regex
                .replace(&record, |caps: &regex::Captures| {
                    let head_attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    format!(
                        "<head {}>{}{}",
                        head_attrs,
                        js_code,
                        &record[caps.get(0).unwrap().as_str().len()..]
                    )
                })
                .to_string()
        } else {
            format!("{}{}", js_code, record)
        };

        format!(
            "{}<script src={}/ldoce_resources/jquery-3.5.1/jquery.min.js></script><script src={}/ldoce_resources/bootstrap-4.5.0/js/bootstrap.bundle.min.js></script>",
            processed_html, base_url, base_url
        )
    } else {
        String::new()
    };

    let mut response = Response::new(axum::body::Body::from(html_content));
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(response)
}

pub async fn ldoce_resource(
    // dict functions use SQLite, not PgPool
    Path(resource_path): Path<String>,
) -> ApiResult<Response> {
    let mut resource_path = resource_path.trim().to_string();

    // 去除末尾的斜杠
    while resource_path.ends_with('/') {
        resource_path.pop();
    }

    let disk_path = static_path("ldoce_resources", &resource_path);

    if disk_path.exists() {
        if let Ok(contents) = std::fs::read(&disk_path) {
            let content_type = get_content_type(&resource_path);
            let mut response = Response::new(axum::body::Body::from(contents));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=31536000"),
            );
            return Ok(response);
        }
    }

    if let Some(data) = lookup_resource("ldoce_resources", &resource_path).await {
        let content_type = get_content_type(&resource_path);
        let mut response = Response::new(axum::body::Body::from(data));
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static(content_type),
        );
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=31536000"),
        );
        return Ok(response);
    }
    Err(ApiError::not_found(ApiError::RESOURCE_NOT_FOUND, "Resource not found"))
}

pub async fn xiandaihanyu_resource(Path(resource_path): Path<String>) -> ApiResult<Response> {
    let mut resource_path = resource_path.trim().to_string();

    // 去除末尾的斜杠
    while resource_path.ends_with('/') {
        resource_path.pop();
    }

    let disk_path = static_path("ldoce_resources", &resource_path);

    if disk_path.exists() {
        if let Ok(contents) = std::fs::read(&disk_path) {
            let content_type = get_content_type(&resource_path);
            let mut response = Response::new(axum::body::Body::from(contents));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=31536000"),
            );
            return Ok(response);
        } else {
            return Err(ApiError::not_found(ApiError::RESOURCE_NOT_FOUND, "Resource not found"));
        }
    } else {
        return Err(ApiError::not_found(ApiError::RESOURCE_NOT_FOUND, "Resource not found"));
    }
}

pub async fn get_recent_history(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<RecentHistoryQuery>,
) -> ApiResult<ApiResponse<Vec<WordHistory>>> {
    let limit = query.limit;
    let user_id = Uuid::parse_str(&claims.id).unwrap_or_default();

    let histories = sqlx::query_as::<_, WordHistory>(
        "SELECT id, word, time, created_at FROM word_histories WHERE user_id = $1 ORDER BY time DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    debug!("Found {} history records", histories.len());
    Ok(ApiResponse::ok(histories))
}

pub async fn get_top_words(
    Extension(pool): Extension<PgPool>,
) -> ApiResult<ApiResponse<Vec<Word>>> {
    let words = sqlx::query_as::<_, Word>(
        "SELECT id, word, has_searched_times FROM words ORDER BY has_searched_times DESC LIMIT 100",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    debug!("Found {} words", words.len());
    Ok(ApiResponse::ok(words))
}

// 资源文件路由，使用绝对路径
pub fn dict_resource_routes() -> axum::Router {
    axum::Router::new()
        .route("/collins_resources/{*resource_path}", axum::routing::get(collins_resource))
        .route("/collins_resource/{*resource_path}", axum::routing::get(collins_resource))
        .route("/ldoce_resources/{*resource_path}", axum::routing::get(ldoce_resource))
        .route("/xiandaihanyu_resources/{*resource_path}", axum::routing::get(xiandaihanyu_resource))
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".mp3") {
        "audio/mpeg"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".eot") {
        "application/vnd.ms-fontobject"
    } else {
        "application/octet-stream"
    }
}

pub fn dict_routes() -> axum::Router {
    axum::Router::new()
        .route("/collins", axum::routing::get(search_collins))
        .route("/ldoce", axum::routing::get(search_ldoce))
        .route("/xiandaihanyu", axum::routing::get(search_xiandaihanyu))
        .route("/recent-history", axum::routing::get(get_recent_history))
        .route("/top-words", axum::routing::get(get_top_words))
}
