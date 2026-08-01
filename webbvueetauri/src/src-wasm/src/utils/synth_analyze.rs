//! 音色分析-再合成：从钢琴采样 MP3 中提取音色参数，再用参数驱动加法合成
// 热路径 DSP 循环：延迟线/相位计算依赖索引形式，保持索引循环
#![allow(clippy::needless_range_loop)]
//!
//! 管线：
//! 1. `analyze_piano_samples`：STFT + Hann 窗，逐帧提取前 H 个谐波的峰值振幅、
//!    整体 RMS 包络、以及锤击瞬态（非谐波区）噪声频谱
//! 2. `synth_analyzed_note`：加法合成——每个谐波一个正弦（频率含不谐和度
//!    n·f0·sqrt(1+B·n²)），振幅按提取包络插值，加上噪声瞬态
//!
//! 参数布局（Vec<f32>）：
//!   [0] freq  [1] sample_rate  [2] B  [3] H  [4] M
//!   [5 .. 5+H*M]          各谐波振幅包络（M 个对数时间点，线性振幅）
//!   [5+H*M .. 5+H*M+M]    整体 RMS 包络（归一化 0..1）
//!   [5+H*M+M .. +NOISE]   噪声频谱能量（NOISE 段）

use super::synth::{inharmonicity, OnePole, Rng};
use wasm_bindgen::prelude::*;

const H: usize = 40; // 最多谐波数
const M: usize = 24; // 包络点数
const NOISE_BINS: usize = 32; // 噪声频谱分段
const FRAME: usize = 4096; // FFT 帧长
const HOP: usize = 1024; // 帧移
const PARAM_HEAD: usize = 5;

/// 迭代 radix-2 FFT（原位，re/im 分离）
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());

    // bit-reversal 置换
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // 蝶形运算
    let mut len = 2usize;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        let half = len >> 1;
        let mut i = 0usize;
        while i < n {
            let (mut w_r, mut w_i) = (1.0f32, 0.0f32);
            for k in 0..half {
                let er = re[i + k];
                let ei = im[i + k];
                let or = re[i + k + half];
                let oi = im[i + k + half];
                re[i + k] = er + w_r * or - w_i * oi;
                im[i + k] = ei + w_r * oi + w_i * or;
                re[i + k + half] = er - w_r * or + w_i * oi;
                im[i + k + half] = ei - w_r * oi - w_i * or;
                let nw_r = w_r * wr - w_i * wi;
                w_i = w_r * wi + w_i * wr;
                w_r = nw_r;
            }
            i += len;
        }
        len <<= 1;
    }
}

/// 对数时间包络位置（偏向开头，模拟听感的时间刻度）
fn env_pos(j: usize, m: usize, frames: usize) -> (usize, usize, f32) {
    let t_pos = (j as f32 / (m as f32 - 1.0)).powf(1.5) * (frames as f32 - 1.0);
    let idx = t_pos.floor() as usize;
    let idx2 = (idx + 1).min(frames - 1);
    let frac = t_pos - idx as f32;
    (idx, idx2, frac)
}

