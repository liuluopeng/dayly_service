<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  synth_white_noise,
  synth_pink_noise,
  synth_brown_noise,
  synth_rain_noise,
} from '../types/wasm-typed';

const { t } = useI18n();

type NoiseType = 'white' | 'pink' | 'brown' | 'rain';

interface NoiseOption {
  type: NoiseType;
  icon: string;
}

const NOISE_OPTIONS: NoiseOption[] = [
  { type: 'white', icon: '❄️' },
  { type: 'pink', icon: '🌊' },
  { type: 'brown', icon: '🌫️' },
  { type: 'rain', icon: '🌧️' },
];

const NOISE_SAMPLE_RATE = 96000;
const NOISE_LOOP_MS = 15000; // 15 秒循环缓冲（无缝循环）

const noiseType = ref<NoiseType>('white');
const playing = ref(false);
const volume = ref(0.6);
const seed = ref(42);
const noiseCache = new Map<string, AudioBuffer>();

let audioCtx: AudioContext | null = null;
let source: AudioBufferSourceNode | null = null;
let gainNode: GainNode | null = null;

// 持续随机模式（AudioWorklet 实时生成，永不循环重复）
const mode = ref<'loop' | 'live'>('loop');
let workletNode: AudioWorkletNode | null = null;
let workletReady = false;

function getCtx(): AudioContext | null {
  if (audioCtx) return audioCtx;
  try {
    audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
  } catch (e) {
    console.error('AudioContext 创建失败:', e);
    audioCtx = null;
  }
  return audioCtx;
}

function generateNoise(type: NoiseType): Float32Array | null {
  switch (type) {
    case 'white': return synth_white_noise(NOISE_LOOP_MS, NOISE_SAMPLE_RATE, seed.value);
    case 'pink': return synth_pink_noise(NOISE_LOOP_MS, NOISE_SAMPLE_RATE, seed.value);
    case 'brown': return synth_brown_noise(NOISE_LOOP_MS, NOISE_SAMPLE_RATE, seed.value);
    case 'rain': return synth_rain_noise(NOISE_LOOP_MS, NOISE_SAMPLE_RATE, seed.value);
  }
}

function getBuffer(type: NoiseType): AudioBuffer | null {
  const ctx = getCtx();
  if (!ctx) return null;
  let buf = noiseCache.get(type);
  if (!buf) {
    const samples = generateNoise(type);
    if (!samples || samples.length === 0) return null;
    buf = ctx.createBuffer(1, samples.length, NOISE_SAMPLE_RATE);
    buf.copyToChannel(samples, 0);
    noiseCache.set(type, buf);
  }
  return buf;
}

function applyVolume() {
  if (gainNode) gainNode.gain.value = volume.value;
}

function stopAll() {
  source?.stop();
  source?.disconnect();
  source = null;
  workletNode?.disconnect();
  workletNode = null;
  playing.value = false;
}

async function ensureWorklet(ctx: AudioContext) {
  if (workletReady) return;
  await ctx.audioWorklet.addModule(`${import.meta.env.BASE_URL}noise-worklet.js`);
  workletReady = true;
}

async function toggle() {
  const ctx = getCtx();
  if (!ctx) return;
  if (ctx.state === 'suspended') ctx.resume();

  if (playing.value) {
    stopAll();
    return;
  }

  gainNode = ctx.createGain();
  gainNode.gain.value = volume.value;
  gainNode.connect(ctx.destination);

  if (mode.value === 'live') {
    // 持续随机：AudioWorklet 实时生成（永不循环重复）
    try {
      await ensureWorklet(ctx);
      workletNode = new AudioWorkletNode(ctx, 'noise-worklet');
      workletNode.port.postMessage({ type: noiseType.value, seed: seed.value });
      workletNode.connect(gainNode);
      playing.value = true;
    } catch (e) {
      console.error('Worklet 启动失败:', e);
      workletNode?.disconnect();
      workletNode = null;
      gainNode.disconnect();
      gainNode = null;
    }
    return;
  }

  const buf = getBuffer(noiseType.value);
  if (!buf) return;

  // 每次生成新种子，同类型可换变化（对雨声/粉红明显）
  seed.value = (seed.value * 2654435761 + 1013904223) >>> 0;

  source = ctx.createBufferSource();
  source.buffer = buf;
  source.loop = true;
  source.connect(gainNode);
  source.start();
  playing.value = true;
}

