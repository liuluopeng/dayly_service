//! 环境白噪音合成器 —— 实现已提取至 common::front_can_do::noise（与 Flutter 共享同一份逻辑）
//! 此处仅做 wasm-bindgen 包装（wasm-bindgen 不导出 common 中的裸函数）
//! AudioWorklet 实时接口见 noise_live.rs

use wasm_bindgen::prelude::*;

use common::front_can_do::noise as noise_impl;

/// 生成白噪音 PCM（f32，无缝循环）
#[wasm_bindgen]
pub fn synth_white_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    noise_impl::synth_white_noise(duration_ms, sample_rate, seed)
}

/// 生成粉红噪音 PCM
#[wasm_bindgen]
pub fn synth_pink_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    noise_impl::synth_pink_noise(duration_ms, sample_rate, seed)
}

/// 生成棕噪音 PCM
#[wasm_bindgen]
pub fn synth_brown_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    noise_impl::synth_brown_noise(duration_ms, sample_rate, seed)
}

/// 生成雨声 PCM
#[wasm_bindgen]
pub fn synth_rain_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    noise_impl::synth_rain_noise(duration_ms, sample_rate, seed)
}
