#[cfg(not(target_arch = "wasm32"))]
use lofty::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use lofty::read_from_path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u32>,
    pub picture: Option<Vec<u8>>,
}

/// 读取单个音频文件的元数据
#[cfg(not(target_arch = "wasm32"))]
pub fn read_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    let path = Path::new(file_path);
    let tagged_file = read_from_path(path).map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .ok_or_else(|| format!("No tags found: {}", file_path))?;

    let title = tag.title().map(|s| s.to_string());
    let artist = tag.artist().map(|s| s.to_string());
    let album = tag.album().map(|s| s.to_string());
    let duration_ms = tagged_file.properties().duration().as_millis() as u32;
    let picture = tag.pictures().first().map(|pic| pic.data().to_vec());

    Ok(AudioMetadata { title, artist, album, duration_ms: Some(duration_ms), picture })
}

#[cfg(target_arch = "wasm32")]
pub fn read_metadata(_file_path: &str) -> Result<AudioMetadata, String> {
    Err("read_metadata is not supported in wasm".to_string())
}

/// 批量读取元数据（仅 native 用 rayon 并行）
#[cfg(not(target_arch = "wasm32"))]
pub fn read_metadata_batch(file_paths: &[String]) -> Vec<Result<AudioMetadata, String>> {
    use rayon::prelude::*;
    file_paths.par_iter().map(|p| read_metadata(p)).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn read_metadata_batch(file_paths: &[String]) -> Vec<Result<AudioMetadata, String>> {
    file_paths.iter().map(|p| read_metadata(p)).collect()
}
