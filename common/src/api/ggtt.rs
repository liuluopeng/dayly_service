use crate::api::base::{ApiError, ApiResponse, ApiResult};
use my_type::dto;
use my_type::model::ggtt::GgttCode;

use super::client::ApiClient;

pub use dto::SearchRequest;

/// 搜索 GGTT 五笔编码
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

/// 搜索 GGTT 编码（WASM 封装版）
pub async fn search_ggtt_code_for_wasm(req: SearchRequest) -> ApiResult<ApiResponse<GgttCode>> {
    // 复用统一客户端（DAYLY_API_URL 环境变量或同源相对路径，不写死端口）
    let client = ApiClient::default();
    search_ggtt_code(&client, req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_ggtt_code() {
        let client = ApiClient::test_client();
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
