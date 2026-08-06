//! 环境白噪音合成器 —— 实现已提取至 common::front_can_do::noise（与 Flutter 共享同一份逻辑）
//! AudioWorklet 实时接口见 noise_live.rs

pub use common::front_can_do::noise::{synth_brown_noise, synth_pink_noise, synth_rain_noise, synth_white_noise};
