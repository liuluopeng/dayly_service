use crate::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::dto;
use my_type::model::ggtt::GgttCode;

use super::client::ApiClient;

pub use dto::SearchRequest;

pub async fn search_ggtt_code(
    client: &ApiClient,
    req: SearchRequest,
) -> ApiResult<ApiResponse<GgttCode>> {
    let response = client
        .post("/api/ggtt/search_ggtt", req)
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status, body
        )));
    }

    let api_response: ApiResponse<GgttCode> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

pub async fn search_ggtt_code_for_wasm(req: SearchRequest) -> ApiResult<ApiResponse<GgttCode>> {
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Duration;

    #[cfg(not(target_arch = "wasm32"))]
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    #[cfg(target_arch = "wasm32")]
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let base_url = "http://192.168.31.58:23001";
    let url = format!("{}{}", base_url, "/api/ggtt/search_ggtt");

    #[cfg(not(target_arch = "wasm32"))]
    let response = client
        .post(&url)
        .json(&req)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    #[cfg(target_arch = "wasm32")]
    let response = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(ApiError::Internal(format!(
            "Request failed with status {}: {}",
            status, body
        )));
    }

    let api_response: ApiResponse<GgttCode> = serde_json::from_str(&body)
        .map_err(|e| ApiError::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(api_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_ggtt_code() {
        let client = ApiClient::tester_client();
        let req = SearchRequest {
            search: "当".to_string(),
        };

        match search_ggtt_code(&client, req).await {
            Ok(response) => {
                println!("请求成功！");
                println!("响应消息: {}", response.msg);
                if let Some(code) = response.data {
                    println!("响应数据: {}", code);
                }
            }
            Err(error) => {
                println!("请求失败: {}", error);
            }
        }
    }
}
