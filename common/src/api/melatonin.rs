use my_type::model::melatonin_movie::{MelatoninMovie, MelatoninMovieList};

use super::base::{ApiResponse, ApiResult, PaginatedResponse};
use super::client::ApiClient;
use serde_json::Value;
use uuid::Uuid;

pub use my_type::dto::{ActorMovieQuery, MelatoninListQuery, ScanMelatoninQuery, ScanMelatoninResult};

/// 获取 melatonin 电影列表
pub async fn get_melatonin_movies(
    client: &ApiClient,
    query: &MelatoninListQuery,
) -> ApiResult<ApiResponse<PaginatedResponse<MelatoninMovieList>>> {
    let mut path = String::from("/api/melatonin/list");
    let mut params = Vec::new();

    if let Some(page) = query.page {
        params.push(format!("page={}", page));
    }
    if let Some(page_size) = query.page_size {
        params.push(format!("page_size={}", page_size));
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

/// 根据 ID 获取 melatonin 电影详情
pub async fn get_melatonin_movie_by_id(
    client: &ApiClient,
    id: &Uuid,
) -> ApiResult<ApiResponse<MelatoninMovie>> {
    let path = format!("/api/melatonin/detail/{}", id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

/// 根据演员获取 melatonin 电影列表
pub async fn get_movies_by_actor(
    client: &ApiClient,
    query: &ActorMovieQuery,
) -> ApiResult<ApiResponse<PaginatedResponse<MelatoninMovieList>>> {
    let mut path = format!("/api/melatonin/actor?actor={}", query.actor);
    let mut params = Vec::new();

    if let Some(page) = query.page {
        params.push(format!("page={}", page));
    }
    if let Some(page_size) = query.page_size {
        params.push(format!("page_size={}", page_size));
    }

    if !params.is_empty() {
        path.push('&');
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

/// 获取 bt_list.csv 磁力链接列表
pub async fn get_bt_list(
    client: &ApiClient, id: &Uuid,
) -> ApiResult<ApiResponse<Vec<serde_json::Value>>> {
    let path = format!("/api/melatonin/bt_list/{}", id);
    let response = client.get(&path).await.map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response.json().await.map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

/// 根据类型获取 melatonin 电影列表
pub async fn get_movies_by_genre(
    client: &ApiClient,
    query: &ActorMovieQuery,
) -> ApiResult<ApiResponse<PaginatedResponse<MelatoninMovieList>>> {
    let mut path = format!("/api/melatonin/genre?actor={}", query.actor);
    if let Some(page) = query.page { path.push_str(&format!("&page={}", page)); }
    if let Some(page_size) = query.page_size { path.push_str(&format!("&page_size={}", page_size)); }
    let response = client.get(&path).await.map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response.json().await.map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_get_melatonin_movies_for_dart() {
    let client = ApiClient::tester_client();
    let page = 0;
    let page_size = 10;

    let result = get_melatonin_movies(
        &client,
        &MelatoninListQuery {
            page: Some(page),
            page_size: Some(page_size),
        },
    )
    .await
    .unwrap();

    println!("{}", result);
}
