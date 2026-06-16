use common::api::note::{
    create_note, get_note, list_notes, save_note, search_notes, CreateNoteRequest, SaveNoteRequest,
};
use my_type::dto::NoteSummary;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{api::init::get_api_client, console_log};

#[wasm_bindgen]
pub async fn list_notes_wasm(page: Option<u32>, limit: Option<u32>) -> Result<JsValue, JsValue> {
    console_log!("获取笔记列表: page={:?}, limit={:?}", page, limit);

    let client = get_api_client(None);

    match list_notes(client, page, limit).await {
        Ok(notes) => {
            if let Some(notes) = notes.data {
                console_log!("获取笔记列表成功！找到 {} 条笔记", notes.len());

                // 将数据转换为JavaScript对象
                match to_value(&notes) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                console_log!("获取笔记列表成功，但没有数据");
                match to_value(&Vec::<NoteSummary>::new()) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            }
        }
        Err(error) => {
            console_log!("获取笔记列表失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn get_note_wasm(uuid: &str) -> Result<JsValue, JsValue> {
    console_log!("获取笔记详情: {:?}", uuid);

    let client = get_api_client(None);

    match get_note(client, uuid).await {
        Ok(note) => {
            if let Some(note) = note.data {
                console_log!("获取笔记详情成功！");

                // 将数据转换为JavaScript对象
                match to_value(&note) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                Err(JsValue::from_str("笔记不存在"))
            }
        }
        Err(error) => {
            console_log!("获取笔记详情失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn search_notes_wasm(query: &str) -> Result<JsValue, JsValue> {
    console_log!("搜索笔记: {:?}", query);

    let client = get_api_client(None);

    match search_notes(client, query).await {
        Ok(notes) => {
            if let Some(notes) = notes.data {
                console_log!("搜索成功！找到 {} 条笔记", notes.len());

                // 将数据转换为JavaScript对象
                match to_value(&notes) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                console_log!("搜索成功，但没有数据");
                match to_value(&Vec::<NoteSummary>::new()) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            }
        }
        Err(error) => {
            console_log!("搜索失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn save_note_wasm(
    id: Option<String>,
    text: String,
    filename: Option<String>,
) -> Result<JsValue, JsValue> {
    console_log!(
        "保存笔记: id={:?}, text={:?}, filename={:?}",
        id,
        text,
        filename
    );

    let client = get_api_client(None);
    let req = SaveNoteRequest { id, text, filename };

    match save_note(client, &req).await {
        Ok(response) => {
            if let Some(response) = response.data {
                console_log!("保存笔记成功！id={}", response.id);

                // 将数据转换为JavaScript对象
                match to_value(&response) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                Err(JsValue::from_str("保存失败，无响应数据"))
            }
        }
        Err(error) => {
            console_log!("保存笔记失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}

#[wasm_bindgen]
pub async fn create_note_wasm(text: String, filename: Option<String>) -> Result<JsValue, JsValue> {
    console_log!("创建笔记: text={:?}, filename={:?}", text, filename);

    let client = get_api_client(None);
    let req = CreateNoteRequest { text, filename };

    match create_note(client, &req).await {
        Ok(response) => {
            if let Some(response) = response.data {
                console_log!("创建笔记成功！id={}", response.id);

                // 将数据转换为JavaScript对象
                match to_value(&response) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("序列化失败: {}", e))),
                }
            } else {
                Err(JsValue::from_str("创建失败，无响应数据"))
            }
        }
        Err(error) => {
            console_log!("创建笔记失败: {}", error);
            // 将错误转换为JsValue
            Err(JsValue::from_str(&format!("{}", error)))
        }
    }
}
