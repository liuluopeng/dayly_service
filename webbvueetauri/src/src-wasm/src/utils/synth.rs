//! Karplus-Strong 物理建模合成 —— 钢琴音色（润色版）
//!
//! 相比 v1 的改进（消除"皮筋"感）：
//! 1. 分数延迟线（线性插值）：音准精确，无整数延迟的拍频
//! 2. 槌击脉冲激励（脉冲 + 衰减噪声）而非纯噪声：谐波更丰富、更接近琴槌
//! 3. 不谐和度（inharmonicity）：钢琴弦有刚度，泛音频率 = n*f0*sqrt(1+B*n²) 略高于整数倍
//! 4. 双弦失谐（每键两根基频弦，±几音分 detune）：真实钢琴的弦组
//! 5. 分弦衰减：高频泛音衰减快、低频余音长
//! 6. 输出一阶低通：去掉数字采样的高频噪感
//!
//! 输出 HiRes（默认 96kHz）PCM 采样，前端转 AudioBuffer 播放。

use wasm_bindgen::prelude::*;

/// 确定性伪随机数（xorshift 风格）
pub(crate) struct Rng(pub(crate) u32);

impl Rng {
    pub(crate) fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x & 0xFFFF) as f32 / 32768.0 - 1.0
    }
}

/// 单根弦：分数延迟 Karplus-Strong
/// `freq` 弦的振动频率；`decay` 每周期衰减系数（0.99x 附近）
fn pluck_string(freq: f32, sample_rate: u32, length: usize, seed: u32, decay: f32) -> Vec<f32> {
    let d = (sample_rate as f32 / freq).max(2.0);
    let cap = ((d + 2.0) as usize).max(16);
    let mut buf = vec![0f32; cap];
    let mut w = 0.0f32; // 浮点写指针（线性插值读取延迟位置）

    let mut rng = Rng(seed | 1);
    // 槌击激励时长：约 1.5ms
    let excite_len = ((sample_rate as f32 * 0.0015) as usize).max(2);
    let mut out = Vec::with_capacity(length);

    for i in 0..length {
        // 读延迟位置 = 写指针 - d，线性插值
        let rp = w - d;
        let r0 = rp.rem_euclid(cap as f32);
        let i0 = r0.floor() as usize;
        let frac = r0 - r0.floor();
        let i1 = (i0 + 1) % cap;
        let delayed = buf[i0] * (1.0 - frac) + buf[i1] * frac;

        // 激励：槌击脉冲（首个采样强脉冲）+ 快速衰减的噪声（模拟羊毛毡槌头）
        let excite = if i < excite_len {
            let env = 1.0 - i as f32 / excite_len as f32;
            (if i == 0 { 0.9 } else { 0.0 }) + rng.next() * 0.55 * env
        } else {
            0.0
        };

        // Karplus-Strong：相邻平均（低通）* 衰减 + 激励，写回弦缓冲
        let w0 = w.rem_euclid(cap as f32);
        let wi = w0.floor() as usize;
        let prev = buf[wi];
        buf[wi] = (prev + delayed) * 0.5 * decay + excite;
        out.push(delayed);

        w += 1.0;
    }
    out
}

/// 一阶低通（平滑数字味）
pub(crate) struct OnePole {
    y: f32,
}

impl OnePole {
    pub(crate) fn new() -> Self {
        Self { y: 0.0 }
    }

    pub(crate) fn process(&mut self, x: f32, alpha: f32) -> f32 {
        self.y += alpha * (x - self.y);
        self.y
    }
}

/// 钢琴不谐和度系数 B（低音大、高音小）
pub(crate) fn inharmonicity(freq: f32) -> f32 {
    // 中央 C 附近 ~0.0003，每低一个八度约 ×1.6
    (0.0003 * (261.63 / freq).powf(0.8)).clamp(0.00005, 0.004)
}

