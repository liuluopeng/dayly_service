// WASM stub — 无缓存，直接读文件
use common::read_metadata::{read_metadata, AudioMetadata};

pub fn read_metadata_cached(file_path: String) -> Result<AudioMetadata, String> {
    read_metadata(&file_path)
}
