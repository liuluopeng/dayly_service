use flutter_rust_bridge::frb;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::get_client_clone};

pub use common::api::{
    base::ApiError,
    client::ApiClient,
    files::{DirListing, FileInfo, list_files, get_file_info, build_file_url},
};

#[frb(mirror(FileEntry))]
pub struct _FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: Option<String>,
}

pub struct DirListingForDart {
    pub path: String,
    pub entries: Vec<FileEntryForDart>,
    pub total: i64,
}

pub struct FileEntryForDart {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: Option<String>,
}

pub struct FileInfoForDart {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: Option<String>,
    pub content_type: String,
}

pub async fn list_files_for_dart(path: String, page: Option<u32>, page_size: Option<u32>) -> Result<DirListingForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;

    match list_files(&client, &path, page.map(|p| p as usize), page_size.map(|p| p as usize)).await {
        Ok(listing) => {
            let count = listing.entries.len();
            let total = listing.total;
            let entries = listing
                .entries
                .into_iter()
                .map(|e| FileEntryForDart {
                    name: e.name,
                    path: e.path,
                    is_dir: e.is_dir,
                    size: e.size,
                    last_modified: e.last_modified,
                })
                .collect();
            log_to_dart(format!("列出目录 {}: {} 项 (共 {} 项)", path, count, total));
            Ok(DirListingForDart {
                path: listing.path,
                entries,
                total: total as i64,
            })
        }
        Err(err) => Err(err),
    }
}

pub async fn get_file_info_for_dart(path: String) -> Result<FileInfoForDart, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;

    match get_file_info(&client, &path).await {
        Ok(info) => {
            log_to_dart(format!("文件信息: {} ({})", info.name, info.content_type));
            Ok(FileInfoForDart {
                name: info.name,
                path: info.path,
                is_dir: info.is_dir,
                size: info.size,
                last_modified: info.last_modified,
                content_type: info.content_type,
            })
        }
        Err(err) => Err(err),
    }
}

pub fn get_file_url_for_dart(path: String) -> String {
    let client = get_client_clone().unwrap_or_else(|_| ApiClient::default());
    build_file_url(&client, &path)
}
