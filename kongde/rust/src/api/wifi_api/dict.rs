use chrono::Local;
use flutter_rust_bridge::frb;
use uuid::Uuid;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::{get_client_clone, init_client}};

pub use common::{
    api::{
        base::ApiError,
        client::ApiClient,
        dict::{
            collins_resource, get_recent_history, get_top_words, ldoce_resource, search_collins,
            search_ldoce, search_xiandaihanyu, xiandaihanyu_resource,
        },
    },
};

pub use my_type::model::dict::{Word, WordHistory};

pub async fn search_xianzaihanyu_for_dart(word: String) -> Result<String, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match search_xiandaihanyu(&client, &word).await {
        Ok(res) => {
            if let Some(html) = res.data {
                log_to_dart(format!("现代汉语查询成功: {}", word));
                Ok(html)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn search_collins_for_dart(word: String) -> Result<String, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match search_collins(&client, &word).await {
        Ok(res) => {
            if let Some(html) = res.data {
                log_to_dart(format!("Collins 查询成功: {}", word));
                Ok(html)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn collins_resource_for_dart(resource_path: String) -> Result<Vec<u8>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match collins_resource(&client, &resource_path).await {
        Ok(res) => {
            if let Some(data) = res.data {
                Ok(data)
            } else {
                Err(ApiError::Internal(
                    "No data found in response".to_string()
                ))
            }
        }

        Err(err) => Err(err),
    }
}

pub async fn search_ldoce_for_dart(word: String) -> Result<String, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match search_ldoce(&client, &word).await {
        Ok(res) => {
            if let Some(html) = res.data {
                log_to_dart(format!("LDOCE 查询成功: {}", word));
                Ok(html)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn ldoce_resource_for_dart(resource_path: String) -> Result<Vec<u8>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match ldoce_resource(&client, &resource_path).await {
        Ok(res) => {
            if let Some(data) = res.data {
                Ok(data)
            } else {
                Err(ApiError::Internal(
                    "No data found in response".to_string()
                ))
            }
        }

        Err(err) => Err(err),
    }
}

pub async fn xiandaihanyu_resource_for_dart(resource_path: String) -> Result<Vec<u8>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match xiandaihanyu_resource(&client, &resource_path).await {
        Ok(res) => {
            if let Some(data) = res.data {
                Ok(data)
            } else {
                Err(ApiError::Internal(
                    "No data found in response".to_string()
                ))
            }
        }

        Err(err) => Err(err),
    }
}

#[frb(mirror(WordHistory))]
pub struct _WordHistory {
    pub id: Uuid,
    pub word: String,
    pub time: chrono::DateTime<Local>,
    pub created_at: chrono::DateTime<Local>,
}

#[frb(mirror(Word))]
pub struct _Word {
    pub id: Uuid,
    pub word: String,
    pub has_searched_times: i32,
}

pub async fn get_recent_history_for_dart(limit: i64) -> Result<Vec<WordHistory>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_recent_history(&client, limit).await {
        Ok(res) => {
            if let Some(data) = res.data {
                log_to_dart(format!("查询历史成功: {} 条", data.len()));
                Ok(data)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_top_words_for_dart() -> Result<Vec<Word>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_top_words(&client).await {
        Ok(res) => {
            if let Some(data) = res.data {
                log_to_dart(format!("高频词汇成功: {} 个", data.len()));
                Ok(data)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}
