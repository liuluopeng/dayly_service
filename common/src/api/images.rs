use serde_json::Value;

use crate::{
    api::base::{ApiResponse, ApiResult, PaginatedResponse},
    api::client::ApiClient,
};

use my_type::dto::images::ImageWithUrl;

/// 扫描图片
pub async fn scan_images(client: &ApiClient) -> ApiResult<ApiResponse<Value>> {
    let response = client
        .post("/api/images/scan", &())
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(data))
}

/// 获取图片文件夹列表
pub async fn get_image_folders(client: &ApiClient) -> ApiResult<ApiResponse<Vec<String>>> {
    let response = client
        .get("/api/images/folders")
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

/// 获取图片列表（支持分页和文件夹过滤）
pub async fn list_images(
    client: &ApiClient,
    folder: Option<&str>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> ApiResult<ApiResponse<PaginatedResponse<ImageWithUrl>>> {
    let mut path = "/api/images/list".to_string();

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
