use my_type::dto;
use my_type::model::dict::{ModernChineseWord, Word, WordHistory};

use super::base::{ApiResponse, ApiResult};
use super::client::ApiClient;
use serde_json::Value;

pub use dto::{
    DictCandidateResponse, DictExactResponse, DictSearchQuery, DictSearchResult, RecentHistoryQuery,
};

/// 搜索现代汉语词典
pub async fn search_xiandaihanyu(
    client: &ApiClient,
    query: &str,
) -> ApiResult<ApiResponse<String>> {
    let path = format!("/api/dict/xiandaihanyu?search={}", query);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let text = response
        .text()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(text))
}

/// 搜索柯林斯词典
pub async fn search_collins(client: &ApiClient, query: &str) -> ApiResult<ApiResponse<String>> {
    let path = format!("/api/dict/collins?search={}", query);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let text = response
        .text()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(text))
}

/// 获取柯林斯词典资源
pub async fn collins_resource(
    client: &ApiClient,
    resource_path: &str,
) -> ApiResult<ApiResponse<Vec<u8>>> {
    let path = format!("/api/dict/collins_resources/{}", resource_path);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(bytes.to_vec()))
}

/// 搜索朗文词典
pub async fn search_ldoce(client: &ApiClient, query: &str) -> ApiResult<ApiResponse<String>> {
    let path = format!("/api/dict/ldoce?search={}", query);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let text = response
        .text()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(text))
}

/// 获取朗文词典资源
pub async fn ldoce_resource(
    client: &ApiClient,
    resource_path: &str,
) -> ApiResult<ApiResponse<Vec<u8>>> {
    let path = format!("/api/dict/ldoce_resources/{}", resource_path);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(bytes.to_vec()))
}

/// 获取现代汉语词典资源
pub async fn xiandaihanyu_resource(
    client: &ApiClient,
    resource_path: &str,
) -> ApiResult<ApiResponse<Vec<u8>>> {
    let path = format!("/api/dict/xiandaihanyu_resources/{}", resource_path);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::ok(bytes.to_vec()))
}

/// 获取最近搜索历史
pub async fn get_recent_history(
    client: &ApiClient,
    limit: i64,
) -> ApiResult<ApiResponse<Vec<WordHistory>>> {
    let path = format!("/api/dict/recent-history?limit={}", limit);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let history: ApiResponse<Vec<WordHistory>> = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(history)
}

/// 获取热门词语
pub async fn get_top_words(client: &ApiClient) -> ApiResult<ApiResponse<Vec<Word>>> {
    let response = client
        .get("/api/dict/top-words")
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let words: ApiResponse<Vec<Word>> = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(words)
}

pub async fn word_search_count(
    client: &ApiClient,
    word: &str,
) -> ApiResult<ApiResponse<i64>> {
    let path = format!("/api/dict/word-count?word={}", urlencoding::encode(word));
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    let count: ApiResponse<i64> = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(e.to_string()))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_xiandaihanyu() {
        let client = ApiClient::default();

        match search_xiandaihanyu(&client, "市场").await {
            Ok(response) => {
                println!("搜索现代汉语词典成功！响应: {}", response);
            }
            Err(error) => {
                println!("搜索现代汉语词典失败: {}", error);
            }
        }
    }

    #[tokio::test]
    async fn test_search_collins() {
        let client = ApiClient::default();

        match search_collins(&client, "hello").await {
            Ok(response) => {
                println!("搜索柯林斯词典成功！响应: {}", response);
            }
            Err(error) => {
                println!("搜索柯林斯词典失败: {}", error);
            }
        }
    }

    #[tokio::test]
    async fn test_search_ldoce() {
        let client = ApiClient::default();

        match search_ldoce(&client, "world").await {
            Ok(response) => {
                println!("搜索朗文词典成功！响应: {}", response);
            }
            Err(error) => {
                println!("搜索朗文词典失败: {}", error);
            }
        }
    }

    #[tokio::test]
    async fn test_get_recent_history() {
        let client = ApiClient::default();

        match get_recent_history(&client, 10).await {
            Ok(response) => {
                println!("获取最近搜索历史成功！");
                println!("响应消息: {}", response.msg);
                if let Some(data) = response.data {
                    println!("获取到 {} 条历史记录", data.len());
                }
            }
            Err(error) => {
                println!("获取最近搜索历史失败: {}", error);
            }
        }
    }

    #[tokio::test]
    async fn test_get_top_words() {
        let client = ApiClient::default();

        match get_top_words(&client).await {
            Ok(response) => {
                println!("获取热门词语成功！");
                println!("响应消息: {}", response.msg);
                if let Some(data) = response.data {
                    println!("获取到 {} 个热门词语", data.len());
                }
            }
            Err(error) => {
                println!("获取热门词语失败: {}", error);
            }
        }
    }
}
