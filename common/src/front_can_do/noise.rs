//! 环境白噪音合成器 —— 纯函数生成，无需采样文件（与 wasm-demo/Vue 共享的实现）
//!
//! 类型：
//! - 白噪音（white）：均匀随机数
//! - 粉红噪音（pink）：Paul Kellet 五级滤波（能量每倍频程 -3dB，更像雨/风/瀑布）
//! - 棕噪音（brown）：随机游走积分（能量 -6dB/倍频程，低沉，像雷声/暖风机）
//! - 雨声（rain）：低通白噪 + 慢速振幅调制（模拟雨落）
//!
//! 所有输出为 HiRes（默认 96kHz）PCM，尾部 100ms 与头部交叉淡化实现无缝循环。

/// xorshift32 伪随机数生成器（确定性，可复现）
#[derive(Clone, Copy)]
pub struct Rng(pub u32);

impl Rng {
    pub fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x & 0xFFFF) as f32 / 32768.0 - 1.0
    }
}

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
    if peak > 0.0001f32 {
        let scale = (1.0 / peak).min(1.0) * 0.85;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }
}

/// 生成白噪音
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
pub fn synth_pink_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);
    let mut b = [0f32; 7];
    for x in out.iter_mut() {
        let white = rng.next();
        b[0] = 0.99886f32 * b[0] + white * 0.0555179;
        b[1] = 0.99332f32 * b[1] + white * 0.0750759;
        b[2] = 0.969f32 * b[2] + white * 0.153852f32;
        b[3] = 0.86650f32 * b[3] + white * 0.3104856;
        b[4] = 0.55000f32 * b[4] + white * 0.5329522;
        b[5] = -0.7616f32 * b[5] - white * 0.0168980;
        *x = (b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362) * 0.11;
        b[6] = white * 0.115926;
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}

/// 生成棕噪音（随机游走积分，-6dB/oct，低沉）
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
pub fn synth_rain_noise(duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let mut out = vec![0f32; n];
    let mut rng = Rng(seed | 1);
    let mut lp = 0f32;
    let mut phase = 0.0f32;
    for x in out.iter_mut() {
        // 慢速振幅调制（1-3Hz 随机）
        phase += 0.0002 + rng.next().abs() * 0.0002;
        let mod_amp = 0.35 + 0.65 * (phase.sin() * 0.5 + 0.5);
        // 低通：一阶 IIR
        lp = lp * 0.85 + rng.next() * 0.15;
        *x = lp * mod_amp;
    }
    make_loopable(&mut out, sample_rate);
    normalize(&mut out);
    out
}
