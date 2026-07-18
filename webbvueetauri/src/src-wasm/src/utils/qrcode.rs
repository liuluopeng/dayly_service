use common::front_can_do::qrcode;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_qr_png_wasm(text: &str, scale: u32, margin: u32) -> Result<Vec<u8>, String> {
    qrcode::generate_qr_png(text, scale, margin).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn generate_qr_unicode_wasm(text: &str) -> Result<String, String> {
    qrcode::generate_qr_unicode(text).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn qr_info_wasm(text: &str) -> Result<js_sys::Array, String> {
    let (version, size) = qrcode::qr_info(text).map_err(|e| e.to_string())?;
    let arr = js_sys::Array::new();
    arr.push(&wasm_bindgen::JsValue::from(version));
    arr.push(&wasm_bindgen::JsValue::from(size as f64));
    Ok(arr)
}
