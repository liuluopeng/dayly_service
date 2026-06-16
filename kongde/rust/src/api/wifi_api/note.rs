use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};
use chrono;
use flutter_rust_bridge::frb;
use serde_json;
pub use serde_json::Value;
use uuid::Uuid;

pub use common::api::{
    base::{ApiError, ApiResponse},
    client::ApiClient,
    note::{
        CreateNoteRequest, CreateNoteResponse, SaveNoteRequest, SaveNoteResponse, create_note,
        get_note, list_notes, save_note, search_notes,
    },
};

pub use my_type::dto::NoteSummary;

#[frb(mirror(NoteSummary))]
pub struct _NoteSummary {
    pub id: Uuid,
    pub text: Option<String>,
    pub simple_text: Option<String>,
    pub filepath: Option<String>,
    pub filename: Option<String>,
    pub file_info: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: chrono::DateTime<chrono::Local>,
    pub sha256: Option<String>,
}

#[frb(mirror(CreateNoteResponse))]
pub struct _CreateNoteResponse {
    pub id: String,
    pub message: String,
}

#[frb(mirror(SaveNoteResponse))]
pub struct _SaveNoteResponse {
    pub id: String,
    pub message: String,
}

pub async fn get_note_for_dart(id: String) -> Result<NoteSummary, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match get_note(&client, &id).await {
        Ok(note) => {
            if let Some(note) = note.data {
                Ok(note)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn list_notes_for_dart(page: i32, page_size: i32) -> Result<Vec<NoteSummary>, ApiError> {
    let client = get_client_clone().map_err(|e| {
        log_to_dart(format!("list_notes_for_dart error {:?}", e));
        ApiError::Internal(e.to_string())
    })?;

    log_to_dart(format!("note 启动 page {} page_size {} base_url: {}", page, page_size, client.base_url()));

    match list_notes(&client, Some(page as u32), Some(page_size as u32)).await {
        Ok(notes) => {
            if let Some(notes) = notes.data {
                Ok(notes)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => {
            log_to_dart(format!("list_notes_for_dart error {:?}", err));
            Err(err)
        }
    }
}

pub async fn create_note_for_dart(
    text: String,
    filename: Option<String>,
) -> Result<NoteSummary, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let req = CreateNoteRequest { text, filename };
    match create_note(&client, &req).await {
        Ok(response) => {
            if let Some(note_id) = response.data {
                // 创建成功后获取笔记详情
                match get_note(&client, &note_id.id).await {
                    Ok(note) => {
                        if let Some(note) = note.data {
                            Ok(note)
                        } else {
                            Err(ApiError::Internal("No data found in response".to_string()))
                        }
                    }
                    Err(err) => Err(err),
                }
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn save_note_for_dart(
    id: String,
    text: String,
    filename: Option<String>,
) -> Result<NoteSummary, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let req = SaveNoteRequest {
        id: Some(id.clone()),
        text,
        filename,
    };
    match save_note(&client, &req).await {
        Ok(response) => {
            if let Some(_) = response.data {
                // 保存成功后获取笔记详情
                match get_note(&client, &id).await {
                    Ok(note) => {
                        if let Some(note) = note.data {
                            Ok(note)
                        } else {
                            Err(ApiError::Internal("No data found in response".to_string()))
                        }
                    }
                    Err(err) => Err(err),
                }
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn search_notes_for_dart(query: String) -> Result<Vec<NoteSummary>, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    match search_notes(&client, &query).await {
        Ok(notes) => {
            if let Some(notes) = notes.data {
                Ok(notes)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}
