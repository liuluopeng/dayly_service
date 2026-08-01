use serde_json::Value;
use uuid::Uuid;

use crate::{
    api::base::{ApiResponse, ApiResult, PaginatedResponse},
    api::client::ApiClient,
};

use my_type::dto;

pub use dto::{AllLyricsResponse, LyricsResponse, SongWithUrl};

/// 扫描歌曲
/// 触发歌曲库扫描
pub async fn scan_songs(client: &ApiClient) -> ApiResult<ApiResponse<Value>> {
    let response = client
        .post("/api/songs/scan", &())
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(data))
}

/// 获取所有歌曲（支持分页）
/// 分页获取歌曲列表（按用户媒体路径过滤）
pub async fn get_all_songs(
    client: &ApiClient,
    page: Option<u32>,
    page_size: Option<u32>,
) -> ApiResult<ApiResponse<PaginatedResponse<SongWithUrl>>> {
    let mut path = "/api/songs/all".to_string();

    // 构建查询参数
    let mut params = Vec::new();
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

/// 根据专辑获取歌曲
/// 按专辑获取歌曲
pub async fn get_songs_by_album(client: &ApiClient, album: &str) -> ApiResult<ApiResponse<Value>> {
    let path = format!("/api/songs/album?album={}", urlencoding::encode(album));
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(data))
}

/// 根据艺术家获取歌曲
/// 按艺术家获取歌曲
pub async fn get_songs_by_artist(
    client: &ApiClient,
    artist: &str,
) -> ApiResult<ApiResponse<Value>> {
    let path = format!("/api/songs/artist?artist={}", urlencoding::encode(artist));
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let data = response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(data))
}

/// 获取歌曲封面
/// 获取歌曲封面图片
pub async fn get_song_cover(client: &ApiClient, song_id: &Uuid) -> ApiResult<ApiResponse<Vec<u8>>> {
    let path = format!("/api/songs/cover/{}", song_id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(bytes.to_vec()))
}

/// 获取歌曲文件
/// 获取歌曲音频文件
pub async fn get_song_file(client: &ApiClient, song_id: &Uuid) -> ApiResult<ApiResponse<Vec<u8>>> {
    let path = format!("/api/songs/{}", song_id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    Ok(ApiResponse::ok(bytes.to_vec()))
}

/// 获取歌词(LRC)
/// 获取歌曲 LRC 歌词
pub async fn get_song_lyrics(
    client: &ApiClient,
    song_id: &Uuid,
) -> ApiResult<ApiResponse<LyricsResponse>> {
    let path = format!("/api/songs/{}/lyrics", song_id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

/// 获取 TTML 逐字歌词（AMLL 格式）
/// 获取歌曲 TTML 逐字歌词
pub async fn get_song_ttml(client: &ApiClient, song_id: &Uuid) -> ApiResult<ApiResponse<String>> {
    let path = format!("/api/songs/{}/ttml", song_id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

/// 获取所有类型的歌词
/// 获取歌曲全部类型的歌词
pub async fn get_all_lyrics(
    client: &ApiClient,
    song_id: &Uuid,
) -> ApiResult<ApiResponse<AllLyricsResponse>> {
    let path = format!("/api/songs/{}/all-lyrics", song_id);
    let response = client
        .get(&path)
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))?;
    response
        .json()
        .await
        .map_err(|e| crate::api::base::ApiError::Internal(format!("{}", e)))
}

#[cfg(test)]
mod tests {

    use super::*;

    // GET请求的测试，默认执行
    #[tokio::test]
    async fn test_get_all_songs() {
        let client = ApiClient::default();

        // 测试默认分页（第1页，每页10条）
        match get_all_songs(&client, None, None).await {
            Ok(response) => {
                println!("获取所有歌曲成功！");
                println!("响应: {}", response);
            }
            Err(error) => {
                println!("获取所有歌曲失败: {}", error);
                panic!("测试失败：获取歌曲失败");
            }
        }
    }
}
