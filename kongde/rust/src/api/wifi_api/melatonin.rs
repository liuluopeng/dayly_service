use flutter_rust_bridge::frb;
use uuid::Uuid;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};

pub use common::api::{
    base::{ApiError, PaginatedResponse},
    client::ApiClient,
    melatonin::{
        ActorMovieQuery, MelatoninListQuery, get_bt_list, get_movies_by_actor, get_movies_by_genre, get_melatonin_movie_by_id, get_melatonin_movies,
    },
};

pub use my_type::model::melatonin_movie::{MelatoninMovie, MelatoninMovieList};

#[frb(mirror(MelatoninListQuery))]
pub struct _MelatoninListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[frb(mirror(ActorMovieQuery))]
pub struct _ActorMovieQuery {
    pub actor: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub struct MelatoninMovieListForDart {
    pub id: Uuid,
    pub title: String,
    pub cover_path: String,
    pub video_paths: Vec<String>,
    pub cover_url: Option<String>,
    pub video_urls: Vec<String>,
}

pub struct MelatoninMovieDetailForDart {
    pub id: Uuid,
    pub title: String,
    pub cover_path: String,
    pub video_paths: Vec<String>,
    pub cover_url: Option<String>,
    pub video_urls: Vec<String>,
    pub nfo_json: String,
    pub preview_urls: Vec<String>,
}

pub struct PaginatedResponseForDart {
    pub data: Vec<MelatoninMovieListForDart>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: i64,
}

pub async fn get_melatonin_movies_for_dart(
    page: u32,
    page_size: u32,
) -> Result<PaginatedResponseForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let query = MelatoninListQuery {
        page: Some(page),
        page_size: Some(page_size),
    };

    match get_melatonin_movies(&client, &query).await {
        Ok(melatonin_movies) => {
            if let Some(melatonin_movies) = melatonin_movies.data {
                let total = melatonin_movies.total;
                let data = melatonin_movies
                    .data
                    .into_iter()
                    .map(|movie| MelatoninMovieListForDart {
                        id: movie.id,
                        title: movie.title,
                        cover_path: movie.cover_path,
                        video_paths: movie.video_paths,
                        cover_url: movie.cover_url,
                        video_urls: movie.video_urls,
                    })
                    .collect();
                log_to_dart(format!("melatonin 电影列表: page={}, 共 {} 部", page, total));
                Ok(PaginatedResponseForDart {
                    data,
                    total: melatonin_movies.total,
                    page: melatonin_movies.page,
                    page_size: melatonin_movies.page_size,
                    total_pages: melatonin_movies.total_pages,
                })
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_movies_by_actor_for_dart(
    actor: String,
    page: u32,
    page_size: u32,
) -> Result<PaginatedResponseForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let actor_clone = actor.clone();
    let query = ActorMovieQuery {
        actor,
        page: Some(page),
        page_size: Some(page_size),
    };

    match get_movies_by_actor(&client, &query).await {
        Ok(melatonin_movies) => {
            if let Some(melatonin_movies) = melatonin_movies.data {
                let total = melatonin_movies.total;
                let data = melatonin_movies
                    .data
                    .into_iter()
                    .map(|movie| MelatoninMovieListForDart {
                        id: movie.id,
                        title: movie.title,
                        cover_path: movie.cover_path,
                        video_paths: movie.video_paths,
                        cover_url: movie.cover_url,
                        video_urls: movie.video_urls,
                    })
                    .collect();
                log_to_dart(format!("演员 {} 的电影: {} 部", actor_clone, total));
                Ok(PaginatedResponseForDart {
                    data,
                    total: melatonin_movies.total,
                    page: melatonin_movies.page,
                    page_size: melatonin_movies.page_size,
                    total_pages: melatonin_movies.total_pages,
                })
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_movies_by_genre_for_dart(
    genre: String,
    page: u32,
    page_size: u32,
) -> Result<PaginatedResponseForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let genre_clone = genre.clone();
    let query = ActorMovieQuery {
        actor: genre,
        page: Some(page),
        page_size: Some(page_size),
    };
    match get_movies_by_genre(&client, &query).await {
        Ok(melatonin_movies) => {
            if let Some(melatonin_movies) = melatonin_movies.data {
                let total = melatonin_movies.total;
                let data = melatonin_movies.data.into_iter().map(|movie| MelatoninMovieListForDart {
                    id: movie.id, title: movie.title,
                    cover_path: movie.cover_path, video_paths: movie.video_paths,
                    cover_url: movie.cover_url, video_urls: movie.video_urls,
                }).collect();
                log_to_dart(format!("类型 {} 的电影: {} 部", genre_clone, total));
                Ok(PaginatedResponseForDart {
                    data, total: melatonin_movies.total,
                    page: melatonin_movies.page, page_size: melatonin_movies.page_size,
                    total_pages: melatonin_movies.total_pages,
                })
            } else { Err(ApiError::Internal("No data found".to_string())) }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_melatonin_movie_by_id_for_dart(id: Uuid) -> Result<MelatoninMovieDetailForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_melatonin_movie_by_id(&client, &id).await {
        Ok(melatonin_movie) => {
            if let Some(melatonin_movie) = melatonin_movie.data {
                log_to_dart(format!("melatonin 电影详情: {}", melatonin_movie.title));
                let nfo_str = serde_json::to_string(&melatonin_movie.nfo_json).unwrap_or_default();
                Ok(MelatoninMovieDetailForDart {
                    id: melatonin_movie.id,
                    title: melatonin_movie.title,
                    cover_path: melatonin_movie.cover_path,
                    video_paths: melatonin_movie.video_paths,
                    cover_url: melatonin_movie.cover_url,
                    video_urls: melatonin_movie.video_urls,
                    nfo_json: nfo_str,
                    preview_urls: melatonin_movie.preview_urls,
                })
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_bt_list_for_dart(id: Uuid) -> Result<Vec<String>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_bt_list(&client, &id).await {
        Ok(resp) => {
            if let Some(items) = resp.data {
                Ok(items.into_iter().map(|v| v.to_string()).collect())
            } else { Ok(vec![]) }
        }
        Err(err) => Err(err),
    }
}
