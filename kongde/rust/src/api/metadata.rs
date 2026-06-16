use common::read_metadata as common_meta;
pub use common::read_metadata::AudioMetadata;

/// FRB 入口：native 走缓存，WASM 直接读
#[cfg(not(target_arch = "wasm32"))]
#[flutter_rust_bridge::frb]
pub fn read_metadata(file_path: String) -> Result<AudioMetadata, String> {
    crate::api::music_cache::read_metadata_cached(file_path)
}

#[cfg(target_arch = "wasm32")]
#[flutter_rust_bridge::frb]
pub fn read_metadata(file_path: String) -> Result<AudioMetadata, String> {
    common_meta::read_metadata(&file_path)
}

/// FRB 批量入口
#[cfg(not(target_arch = "wasm32"))]
#[flutter_rust_bridge::frb]
pub fn read_metadata_batch(file_paths: Vec<String>) -> Vec<Result<AudioMetadata, String>> {
    common_meta::read_metadata_batch(&file_paths)
}

#[cfg(target_arch = "wasm32")]
#[flutter_rust_bridge::frb]
pub fn read_metadata_batch(file_paths: Vec<String>) -> Vec<Result<AudioMetadata, String>> {
    file_paths.iter().map(|p| common_meta::read_metadata(p)).collect()
}
