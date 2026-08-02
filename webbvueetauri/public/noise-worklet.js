// 持续随机噪音 Worklet —— 音频线程实时生成，永不循环重复
// 算法与 Wasm 离线版（synth/noise.rs）完全一致：
// - 白噪：xorshift 均匀随机
// - 粉红：Paul Kellet 7 级滤波（-3dB/oct）
// - 棕噪：随机游走积分（-6dB/oct）
// - 雨声：低通白噪 + 1-3Hz 慢速振幅调制
// 通过 port 消息切换音色/种子。

class NoiseWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._type = 'white';
    this._seed = 42;
    this._reset();
    this.port.onmessage = (e) => {
      const msg = e.data;
      if (msg && msg.type !== undefined) {
        this._type = msg.type;
        this._reset();
      }
      if (msg && msg.seed !== undefined) {
        this._seed = msg.seed >>> 0;
        this._reset();
      }
    };
  }

  _reset() {
    this._x = (this._seed | 1) >>> 0;
    this._b = new Float64Array(7);
    this._last = 0;
    this._lp = 0;
    this._modLevel = 0.75;
    this._modTarget = 0.75;
    this._modCount = 0;
    // 与 wasm 版一致的参数
    this._lpAlpha = Math.min(Math.max(96000 * 0.0001, 0.03), 0.2);
    this._modSpeed = Math.max(Math.round(96000 * 0.0012), 1);
  }

  // xorshift32 → [-1, 1)
  _rng() {
    let x = this._x;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this._x = x >>> 0;
    return ((x >>> 8) & 0xffff) / 32768.0 - 1.0;
  }

  _sample() {
    const white = this._rng();
    switch (this._type) {
      case 'white':
        return white * 0.85;
      case 'pink': {
        const b = this._b;
        b[0] = 0.99886 * b[0] + white * 0.0555179;
        b[1] = 0.99332 * b[1] + white * 0.0750759;
        b[2] = 0.969 * b[2] + white * 0.153852;
        b[3] = 0.8665 * b[3] + white * 0.3104856;
        b[4] = 0.55 * b[4] + white * 0.5329522;
        b[5] = -0.7616 * b[5] - white * 0.016898;
        const out = (b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362) * 0.11;
        b[6] = white * 0.115926;
        return Math.max(-1, Math.min(1, out * 4.5));
      }
      case 'brown': {
        this._last = (this._last + white * 0.02) * 0.997;
        return Math.max(-1, Math.min(1, this._last * 3.5 * 0.85));
      }
      case 'rain': {
        this._lp += this._lpAlpha * (white - this._lp);
        this._modCount++;
        if (this._modCount >= this._modSpeed) {
          this._modCount = 0;
          this._modTarget = 0.5 + Math.abs(this._rng()) * 0.45;
        }
        this._modLevel += (this._modTarget - this._modLevel) * 0.002;
        return Math.max(-1, Math.min(1, this._lp * this._modLevel));
      }
      default:
        return 0;
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0][0];
    if (out) {
      for (let i = 0; i < out.length; i++) {
        out[i] = this._sample();
      }
    }
    return true; // 持续运行
  }
}

registerProcessor('noise-worklet', NoiseWorkletProcessor);
