//! 环境白噪音合成器 —— 纯函数生成，无需采样文件
//!
//! 类型：
//! - 白噪音（white）：均匀随机数
//! - 粉红噪音（pink）：Paul Kellet 五级滤波（能量每倍频程 -3dB，更像雨/风/瀑布）
//! - 棕噪音（brown）：随机游走积分（能量 -6dB/倍频程，低沉，像雷声/暖风机）
//! - 雨声（rain）：低通白噪 + 慢速振幅调制（模拟雨落）
//!
//! 所有输出为 HiRes（默认 96kHz）PCM，尾部 100ms 与头部交叉淡化实现无缝循环，
//! 前端用 AudioBufferSourceNode.loop 无限播放。

use super::synth::Rng;
use wasm_bindgen::prelude::*;

/// 无缝循环处理：尾部 100ms 渐变为头部波形，消除循环接缝爆音
fn make_loopable(out: &mut [f32], sample_rate: u32) {
    let cf = ((sample_rate as f64 * 0.1) as usize).min(out.len() / 2);
    let n = out.len();
    for i in 0..cf {
        let w = i as f32 / cf as f32; // 0 → 1
        out[n - cf + i] = out[n - cf + i] * (1.0 - w) + out[i] * w;
    }
}

fn normalize(out: &mut [f32]) {
    let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0001 {
        let scale = (1.0 / peak).min(1.0) * 0.85;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }
}

/// 生成白噪音
#[wasm_bindgen]
pub fn synth_white_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);
    for x in out.iter_mut() {
        *x = rng.next();
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}

/// 生成粉红噪音（Paul Kellet 滤波，-3dB/oct）
#[wasm_bindgen]
pub fn synth_pink_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);
    let mut b = [0f32; 7];
    for (i, x) in out.iter_mut().enumerate() {
        let white = rng.next();
        b[0] = 0.99886 * b[0] + white * 0.0555179;
        b[1] = 0.99332 * b[1] + white * 0.0750759;
        b[2] = 0.96900 * b[2] + white * 0.1538520;
        b[3] = 0.86650 * b[3] + white * 0.3104856;
        b[4] = 0.55000 * b[4] + white * 0.5329522;
        b[5] = -0.7616 * b[5] - white * 0.0168980;
        *x = (b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362) * 0.11;
        b[6] = white * 0.115926;
        let _ = i;
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}

/// 生成棕噪音（随机游走积分，-6dB/oct，低沉）
#[wasm_bindgen]
pub fn synth_brown_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);
    let mut last = 0f32;
    for x in out.iter_mut() {
        last = (last + rng.next() * 0.02) * 0.997;
        *x = last * 3.5;
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}

/// 生成雨声（低通白噪 + 1-3Hz 慢速振幅调制）
#[wasm_bindgen]
pub fn synth_rain_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let sr = sample_rate as f32;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);

    // 低通系数（~3kHz 截止，雨声偏中高频柔和）
    let lp_alpha = (sr * 0.0001).clamp(0.03, 0.2);
    let mut lp = 0f32;

    // 慢速调制：1.2-2.5Hz 的随机游走
    let mut mod_level = 0.75f32;
    let mut mod_target = 0.75f32;
    let mod_speed = (sr * 0.0012).max(1.0) as usize; // ~每 0.8ms 更新一次目标

    for (i, x) in out.iter_mut().enumerate() {
        let white = rng.next();
        lp += lp_alpha * (white - lp);
        // 低通后的白噪再轻微高通（去极低频隆隆声）
        *x = lp;
        if i % mod_speed == 0 {
            mod_target = 0.5 + rng.next().abs() * 0.45;
        }
        mod_level += (mod_target - mod_level) * 0.002;
        *x *= mod_level;
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}

// ─── AudioWorklet 实时接口（extern "C" 指针直写共享内存，零逐采样拷贝） ───
//
// worklet 每 128 采样调用一次 noise_live_fill(len)：
// wasm 直接写入内部静态缓冲区，worklet 用 Float32Array 视图 + set() 拷贝到输出。
// 状态（种子/滤波）全部保留在 wasm 侧，JS 只传指针。