/// 从一段钢琴采样中提取音色参数
///
/// # Arguments
/// * `samples`     - 单声道 PCM 采样（f32，-1..1）
/// * `freq`        - 该采样的基频（Hz）
/// * `sample_rate` - 采样的采样率
#[wasm_bindgen]
pub fn analyze_piano_samples(samples: &[f32], freq: f32, sample_rate: u32) -> Vec<f32> {
    let n = samples.len();
    if n < FRAME || !freq.is_finite() || freq <= 0.0 || sample_rate < 8000 {
        return Vec::new();
    }

    let frames = (n - FRAME) / HOP + 1;
    let sr = sample_rate as f32;
    let b = inharmonicity(freq);

    // Hann 窗
    let mut win = vec![0f32; FRAME];
    for i in 0..FRAME {
        win[i] = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FRAME - 1) as f32).cos());
    }

    let mut amps = vec![vec![0f32; frames]; H];
    let mut rms = vec![0f32; frames];
    let mut noise_energy = [0f32; NOISE_BINS];

    let mut re = vec![0f32; FRAME];
    let mut im = vec![0f32; FRAME];
    let mut mag = vec![0f32; FRAME];
    let mut masked = vec![0f32; FRAME];

    for f in 0..frames {
        let start = f * HOP;
        for i in 0..FRAME {
            re[i] = samples[start + i] * win[i];
            im[i] = 0.0;
        }
        fft(&mut re, &mut im);
        for i in 0..FRAME {
            mag[i] = (re[i] * re[i] + im[i] * im[i]).sqrt();
        }

        // 整体 RMS
        let mut s = 0.0f32;
        for i in 0..FRAME {
            let v = samples[start + i];
            s += v * v;
        }
        rms[f] = (s / FRAME as f32).sqrt();

        // 逐谐波峰值提取：抛物线插值精确定位（对数幅度域），
        // 避免谐波落在 bin 之间时振幅被低估
        for k in 0..H {
            let nk = (k + 1) as f32;
            let fk = nk * freq * (1.0 + b * nk * nk).sqrt();
            let bin = (fk / sr * FRAME as f32) as usize;
            let lo = bin.saturating_sub(3);
            let hi = (bin + 4).min(FRAME - 1);
            let mut best = 0.0f32;
            let mut best_bin = lo;
            for i in lo..hi {
                if mag[i] > best {
                    best = mag[i];
                    best_bin = i;
                }
            }
            if best_bin > 0 && best_bin + 1 < FRAME && mag[best_bin] > 1e-6 {
                let y0 = mag[best_bin - 1].max(1e-12).ln();
                let y1 = mag[best_bin].max(1e-12).ln();
                let y2 = mag[best_bin + 1].max(1e-12).ln();
                let denom = y0 - 2.0 * y1 + y2;
                let p = if denom.abs() > 1e-9 {
                    0.5 * (y0 - y2) / denom
                } else {
                    0.0
                };
                let amp = y1 - 0.25 * (y0 - y2) * p;
                amps[k][f] = amp.exp() * 4.0 / FRAME as f32;
            } else {
                amps[k][f] = best * 4.0 / FRAME as f32;
            }
        }

        // 锤击瞬态噪声频谱（前 3 帧，屏蔽谐波邻域后的残差能量）
        if f < 3 {
            masked.copy_from_slice(&mag);
            for k in 0..H {
                let nk = (k + 1) as f32;
                let fk = nk * freq * (1.0 + b * nk * nk).sqrt();
                let bin = (fk / sr * FRAME as f32) as usize;
                let lo = bin.saturating_sub(4);
                let hi = (bin + 5).min(FRAME - 1);
                for i in lo..hi {
                    masked[i] = 0.0;
                }
            }
            for seg in 0..NOISE_BINS {
                let seg_start = seg * FRAME / NOISE_BINS;
                let seg_end = (seg + 1) * FRAME / NOISE_BINS;
                let mut e = 0.0f32;
                for i in seg_start..seg_end {
                    e += masked[i] * masked[i];
                }
                noise_energy[seg] += e / 3.0;
            }
        }
    }

    // 打包参数
    let total = PARAM_HEAD + H * M + M + NOISE_BINS;
    let mut out = vec![0f32; total];
    out[0] = freq;
    out[1] = sample_rate as f32;
    out[2] = b;
    out[3] = H as f32;
    out[4] = M as f32;

    let base = PARAM_HEAD;
    for k in 0..H {
        for j in 0..M {
            let (i0, i1, frac) = env_pos(j, M, frames);
            out[base + k * M + j] = amps[k][i0] + (amps[k][i1] - amps[k][i0]) * frac;
        }
    }

    let env_base = base + H * M;
    let mut env_max = 0.0f32;
    for j in 0..M {
        let (i0, i1, frac) = env_pos(j, M, frames);
        out[env_base + j] = rms[i0] + (rms[i1] - rms[i0]) * frac;
        if out[env_base + j] > env_max {
            env_max = out[env_base + j];
        }
    }
    if env_max > 0.0001 {
        for j in 0..M {
            out[env_base + j] /= env_max;
        }
    }

    let noise_base = env_base + M;
    let mut n_max = 0.0f32;
    for seg in 0..NOISE_BINS {
        if noise_energy[seg] > n_max {
            n_max = noise_energy[seg];
        }
    }
    if n_max > 0.0 {
        for seg in 0..NOISE_BINS {
            out[noise_base + seg] = (noise_energy[seg] / n_max).sqrt();
        }
    }

    out
}

