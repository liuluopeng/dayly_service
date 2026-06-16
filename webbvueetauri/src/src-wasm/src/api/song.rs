use common::api::{
    client::ApiClient,
    songs::{
        get_all_lyrics, get_all_songs, get_song_cover, get_song_file, get_song_lyrics,
        get_song_ttml, get_songs_by_album, get_songs_by_artist, scan_songs,
    },
};
use serde_wasm_bindgen::to_value;
use std::cell::RefCell;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys;

#[cfg(test)]
use wasm_bindgen_test::*;

use crate::{api::init::get_api_client, console_log};

#[cfg(test)]
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
pub async fn scan_songs_wasm() -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match scan_songs(client).await {
        Ok(response) => {
            console_log!("扫描歌曲成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("扫描歌曲失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_all_songs_wasm(
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_all_songs(client, page, page_size).await {
        Ok(response) => {
            console_log!("获取所有歌曲成功！");
            console_log!("响应消息: {}", response);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取所有歌曲失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_songs_by_album_wasm(album: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_songs_by_album(client, album).await {
        Ok(response) => {
            console_log!("根据专辑获取歌曲成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("根据专辑获取歌曲失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_songs_by_artist_wasm(artist: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    match get_songs_by_artist(client, artist).await {
        Ok(response) => {
            console_log!("根据艺术家获取歌曲成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("根据艺术家获取歌曲失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_song_cover_wasm(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_song_cover(client, &uuid).await {
        Ok(response) => {
            console_log!("获取歌曲封面成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取歌曲封面失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_song_file_wasm(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_song_file(client, &uuid).await {
        Ok(response) => {
            console_log!("获取歌曲文件成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取歌曲文件失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_song_lyrics_wasm(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_song_lyrics(client, &uuid).await {
        Ok(response) => {
            console_log!("获取歌词成功！");
            console_log!("响应消息: {}", response.msg);
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取歌词失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_song_ttml_wasm(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_song_ttml(client, &uuid).await {
        Ok(response) => {
            console_log!("获取 TTML 歌词成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取 TTML 歌词失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_all_lyrics_wasm(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_all_lyrics(client, &uuid).await {
        Ok(response) => {
            console_log!("获取所有歌词成功！");
            match to_value(&response.data) {
                Ok(js_value) => Ok(js_value),
                Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
            }
        }
        Err(error) => {
            console_log!("获取所有歌词失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

// WASM 内存中的音频缓存，零拷贝用
thread_local! {
    static AUDIO_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn get_wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// 零拷贝获取歌曲文件：返回 [ptr, len]，JS 直接从 WASM 内存创建 Uint8Array 视图
#[wasm_bindgen]
pub async fn load_song_audio_zc(song_id: &str) -> Result<JsValue, JsValue> {
    let client = get_api_client(None);

    let uuid = match Uuid::parse_str(song_id) {
        Ok(uuid) => uuid,
        Err(e) => return Err(JsValue::from_str(&format!("无效的UUID: {}", e))),
    };

    match get_song_file(client, &uuid).await {
        Ok(response) => {
            console_log!("零拷贝获取音频成功！");
            if let Some(data) = response.data {
                let len = data.len();
                let ptr = data.as_ptr() as usize;
                AUDIO_BUF.with(|buf| {
                    *buf.borrow_mut() = data;
                });

                to_value(&(ptr as u32, len as u32)).map_err(|e| JsValue::from_str(&format!("{}", e)))
            } else {
                Err(JsValue::from_str("没有音频数据"))
            }
        }
        Err(error) => {
            console_log!("零拷贝获取音频失败: {}", error);
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[cfg(test)]
#[wasm_bindgen_test]
async fn test_get_all_songs_wasm() {
    // Test the get_all_songs_wasm function with page 1 and page_size 10
    let result = get_all_songs_wasm(Some(1), Some(10)).await;

    // Check if the result is Ok
    match result {
        Ok(data) => {
            // Log the result to the console
            web_sys::console::log_1(&format!("Test passed! Result: {:?}", data).into());
        }
        Err(err) => {
            // Log the error to the console
            web_sys::console::error_1(&format!("Test failed! Error: {:?}", err).into());
            panic!("Test failed with error: {:?}", err);
        }
    }
}
