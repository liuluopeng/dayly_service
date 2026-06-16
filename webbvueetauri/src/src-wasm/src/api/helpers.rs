use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::console_log;

/// Convert serde_json::Value to JsValue using json_compatible serializer.
/// This produces plain JS objects instead of Map instances.
pub fn to_js(value: &Value) -> Result<JsValue, JsValue> {
    use serde_wasm_bindgen::Serializer;
    value
        .serialize(&Serializer::json_compatible())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Extract "data" field from ApiResponse { msg, data } and convert to JsValue.
pub fn data_to_js(result: Value) -> Result<JsValue, JsValue> {
    let data = match result {
        Value::Object(map) => map.get("data").cloned().unwrap_or(Value::Null),
        other => other,
    };
    to_js(&data)
}

/// Extract "data" field, defaulting to empty array if missing.
pub fn data_array_to_js(result: Value) -> Result<JsValue, JsValue> {
    let data = match result {
        Value::Object(map) => map.get("data").cloned().unwrap_or(Value::Array(vec![])),
        other => other,
    };
    to_js(&data)
}

/// Wrap an API call with automatic logging.
/// Logs the function name on entry, and success/failure on exit.
///
/// Usage:
///   api_call!("get_all_songs", get_all_songs(&client, ...).await)
///   api_call!("get_all_songs", data_to_js, get_all_songs(&client, ...).await)
#[macro_export]
macro_rules! api_call {
    ($name:expr, $result:expr) => {{
        console_log!("[API] {}", $name);
        match $result {
            Ok(val) => {
                console_log!("[API] {} => ok", $name);
                Ok(val)
            }
            Err(e) => {
                let msg = format!("[API] {} => error: {}", $name, e);
                console_log!("{}", msg);
                Err(JsValue::from_str(&msg))
            }
        }
    }};
    ($name:expr, $serializer:expr, $result:expr) => {{
        console_log!("[API] {}", $name);
        match $result {
            Ok(val) => {
                let js = $serializer(val);
                match &js {
                    Ok(_) => console_log!("[API] {} => ok", $name),
                    Err(e) => console_log!("[API] {} => serialize error: {:?}", $name, e),
                }
                js
            }
            Err(e) => {
                let msg = format!("[API] {} => error: {}", $name, e);
                console_log!("{}", msg);
                Err(JsValue::from_str(&msg))
            }
        }
    }};
}