/// 生成钢琴音符 PCM（润色版 Karplus-Strong 多弦模型）
///
/// # Arguments
/// * `freq`        - 音符基频（Hz），中央 C4 = 261.63
/// * `duration_ms` - 持续时长（毫秒）
/// * `sample_rate` - 采样率（建议 96000 HiRes）
/// * `seed`        - 随机种子（同一音符固定 seed 得到稳定音色）
#[wasm_bindgen]
pub fn synth_piano_note(freq: f32, duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }

    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let sr = sample_rate as f32;
    let mut out = vec![0f32; n];

    // 双弦失谐：真实钢琴同键 2-3 根弦，微差音分（±0.05% ~ ±0.2%）
    let detune = 0.0008 + (seed & 0x1F) as f32 * 0.00004; // 与种子相关，每键略有差异
    let f_a = freq * (1.0 - detune);
    let f_b = freq * (1.0 + detune);

    // 不谐和度：泛音弦频率略高于整数倍
    let b = inharmonicity(freq);
    let f_h2 = 2.0 * freq * (1.0 + b * 4.0).sqrt();
    let f_h3 = 3.0 * freq * (1.0 + b * 9.0).sqrt();

    // 分弦衰减：低频余音长、高频衰减快
    let decay_base = 0.9965 - freq / sr * 0.8;
    let decay_a = decay_base;
    let decay_b = decay_base;
    let decay_h2 = (decay_base - 0.0015).clamp(0.9, 0.999);
    let decay_h3 = (decay_base - 0.003).clamp(0.9, 0.999);

    let main_a = pluck_string(f_a, sample_rate, n, seed, decay_a);
    let main_b = pluck_string(f_b, sample_rate, n, seed ^ 0x7F4A_7C15, decay_b);
    let harm2 = pluck_string(f_h2, sample_rate, n, seed ^ 0x9E37_79B9, decay_h2);
    let harm3 = pluck_string(f_h3, sample_rate, n, seed ^ 0xC2B2_AE35, decay_h3);

    // 槌头瞬态噪声通道：~12ms 指数衰减，提供"击键感"
    let hammer_len = ((sample_rate as f64 * 0.012) as usize).min(n);
    let mut hammer = vec![0f32; n];
    {
        let mut rng = Rng(seed ^ 0x5EED_CAFE);
        for i in 0..hammer_len {
            let env = (1.0 - i as f32 / hammer_len as f32).powf(2.0);
            hammer[i] = rng.next() * 0.35 * env;
        }
    }

    // 混合 + 包络 + 低通
    let mut lp = OnePole::new();
    let lp_alpha = (sr * 0.00006).clamp(0.02, 0.12); // ~2.6kHz 截止（96kHz 时）

    for i in 0..n {
        let t = i as f64 / sample_rate as f64;
        let mut v = main_a[i] * 0.55
            + main_b[i] * 0.55
            + harm2[i] * 0.22
            + harm3[i] * 0.08
            + hammer[i];

        // 起音（~2.5ms）防爆音
        let attack = ((t * 400.0).min(1.0)) as f32;
        // 尾部释放（~80ms 平滑归零）
        let release = (((n - i) as f64 / (sample_rate as f64 * 0.08)).min(1.0).max(0.0)) as f32;

        out[i] = lp.process(v, lp_alpha) * attack * release;
    }

    // 归一化
    let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0001 {
        let scale = (1.0 / peak).min(1.0) * 0.9;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }

    out
}

