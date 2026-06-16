use common::api::base::{ApiError, ApiResponse, ApiResult};

use axum::Json;
use axum::extract::{Extension, Path, Query};
use my_type::model::single_char_pinyin::SingleCharPinyin;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct PinyinQuery {
    pub ori: String,
}

/// 根据拼音 key 返回字典数据
pub async fn get_pinyin_dict(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<PinyinQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let key = query.ori.trim().to_lowercase();
    let words: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT words FROM pinyin_dict WHERE pinyin = $1"
    )
    .bind(&key)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(words.unwrap_or(serde_json::Value::Array(vec![]))))
}

pub async fn get_pinyin_by_ori(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<PinyinQuery>,
) -> ApiResult<Json<Vec<String>>> {
    let ori = query.ori.trim();

    let pinyin_list = sqlx::query_as::<_, SingleCharPinyin>(
        "SELECT id, pinyin, ori, count, pinyin_length, first_letter FROM single_char_pinyin WHERE ori = $1",
    )
    .bind(ori)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let result: Vec<String> = pinyin_list.into_iter().map(|item| item.pinyin).collect();

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_pool;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_get_pinyin_by_ori() {
        let pool = test_pool().await;

        let query = PinyinQuery {
            ori: "yi".to_string(),
        };
        let result = get_pinyin_by_ori(Extension(pool), Query(query))
            .await
            .expect("Failed to get pinyin");

        let pinyin_list = result.0;
        debug!("Pinyin for 'yi': {:?}", pinyin_list);

        // 使用集合来验证结果，确保不多不少正好是期望的四个拼音
        let expected_pinyin: HashSet<_> = vec!["yì", "yí", "yī", "yǐ"]
            .into_iter()
            .map(String::from)
            .collect();
        let actual_pinyin: HashSet<_> = pinyin_list.into_iter().collect();

        assert_eq!(
            actual_pinyin, expected_pinyin,
            "Pinyin list doesn't match expected set"
        );
    }
}

pub fn pinyin_routes() -> axum::Router {
    axum::Router::new()
        .route("/get-by-ori", axum::routing::get(get_pinyin_by_ori))
        .route("/dict", axum::routing::get(get_pinyin_dict))
}
