//! Karplus-Strong 物理建模合成 —— 拨弦/钢琴类音色
//!
//! 原理：用噪声突发激励一段"延迟线"（弦），每周期取相邻采样平均（低通）
//! 并乘以衰减系数，形成自然的指数衰减振荡 —— 比简单正弦波真实得多。
//! 本实现采用多弦模型（主弦 + 失谐八度 + 谐波）+ 槌头瞬态 + 包络，
//! 输出 HiRes（默认 96kHz）PCM 采样，由前端转 AudioBuffer 播放。

use wasm_bindgen::prelude::*;

/// 确定性伪随机数（xorshift 风格，按 seed 可复现）
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x & 0xFFFF) as f32 / 32768.0 - 1.0
    }
}

/// 单根弦：Karplus-Strong 延迟线
fn pluck_string(freq: f32, sample_rate: u32, length: usize, seed: u32) -> Vec<f32> {
    let delay = ((sample_rate as f32 / freq).round() as usize).max(2);
    let mut buffer = vec![0f32; delay];

    // 激励：噪声突发（弦的初始振动）
    let mut rng = Rng(seed | 1);
    for x in buffer.iter_mut() {
        *x = rng.next();
    }

    // 频率相关衰减：低音衰减慢（钢琴低音余音长）
    let decay = (0.9985 - freq / sample_rate as f32 * 1.2).clamp(0.94, 0.9995);

    let mut out = Vec::with_capacity(length);
    let mut idx = 0usize;
    for _ in 0..length {
        let prev = buffer[idx];
        let next = buffer[(idx + 1) % delay];
        let avg = (prev + next) * 0.5;
        buffer[idx] = avg * decay;
        out.push(avg);
        idx = (idx + 1) % delay;
    }
    out
}

/// 生成钢琴音符 PCM 采样（Karplus-Strong 多弦模型）
///
/// # Arguments
/// * `freq`       - 音符基频（Hz），如中央 C4 = 261.63
/// * `duration_ms`- 持续时长（毫秒）
/// * `sample_rate`- 采样率（建议 96000 HiRes，48k 亦可）
/// * `seed`       - 随机种子（同一音符固定 seed 可得到稳定音色）
#[wasm_bindgen]
pub fn synth_piano_note(freq: f32, duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }

    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];

    // 主弦（基频）
    let main = pluck_string(freq, sample_rate, n, seed);
    // 副弦：八度 + 轻微失谐（真实钢琴多弦同击的效果）
    let oct = pluck_string(freq * 2.001, sample_rate, n, seed ^ 0x9E37_79B9);
    // 谐波增强
    let harm = pluck_string(freq * 3.0, sample_rate, n, seed ^ 0xC2B2_AE35);

    // 槌头瞬态：前 5ms 的快速冲击
    let hammer_samples = ((sample_rate as f64 * 0.005) as usize).min(n);

    for i in 0..n {
        let t = i as f64 / sample_rate as f64;
        let mut v = main[i] * 0.9 + oct[i] * 0.35 + harm[i] * 0.12;

        // 槌头瞬态
        if i < hammer_samples {
            let env = (1.0 - i as f64 / hammer_samples as f64) as f32;
            v += env * 0.6 * main[i];
        }

        // 起音（~3ms 防爆音）与尾部释放（~50ms 平滑归零）
        let attack = ((t * 400.0).min(1.0)) as f32;
        let release = (((n - i) as f64 / (sample_rate as f64 * 0.05)).min(1.0).max(0.0)) as f32;
        out[i] = v * attack * release;
    }

    // 归一化到 0.9 峰值，避免削波
    let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0001 {
        let scale = (1.0 / peak).min(1.0) * 0.9;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }

    out
}

/// 生成"明亮"钢琴变体（更强的谐波，可作第二音色）
#[wasm_bindgen]
pub fn synth_piano_bright(freq: f32, duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }

    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];

    let main = pluck_string(freq, sample_rate, n, seed);
    let oct = pluck_string(freq * 2.003, sample_rate, n, seed ^ 0x1234_5678);
    let harm = pluck_string(freq * 3.0, sample_rate, n, seed ^ 0xDEAD_BEEF);
    let bright = pluck_string(freq * 4.02, sample_rate, n, seed ^ 0x0BAD_F00D);

    let hammer_samples = ((sample_rate as f64 * 0.004) as usize).min(n);

    for i in 0..n {
        let t = i as f64 / sample_rate as f64;
        let mut v = main[i] * 0.8 + oct[i] * 0.4 + harm[i] * 0.2 + bright[i] * 0.08;
        if i < hammer_samples {
            let env = (1.0 - i as f64 / hammer_samples as f64) as f32;
            v += env * 0.7 * main[i];
        }
        let attack = ((t * 500.0).min(1.0)) as f32;
        let release = (((n - i) as f64 / (sample_rate as f64 * 0.05)).min(1.0).max(0.0)) as f32;
        out[i] = v * attack * release;
    }

    let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0001 {
        let scale = (1.0 / peak).min(1.0) * 0.9;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }

    out
}
