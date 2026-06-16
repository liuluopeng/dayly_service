use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent};

use crate::api::init::get_api_client;

/// Connect to the chat WebSocket and call `on_message(text)` for each message.
/// Returns the WebSocket handle; call `close()` on it to disconnect.
#[wasm_bindgen]
pub fn connect_chat_ws(path: &str, on_message: js_sys::Function) -> Result<WebSocket, JsValue> {
    let client = get_api_client(None);
    let base_url = client.base_url().to_string();
    let token = client.token().unwrap_or_default().to_string();

    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let full_url = format!("{}{}?token={}", ws_url, path, urlencoding::encode(&token));

    let ws = WebSocket::new(&full_url)?;

    let on_msg_cb = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        if let Some(text) = event.data().as_string() {
            let _ = on_message.call1(&JsValue::NULL, &JsValue::from_str(&text));
        }
    });
    ws.set_onmessage(Some(on_msg_cb.as_ref().unchecked_ref()));
    on_msg_cb.forget();

    let on_err_cb = Closure::<dyn FnMut(ErrorEvent)>::new(move |event: ErrorEvent| {
        web_sys::console::error_1(&format!("WS error: {:?}", event).into());
    });
    ws.set_onerror(Some(on_err_cb.as_ref().unchecked_ref()));
    on_err_cb.forget();

    let on_close_cb = Closure::<dyn FnMut(CloseEvent)>::new(move |event: CloseEvent| {
        web_sys::console::log_1(&format!("WS closed: {}", event.code()).into());
    });
    ws.set_onclose(Some(on_close_cb.as_ref().unchecked_ref()));
    on_close_cb.forget();

    Ok(ws)
}
