//! AudioWorklet 实时噪声接口（extern "C" 指针直写共享内存，零逐采样拷贝）
//!
//! worklet 每 128 采样调用一次 noise_live_fill(len)：
//! wasm 直接写入内部静态缓冲区，worklet 用 Float32Array 视图 + set() 拷贝到输出。
//! 状态（种子/滤波）全部保留在 wasm 侧，JS 只传指针。

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
        LIVE_SR = if sample_rate.is_finite() && sample_rate >= 8000.0 {
            sample_rate
        } else {
            48000.0
        };
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
    std::ptr::addr_of_mut!(LIVE_BUF) as *mut f32
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
                    let b: &mut [f32; 7] = &mut *std::ptr::addr_of_mut!(LIVE_B);
                    b[0] = 0.99886f32 * b[0] + white * 0.0555179;
                    b[1] = 0.99332f32 * b[1] + white * 0.0750759;
                    b[2] = 0.969f32 * b[2] + white * 0.153852f32;
                    b[3] = 0.86650f32 * b[3] + white * 0.3104856;
                    b[4] = 0.55000f32 * b[4] + white * 0.5329522;
                    b[5] = -0.7616f32 * b[5] - white * 0.0168980;
                    let v =
                        (b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362) * 0.11;
                    b[6] = white * 0.115926;
                    (v * 4.5).clamp(-1.0, 1.0)
                }
                2 => {
                    LIVE_LAST = (LIVE_LAST + white * 0.02) * 0.997;
                    (LIVE_LAST * 3.5f32 * 0.85).clamp(-1.0, 1.0)
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
