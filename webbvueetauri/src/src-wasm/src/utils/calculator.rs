use wasm_bindgen::prelude::*;

// 使用公用的console_log宏
use crate::console_log;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    console_log!("add: {:?} {:?}", a, b);
    a + b + 10
}

#[wasm_bindgen]
pub fn add22(a: i32, b: i32) -> i32 {
    console_log!("add: {:?} {:?}", a, b);
    a + b + 22
}

#[wasm_bindgen]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
