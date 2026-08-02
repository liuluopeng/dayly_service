// 持续随机噪音 Worklet —— 音频线程实时生成，永不循环重复
// 首选 Wasm 模式：裸实例化 wasm 模块（extern "C" 导出，指针直写共享内存，
// 每块仅一次 Float32Array.set 拷贝）。wasm 加载失败时回退到 JS 实现。
// 算法与 Wasm 离线版（synth/noise.rs）完全一致。

const KIND = { white: 0, pink: 1, brown: 2, rain: 3 };

class NoiseWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._type = 'white';
    this._seed = 42;
    this._wasm = null;
    this._reset(); // JS 回退状态
    this.port.onmessage = (e) => {
      const msg = e.data;
      if (msg && msg.wasm) {
        this._initWasm(msg.wasm);
      }
      if (msg && msg.type !== undefined) {
        this._type = msg.type;
        this._reset();
        this._applyWasmType();
      }
      if (msg && msg.seed !== undefined) {
        this._seed = msg.seed >>> 0;
        this._reset();
        this._applyWasmType();
      }
    };
  }

  _initWasm(bytes) {
    try {
      const mod = new WebAssembly.Module(bytes);
      // wasm-bindgen 生成的导入：全部用 stub 填充（噪声函数不依赖它们）
      const imports = WebAssembly.Module.imports(mod);
      const stub = {};
      for (const imp of imports) {
        if (!stub[imp.module]) stub[imp.module] = {};
        stub[imp.module][imp.name] = () => {};
      }
      const instance = new WebAssembly.Instance(mod, stub);
      const exp = instance.exports;
      if (typeof exp.noise_live_reset !== 'function') {
        throw new Error('缺少 noise_live 导出');
      }
      this._wasm = {
        reset: exp.noise_live_reset,
        fill: exp.noise_live_fill,
        buffer: exp.noise_live_buffer,
        mem: exp.memory,
      };
      this._applyWasmType();
    } catch (err) {
      this._wasm = null;
      console.error('[noise-worklet] wasm 加载失败，回退 JS:', err);
    }
  }

  _applyWasmType() {
    if (!this._wasm) return;
    this._wasm.reset(KIND[this._type] ?? 0, this._seed, sampleRate);
  }

  _wasmBlock(len) {
    const w = this._wasm;
    w.fill(len);
    // 每次取 memory.buffer（wasm 内存可能增长导致旧 buffer 失效）
    return new Float32Array(w.mem.buffer, w.buffer(), len);
  }

  _reset() {
    this._x = (this._seed | 1) >>> 0;
    this._b = new Float64Array(7);
    this._last = 0;
    this._lp = 0;
    this._modLevel = 0.75;
    this._modTarget = 0.75;
    this._modCount = 0;
    this._lpAlpha = Math.min(Math.max(sampleRate * 0.0001, 0.03), 0.2);
    this._modSpeed = Math.max(Math.round(sampleRate * 0.0012), 1);
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
      if (this._wasm) {
        // Wasm 模式：指针直读 + 一次 set 拷贝（无逐采样跨边界）
        out.set(this._wasmBlock(out.length));
      } else {
        for (let i = 0; i < out.length; i++) {
          out[i] = this._sample();
        }
      }
    }
    return true; // 持续运行
  }
}

registerProcessor('noise-worklet', NoiseWorkletProcessor);