const LIVE_BUF_LEN: usize = 256;

static mut LIVE_KIND: i32 = 0;
static mut LIVE_SR: f32 = 48000.0;
static mut LIVE_X: u32 = 1;
static mut LIVE_B: [f32; 7] = [0.0; 7];
static mut LIVE_LAST: f32 = 0.0;
static mut LIVE_LP: f32 = 0.0;
static mut LIVE_MOD_LEVEL: f32 = 0.75;
static mut LIVE_MOD_TARGET: f32 = 0.75;
static mut LIVE_MOD_COUNT: u32 = 0;
static mut LIVE_BUF: [f32; LIVE_BUF_LEN] = [0.0; LIVE_BUF_LEN];

/// 重置实时噪声状态。kind: 0=白 1=粉红 2=棕 3=雨
#[no_mangle]
pub extern "C" fn noise_live_reset(kind: i32, seed: u32, sample_rate: f32) {
    unsafe {
        LIVE_KIND = kind;
        LIVE_SR = if sample_rate.is_finite() && sample_rate >= 8000.0 { sample_rate } else { 48000.0 };
        LIVE_X = seed | 1;
        LIVE_B = [0.0; 7];
        LIVE_LAST = 0.0;
        LIVE_LP = 0.0;
        LIVE_MOD_LEVEL = 0.75;
        LIVE_MOD_TARGET = 0.75;
        LIVE_MOD_COUNT = 0;
    }
}

/// 返回内部缓冲区指针（worklet 用视图读取）
#[no_mangle]
pub extern "C" fn noise_live_buffer() -> *mut f32 {
    unsafe { LIVE_BUF.as_mut_ptr() }
}

fn live_rng() -> f32 {
    unsafe {
        let mut x = LIVE_X;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        LIVE_X = x;
        ((x >> 8) & 0xFFFF) as f32 / 32768.0 - 1.0
    }
}

/// 生成 len 个采样写入内部缓冲区（len ≤ 256）
#[no_mangle]
pub extern "C" fn noise_live_fill(len: usize) -> i32 {
    let len = len.min(LIVE_BUF_LEN);
    if len == 0 {
        return -1;
    }
    unsafe {
        let sr = LIVE_SR;
        let kind = LIVE_KIND;
        let lp_alpha = (sr * 0.0001).clamp(0.03, 0.2);
        let mod_speed = ((sr * 0.0012).max(1.0)) as u32;
        for out in LIVE_BUF[..len].iter_mut() {
            let white = live_rng();
            let v = match kind {
                0 => white * 0.85,
                1 => {
                    let b = &mut LIVE_B;
                    b[0] = 0.99886 * b[0] + white * 0.0555179;
                    b[1] = 0.99332 * b[1] + white * 0.0750759;
                    b[2] = 0.96900 * b[2] + white * 0.1538520;
                    b[3] = 0.86650 * b[3] + white * 0.3104856;
                    b[4] = 0.55000 * b[4] + white * 0.5329522;
                    b[5] = -0.7616 * b[5] - white * 0.0168980;
                    let v = (b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362) * 0.11;
                    b[6] = white * 0.115926;
                    (v * 4.5).clamp(-1.0, 1.0)
                }
                2 => {
                    LIVE_LAST = (LIVE_LAST + white * 0.02) * 0.997;
                    (LIVE_LAST * 3.5 * 0.85).clamp(-1.0, 1.0)
                }
                _ => {
                    LIVE_LP += lp_alpha * (white - LIVE_LP);
                    LIVE_MOD_COUNT += 1;
                    if LIVE_MOD_COUNT >= mod_speed {
                        LIVE_MOD_COUNT = 0;
                        LIVE_MOD_TARGET = 0.5 + live_rng().abs() * 0.45;
                    }
                    LIVE_MOD_LEVEL += (LIVE_MOD_TARGET - LIVE_MOD_LEVEL) * 0.002;
                    (LIVE_LP * LIVE_MOD_LEVEL).clamp(-1.0, 1.0)
                }
            };
            *out = v;
        }
    }
    0
}
