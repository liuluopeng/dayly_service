// 环境白噪音合成——复用 common::front_can_do::noise（与 Vue/wasm-demo 同一份实现）

use common::front_can_do::noise::{synth_brown_noise, synth_pink_noise, synth_rain_noise, synth_white_noise};

/// 生成白噪音 PCM（f32，无缝循环）
pub fn white_noise_pcm(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    synth_white_noise(duration_ms, sample_rate, seed)
}

/// 生成粉红噪音 PCM
pub fn pink_noise_pcm(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    synth_pink_noise(duration_ms, sample_rate, seed)
}

/// 生成棕噪音 PCM
pub fn brown_noise_pcm(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    synth_brown_noise(duration_ms, sample_rate, seed)
}

/// 生成雨声 PCM
pub fn rain_noise_pcm(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    synth_rain_noise(duration_ms, sample_rate, seed)
}