/// 明亮钢琴变体：更强的谐波与更短的余音（适合快节奏）
#[wasm_bindgen]
pub fn synth_piano_bright(freq: f32, duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }

    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let sr = sample_rate as f32;
    let mut out = vec![0f32; n];

    let detune = 0.0010 + (seed & 0x1F) as f32 * 0.00005;
    let f_a = freq * (1.0 - detune);
    let f_b = freq * (1.0 + detune);

    let b = inharmonicity(freq);
    let f_h2 = 2.0 * freq * (1.0 + b * 4.0).sqrt();
    let f_h3 = 3.0 * freq * (1.0 + b * 9.0).sqrt();
    let f_h4 = 4.0 * freq * (1.0 + b * 16.0).sqrt();

    let decay_base = 0.9950 - freq / sr * 0.8;
    let main_a = pluck_string(f_a, sample_rate, n, seed, decay_base);
    let main_b = pluck_string(f_b, sample_rate, n, seed ^ 0x1234_5678, decay_base);
    let harm2 = pluck_string(f_h2, sample_rate, n, seed ^ 0xDEAD_BEEF, (decay_base - 0.002).clamp(0.9, 0.999));
    let harm3 = pluck_string(f_h3, sample_rate, n, seed ^ 0x0BAD_F00D, (decay_base - 0.004).clamp(0.9, 0.999));
    let harm4 = pluck_string(f_h4, sample_rate, n, seed ^ 0xFACE_B00C, (decay_base - 0.006).clamp(0.9, 0.999));

    let hammer_len = ((sample_rate as f64 * 0.010) as usize).min(n);
    let mut hammer = vec![0f32; n];
    {
        let mut rng = Rng(seed ^ 0x5EED_CAFE);
        for i in 0..hammer_len {
            let env = (1.0 - i as f32 / hammer_len as f32).powf(2.0);
            hammer[i] = rng.next() * 0.45 * env;
        }
    }

    let mut lp = OnePole::new();
    let lp_alpha = (sr * 0.00008).clamp(0.02, 0.15);

    for i in 0..n {
        let t = i as f64 / sample_rate as f64;
        let mut v = main_a[i] * 0.5
            + main_b[i] * 0.5
            + harm2[i] * 0.25
            + harm3[i] * 0.12
            + harm4[i] * 0.06
            + hammer[i];

        let attack = ((t * 450.0).min(1.0)) as f32;
        let release = (((n - i) as f64 / (sample_rate as f64 * 0.06)).min(1.0).max(0.0)) as f32;

        out[i] = lp.process(v, lp_alpha) * attack * release;
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

/// 钟琴/铁琴（Glockenspiel）音色 —— 最容易用函数模拟的乐器
///
/// 原理：金属棒受击产生不谐和泛音（频率比 1:2.76:5.40:8.93...），
/// 每个泛音独立指数衰减，纯闭式表达式：
///   y(t) = Σ Aₙ · sin(2π·f·rₙ·t + φₙ) · e^(-t/τₙ)
///
/// # Arguments
/// * `freq`        - 基频（Hz）
/// * `duration_ms` - 持续时长（毫秒）
/// * `sample_rate` - 采样率（建议 96000）
/// * `seed`        - 随机种子（相位与微失谐）
#[wasm_bindgen]
pub fn synth_bell_note(freq: f32, duration_ms: u32, sample_rate: u32, seed: u32) -> Vec<f32> {
    if !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 || duration_ms == 0 {
        return Vec::new();
    }

    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let sr = sample_rate as f32;
    let mut out = vec![0f32; n];

    // 金属棒泛音比（glockenspiel 理论值）+ 振幅 + 相对衰减时长
    const PARTIALS: [(f32, f32, f32); 6] = [
        (1.0, 1.0, 1.0),      // 基频：余音最长
        (2.76, 0.55, 0.28),   // 第一泛音
        (5.40, 0.30, 0.12),   // 第二泛音
        (8.93, 0.18, 0.06),   // 第三泛音
        (13.34, 0.10, 0.035), // 第四泛音
        (18.76, 0.06, 0.02),  // 第五泛音
    ];

    // 每泛音固定相位 + 轻微失谐（±0.1%，金属棒微差，形成自然拍频）
    let mut rng = Rng(seed | 1);
    let mut phases = [0f32; 6];
    let mut detunes = [0f32; 6];
    for p in phases.iter_mut() {
        *p = rng.next() * std::f32::consts::PI;
    }
    for d in detunes.iter_mut() {
        *d = 0.0008 + rng.next().abs() * 0.0012;
    }

    let tau_base = 1.6f32; // 基频衰减时间常数（秒）
    let t_end = duration_ms as f32 / 1000.0;
    let mut amplitudes = [0f32; 6];
    let mut freqs = [0f32; 6];
    let mut taus = [0f32; 6];
    for i in 0..6 {
        let (r, a, rel) = PARTIALS[i];
        freqs[i] = freq * r * (1.0 + detunes[i]);
        amplitudes[i] = a;
        taus[i] = (tau_base * rel).max(0.01);
    }

    // 打击瞬态：~6ms 金属接触噪声
    let strike_len = ((sr * 0.006) as usize).min(n);

    for i in 0..n {
        let t = i as f32 / sr;
        let mut v = 0.0f32;
        for k in 0..6 {
            if t > t_end {
                break;
            }
            let env = (-t / taus[k]).exp();
            v += amplitudes[k] * (std::f32::consts::TAU * freqs[k] * t + phases[k]).sin() * env;
        }
        if i < strike_len {
            let env = 1.0 - i as f32 / strike_len as f32;
            v += rng.next() * 0.25 * env * env;
        }
        // 起音（1.5ms 防爆音）
        let attack = ((t * 650.0).min(1.0)) as f32;
        out[i] = v * attack;
    }

    // 归一化
    let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0001 {
        let scale = (1.0 / peak).min(1.0) * 0.9;
        for x in out.iter_mut() {
            *x *= scale;
        }
    }

    out
}
