use common::front_can_do::image_convert;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn detect_image_format(data: &[u8]) -> Option<String> {
    image_convert::detect_format(data).map(|s| s.to_string())
}

#[wasm_bindgen]
pub fn convert_image_wasm(
    input: &[u8],
    output_format: &str,
    quality: u8,
) -> Result<Vec<u8>, String> {
    image_convert::convert_image(input, output_format, quality)
        .map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn convert_image_with_size_wasm(
    input: &[u8],
    output_format: &str,
    quality: u8,
    resize_w: u32,
    resize_h: u32,
) -> Result<Vec<u8>, String> {
    image_convert::convert_image_with_size(input, output_format, quality, resize_w, resize_h)
        .map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn crop_image_wasm(
    input: &[u8],
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, String> {
    image_convert::crop_image(input, x, y, w, h)
        .map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn resize_image_wasm(
    input: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    image_convert::resize_image(input, width, height)
        .map_err(|e| e.to_string())
}