/// 用提取的音色参数做加法再合成
///
/// # Arguments
/// * `params`      - `analyze_piano_samples` 的输出
/// * `freq`        - 目标音符基频（可用与分析不同的键，实现音色迁移）
/// * `duration_ms` - 持续时长
/// * `sample_rate` - 输出采样率（建议 96000）
/// * `seed`        - 随机种子（相位）
#[wasm_bindgen]
pub fn synth_analyzed_note(
    params: &[f32],
    freq: f32,
    duration_ms: u32,
    sample_rate: u32,
    seed: u32,
) -> Vec<f32> {
    if params.len() < PARAM_HEAD + 1
        || !freq.is_finite()
        || freq <= 0.0
        || sample_rate < 8000
        || duration_ms == 0
    {
        return Vec::new();
    }

    let h = (params[3] as usize).clamp(1, H);
    let m = (params[4] as usize).clamp(2, M);
    if params.len() < PARAM_HEAD + h * m + m + 1 {
        return Vec::new();
    }

    let b = params[2];
    let n = ((duration_ms as f64 * sample_rate as f64) / 1000.0) as usize;
    let sr = sample_rate as f32;
    let mut out = vec![0f32; n];

    // 动态谐波数：高频键超过 Nyquist 的谐波无意义
    let max_h = (((sr * 0.45) / freq) as usize).min(h);
    if max_h == 0 {
        return out;
    }

    // 每键固定的随机相位
    let mut rng = Rng(seed | 1);
    let mut phases = vec![0f32; max_h];
    for p in phases.iter_mut() {
        *p = rng.next() * std::f32::consts::PI;
    }

    // 时间对数刻度（2ms 起）
    let t0 = 0.002f32;
    let t_end = duration_ms as f32 / 1000.0;
    let inv_range = 1.0 / (t_end.max(t0).ln() - t0.ln());

    let base = PARAM_HEAD;
    let env_base = base + h * m;

    for i in 0..n {
        let t = i as f32 / sr;
        let pos =
            ((t.max(t0).ln() - t0.ln()) * inv_range * (m as f32 - 1.0)).clamp(0.0, m as f32 - 1.0);
        let idx = pos.floor() as usize;
        let frac = pos - idx as f32;
        let idx2 = (idx + 1).min(m - 1);

        // 整体包络（从采样学到的衰减形状）
        let e0 = params[env_base + idx];
        let e1 = params[env_base + idx2];
        let env = e0 + (e1 - e0) * frac;

        let mut v = 0.0f32;
        for k in 0..max_h {
            let nk = (k + 1) as f32;
            let fk = nk * freq * (1.0 + b * nk * nk).sqrt();
            let a0 = params[base + k * m + idx];
            let a1 = params[base + k * m + idx2];
            let amp = a0 + (a1 - a0) * frac;
            if amp <= 0.0001 {
                continue;
            }
            v += amp * env * (std::f32::consts::TAU * fk * t + phases[k]).sin();
        }

        // 锤击瞬态噪声：白噪声按提取的噪声频谱整形（简化：噪声能量缩放）
        if i < ((sr * 0.010) as usize).min(n) {
            let env_t = 1.0 - i as f32 / (sr * 0.010).min(n as f32);
            v += rng.next() * 0.12 * env_t;
        }

        out[i] = v;
    }

    // 轻微低通平滑 + 起音/释放 + 归一化
    let mut lp = OnePole::new();
    let lp_alpha = (sr * 0.00006).clamp(0.02, 0.12);
    let peak_guard = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
    let pre_scale = if peak_guard > 0.0001 {
        (1.0 / peak_guard).min(1.0)
    } else {
        1.0
    };

    for i in 0..n {
        let t = i as f32 / sr;
        let attack = (t * 400.0).min(1.0);
        let release = ((n - i) as f32 / (sr * 0.08)).clamp(0.0, 1.0);
        out[i] = lp.process(out[i] * pre_scale, lp_alpha) * attack * release;
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
