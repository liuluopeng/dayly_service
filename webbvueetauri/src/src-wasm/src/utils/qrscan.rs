use common::front_can_do::qrscan;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn scan_qr_from_image_wasm(data: &[u8]) -> Result<String, String> {
    qrscan::scan_qr_from_image(data).map_err(|e| e.to_string())
}
