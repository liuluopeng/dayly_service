//! zici 字词学习 API（数据在服务端 sqlite，前端通过接口查询）

use crate::api::base::{ApiResponse, ApiResult};
use crate::api::client::ApiClient;
use serde_json::Value;

fn urlencode(s: &str) -> String {
    urlencoding::encode(s).to_string()
}

/// 生字表：按年级/学期（grade 1-6，term 1-2）
pub async fn zici_chars(
    client: &ApiClient,
    grade: u32,
    term: u32,
) -> ApiResult<ApiResponse<Vec<String>>> {
    let resp = client
        .get(&format!("/api/zici/chars?grade={}&term={}", grade, term))
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    client.json_response::<Vec<String>>(resp).await
}

/// 生词表：搜索 + 分页
pub async fn zici_words(
    client: &ApiClient,
    search: &str,
    page: u32,
    page_size: u32,
) -> ApiResult<ApiResponse<serde_json::Value>> {
    let mut path = format!("/api/zici/words?page={}&page_size={}", page, page_size);
    if !search.is_empty() {
        path.push_str(&format!("&search={}", urlencode(search)));
    }
    let resp = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    client.json_response::<serde_json::Value>(resp).await
}

/// 词频搜索（含释义）
pub async fn zici_word_frequency(
    client: &ApiClient,
    search: &str,
    limit: u32,
) -> ApiResult<ApiResponse<Vec<Value>>> {
    let mut path = format!("/api/zici/word-frequency?limit={}", limit);
    if !search.is_empty() {
        path.push_str(&format!("&search={}", urlencode(search)));
    }
    let resp = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    client.json_response::<Vec<Value>>(resp).await
}
