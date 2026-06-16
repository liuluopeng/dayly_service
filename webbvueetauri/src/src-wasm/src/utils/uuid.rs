use wasm_bindgen::prelude::*;
use common::front_can_do::uuid;

#[wasm_bindgen]
pub fn generate_uuid_v4() -> String {
    uuid::generate_uuid_v4()
}

#[wasm_bindgen]
pub fn generate_uuid_v5(namespace: &str, name: &str) -> String {
    uuid::generate_uuid_v5(namespace, name)
}

#[wasm_bindgen]
pub fn generate_uuid_v6() -> String {
    uuid::generate_uuid_v6()
}

#[wasm_bindgen]
pub fn generate_uuid_v7() -> String {
    uuid::generate_uuid_v7()
}

#[wasm_bindgen]
pub fn validate_uuid(uuid_str: &str) -> bool {
    uuid::validate_uuid(uuid_str)
}