function selectType(type: NoiseType) {
  if (type === noiseType.value) return;
  noiseType.value = type;
  if (playing.value) {
    if (mode.value === 'live') {
      // 实时模式：通知 worklet 切音色（无缝切换）
      workletNode?.port.postMessage({ type });
    } else {
      // 循环模式：停旧播新
      stopAll();
      toggle();
    }
  }
}

function newSeed() {
  seed.value = Math.floor(Math.random() * 0xffffffff);
  if (playing.value) {
    if (mode.value === 'live') {
      workletNode?.port.postMessage({ seed: seed.value });
    } else {
      noiseCache.clear();
      stopAll();
      toggle();
    }
  } else {
    noiseCache.clear();
  }
}

function setMode(m: 'loop' | 'live') {
  if (m === mode.value) return;
  const wasPlaying = playing.value;
  stopAll();
  mode.value = m;
  if (wasPlaying) toggle();
}

onUnmounted(() => {
  stopAll();
  if (audioCtx) audioCtx.close();
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-[#0f2027] via-[#203a43] to-[#2c5364] p-4 md:p-8 flex flex-col items-center">
    <h1 class="text-2xl md:text-3xl font-bold text-white mb-2">{{ t('noise.title') }}</h1>
    <p class="text-white/60 text-sm mb-6">{{ t('noise.subtitle') }}</p>

    <!-- 模式切换 -->
    <div class="flex gap-2 mb-6 bg-black/30 rounded-full p-1 border border-white/10">
      <button
        class="px-5 py-1.5 rounded-full text-sm font-medium transition-colors"
        :class="mode === 'loop' ? 'bg-emerald-500 text-black' : 'text-white/70 hover:bg-white/10'"
        @click="setMode('loop')"
      >
        {{ t('noise.modeLoop') }}
      </button>
      <button
        class="px-5 py-1.5 rounded-full text-sm font-medium transition-colors"
        :class="mode === 'live' ? 'bg-emerald-500 text-black' : 'text-white/70 hover:bg-white/10'"
        @click="setMode('live')"
      >
        {{ t('noise.modeLive') }}
      </button>
    </div>

    <!-- 音色选择 -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
      <button
        v-for="opt in NOISE_OPTIONS"
        :key="opt.type"
        class="rounded-xl p-4 flex flex-col items-center gap-2 transition-all border"
        :class="noiseType === opt.type
          ? 'bg-emerald-500/20 border-emerald-400 text-emerald-200 shadow-lg'
          : 'bg-white/5 border-white/10 text-white/70 hover:bg-white/10'"
        @click="selectType(opt.type)"
      >
        <span class="text-3xl">{{ opt.icon }}</span>
        <span class="text-sm font-medium">{{ t(`noise.${opt.type}`) }}</span>
        <span class="text-[11px] text-white/50">{{ t(`noise.${opt.type}Desc`) }}</span>
      </button>
    </div>

    <!-- 播放控制 -->
    <div class="w-full max-w-2xl bg-black/30 rounded-2xl p-6 border border-white/10 flex flex-col items-center gap-5">
      <button
        class="w-24 h-24 rounded-full flex items-center justify-center text-4xl transition-transform hover:scale-105 active:scale-95"
        :class="playing ? 'bg-red-500 text-white shadow-[0_0_30px_rgba(239,68,68,0.5)]' : 'bg-emerald-500 text-white shadow-[0_0_30px_rgba(16,185,129,0.5)]'"
        @click="toggle"
      >
        {{ playing ? '⏸' : '▶' }}
      </button>

      <div class="w-full max-w-sm">
        <label class="flex justify-between text-xs text-white/60 mb-1">
          <span>{{ t('noise.volume') }}</span>
          <span>{{ Math.round(volume * 100) }}%</span>
        </label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          v-model.number="volume"
          class="w-full accent-emerald-400"
          @input="applyVolume"
        />
      </div>

      <button
        class="px-4 py-1.5 rounded-full text-sm bg-white/10 text-white/70 hover:bg-white/20 transition-colors"
        @click="newSeed"
      >
        {{ t('noise.reshuffle') }}
      </button>
    </div>

    <div class="mt-4 text-white/40 text-xs text-center max-w-md">
      {{ t('noise.hint') }}
    </div>
  </div>
</template>
