use common::front_can_do::password;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_password(length: usize) -> String {
    password::generate_password(length)
}

#[wasm_bindgen]
pub fn generate_strong_password(length: usize) -> String {
    password::generate_strong_password(length)
}
