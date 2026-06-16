use wasm_bindgen::prelude::*;
use common::front_can_do::timestamp;

#[wasm_bindgen]
pub fn timestamp_to_local(timestamp: i64) -> String {
    timestamp::timestamp_to_local(timestamp)
}

#[wasm_bindgen]
pub fn timestamp_to_utc(timestamp: i64) -> String {
    timestamp::timestamp_to_utc(timestamp)
}

#[wasm_bindgen]
pub fn local_to_timestamp(time_str: &str) -> Option<i64> {
    timestamp::local_to_timestamp(time_str)
}

#[wasm_bindgen]
pub fn get_current_timestamp() -> i64 {
    timestamp::get_current_timestamp()
}

#[wasm_bindgen]
pub fn get_current_local_time() -> String {
    timestamp::get_current_local_time()
}

#[wasm_bindgen]
pub fn get_current_utc_time() -> String {
    timestamp::get_current_utc_time()
}
