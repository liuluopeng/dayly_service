use serde_json::Value;

use crate::{
    api::base::{ApiResponse, ApiResult, PaginatedResponse},
    api::client::ApiClient,
};

use my_type::dto::videos::VideoWithUrl;

/// 扫描视频
pub async fn scan_videos(client: &ApiClient) -> ApiResult<ApiResponse<Value>> {
    let response = client
        .post("/api/videos/scan", &())
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(data))
}

/// 获取视频列表（支持分页和文件夹过滤）
pub async fn list_videos(
    client: &ApiClient,
    folder: Option<&str>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> ApiResult<ApiResponse<PaginatedResponse<VideoWithUrl>>> {
    let mut path = "/api/videos/list".to_string();

    let mut params = Vec::new();
    if let Some(f) = folder {
        params.push(format!("folder={}", urlencoding::encode(f)));
    }
    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(ps) = page_size {
        params.push(format!("page_size={}", ps));
    }

    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }

    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}
