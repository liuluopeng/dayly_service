<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Howl } from 'howler';
import * as wasm from '../types/wasm-typed';
import { useMusicPlayer } from '../composables/useMusicPlayer';
import type { SongWithUrl, LyricsResponse, LyricsLine } from '../types/models';

const { t } = useI18n();

// 频谱全局单例 — 不随组件销毁而重建
let globalCtx: AudioContext | null = null;
let globalAnalyser: AnalyserNode | null = null;
let globalSource: MediaElementAudioSourceNode | null = null;
let connectedEl: HTMLAudioElement | null = null;

const route = useRoute();
const router = useRouter();
const { next, previous, hasNext, hasPrevious, reset } = useMusicPlayer();

const song = ref<SongWithUrl | null>(null);
const sound = ref<Howl | null>(null);
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const volume = ref(0.8);
const isMuted = ref(false);
const isLoading = ref(false);
const audioUrl = ref<string | null>(null);
const isSwitching = ref(false);
const lyricsText = ref<string>('');
const activeLyricIndex = ref(-1);
const lyricsContainerRef = ref<HTMLElement | null>(null);

let progressInterval: number | null = null;

const BAR_COUNT = 48;
const spectrumData = ref<number[]>(new Array(BAR_COUNT).fill(0));
let animFrameId: number | null = null;

const formattedCurrentTime = computed(() => formatTime(currentTime.value));
const formattedDuration = computed(() => formatTime(duration.value));
const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0);

const parsedLyrics = computed<LyricsLine[]>(() => {
  if (!lyricsText.value) return [];
  const lines: LyricsLine[] = [];
  const regex = /\[(\d{2}):(\d{2})\.(\d{2,3})\]/g;
  const textParts = lyricsText.value.split(/\n(?=\[)/);
  for (const part of textParts) {
    let match: RegExpExecArray | null;
    let lastTime = -1;
    while ((match = regex.exec(part)) !== null) {
      const mins = parseInt(match[1], 10);
      const secs = parseInt(match[2], 10);
      let ms = parseInt(match[3], 10);
      if (ms < 100) ms *= 10;
      lastTime = mins * 60 + secs + ms / 1000;
    }
    if (lastTime >= 0) {
      const text = part.replace(/\[(\d{2}):(\d{2})\.(\d{2,3})\]/g, '').trim();
      if (text) lines.push({ time: lastTime, text });
    }
  }
  return lines;
});

let lyricsSyncInterval: number | null = null;

function startLyricsSync() {
  stopLyricsSync();
  lyricsSyncInterval = window.setInterval(() => {
    if (!parsedLyrics.value.length) return;
    for (let i = parsedLyrics.value.length - 1; i >= 0; i--) {
      if (currentTime.value >= parsedLyrics.value[i].time) {
        activeLyricIndex.value = i;
        break;
      }
      activeLyricIndex.value = -1;
    }
    scrollToActiveLyric();
  }, 200);
}

function stopLyricsSync() {
  if (lyricsSyncInterval) {
    clearInterval(lyricsSyncInterval);
    lyricsSyncInterval = null;
  }
}

function scrollToActiveLyric() {
  if (!lyricsContainerRef.value) return;
  const activeEl = lyricsContainerRef.value.querySelector('.lyric-active');
  if (activeEl) {
    activeEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

async function loadLyrics() {
  if (!song.value) return;
  lyricsText.value = '';
  activeLyricIndex.value = -1;
  try {
    const data = await wasm.get_song_lyrics_wasm(song.value.id) as LyricsResponse;
    lyricsText.value = data.lyrics;
  } catch (_) {
    lyricsText.value = '';
  }
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function setupSpectrum() {
  try {
    const audioEl = (sound.value as any)?._sounds?.[0]?._node as HTMLAudioElement;
    if (!audioEl) return;
    if (audioEl === connectedEl && globalAnalyser) return;

    if (!globalCtx) {
      globalCtx = new AudioContext();
    }

    if (globalSource) { globalSource.disconnect(); globalSource = null; }
    if (globalAnalyser) { globalAnalyser.disconnect(); globalAnalyser = null; }

    globalAnalyser = globalCtx.createAnalyser();
    globalAnalyser.fftSize = 512;
    globalAnalyser.smoothingTimeConstant = 0.5;
    globalAnalyser.minDecibels = -90;
    globalAnalyser.maxDecibels = -10;

    globalSource = globalCtx.createMediaElementSource(audioEl);
    globalSource.connect(globalAnalyser);
    globalAnalyser.connect(globalCtx.destination);
    connectedEl = audioEl;
  } catch (e) {
    if (!globalAnalyser) console.warn('频谱初始化失败:', e);
  }
}

function startSpectrum() {
  if (!globalAnalyser || !globalCtx) return;
  if (globalCtx.state === 'suspended') globalCtx.resume();

  const sampleRate = globalCtx.sampleRate;
  const fftSize = globalAnalyser.fftSize;
  const bufferLen = globalAnalyser.frequencyBinCount;
  const dataArray = new Uint8Array(bufferLen);

  const logMin = Math.log(20);
  const logMax = Math.log(20000);
  const binForBar = new Uint16Array(BAR_COUNT);
  for (let i = 0; i < BAR_COUNT; i++) {
    const freq = Math.exp(logMin + (logMax - logMin) * i / (BAR_COUNT - 1));
    binForBar[i] = Math.min(Math.floor(freq * fftSize / sampleRate), bufferLen - 1);
  }

  function animate() {
    globalAnalyser!.getByteFrequencyData(dataArray);
    for (let i = 0; i < BAR_COUNT; i++) {
      spectrumData.value[i] = dataArray[binForBar[i]] / 255;
    }
    animFrameId = requestAnimationFrame(animate);
  }
  animate();
}

function stopSpectrum() {
  if (animFrameId) {
    cancelAnimationFrame(animFrameId);
    animFrameId = null;
  }
  spectrumData.value = new Array(BAR_COUNT).fill(0);
}

function initSong() {
  const songData = route.query.song;
  if (songData && typeof songData === 'string') {
    song.value = JSON.parse(decodeURIComponent(songData));
    loadSong();
  }
}

// ===== WASM 零拷贝加载 =====
async function loadSong() {
  if (!song.value) return;

  if (sound.value) {
    sound.value.unload();
  }
  if (audioUrl.value) {
    URL.revokeObjectURL(audioUrl.value);
    audioUrl.value = null;
  }
  stopSpectrum();
  lyricsText.value = '';
  activeLyricIndex.value = -1;

  isLoading.value = true;

  try {
    loadLyrics();

    const [ptr, len] = await wasm.load_song_audio_zc(song.value.id) as [number, number];
    const mem = wasm.get_wasm_memory();
    const buffer = (mem as any).buffer as ArrayBuffer;
    const audioData = new Uint8Array(buffer, ptr, len);
    const blob = new Blob([audioData]);
    audioUrl.value = URL.createObjectURL(blob);

    sound.value = new Howl({
      src: [audioUrl.value!],
      html5: true,
      autoplay: true,
      volume: volume.value,
      onload: () => {
        duration.value = sound.value?.duration() || 0;
        isLoading.value = false;
        setupSpectrum();
        if (isPlaying.value) startSpectrum();
      },
      onplay: () => {
        isPlaying.value = true;
        startProgressTracking();
        startSpectrum();
        startLyricsSync();
      },
      onpause: () => {
        isPlaying.value = false;
        stopProgressTracking();
        stopSpectrum();
        stopLyricsSync();
      },
      onend: () => {
        isPlaying.value = false;
        currentTime.value = 0;
        activeLyricIndex.value = 0;
        stopProgressTracking();
        stopSpectrum();
        stopLyricsSync();
      },
      onloaderror: () => {
        isLoading.value = false;
        console.error('音频加载失败');
      }
    });
  } catch (error) {
    isLoading.value = false;
    console.error('获取歌曲文件失败:', error);
  }
}

function startProgressTracking() {
  stopProgressTracking();
  progressInterval = window.setInterval(() => {
    if (sound.value && isPlaying.value) {
      currentTime.value = sound.value.seek() as number;
    }
  }, 100);
}

function stopProgressTracking() {
  if (progressInterval) {
    clearInterval(progressInterval);
    progressInterval = null;
  }
}

function togglePlay() {
  if (!sound.value) return;

  if (isPlaying.value) {
    sound.value.pause();
  } else {
    sound.value.play();
  }
}

function seek(event: Event) {
  const target = event.target as HTMLInputElement;
  const seekTime = (parseFloat(target.value) / 100) * duration.value;
  if (sound.value) {
    sound.value.seek(seekTime);
    currentTime.value = seekTime;
  }
}

function toggleMute() {
  isMuted.value = !isMuted.value;
  if (sound.value) {
    sound.value.volume(isMuted.value ? 0 : volume.value);
  }
}

function changeVolume(event: Event) {
  const target = event.target as HTMLInputElement;
  volume.value = parseFloat(target.value);
  if (sound.value && !isMuted.value) {
    sound.value.volume(volume.value);
  }
}

function goBack() {
  if (sound.value) {
    sound.value.unload();
  }
  reset();
  router.push('/songs?wasm=1');
}

async function playNext() {
  if (!hasNext.value || isSwitching.value) return;

  isSwitching.value = true;
  try {
    const nextSong = await next();
    if (nextSong) {
      song.value = nextSong;
      await loadSong();
      sound.value?.play();
    }
  } finally {
    isSwitching.value = false;
  }
}

async function playPrevious() {
  if (!hasPrevious.value || isSwitching.value) return;

  isSwitching.value = true;
  try {
    const prevSong = await previous();
    if (prevSong) {
      song.value = prevSong;
      await loadSong();
      sound.value?.play();
    }
  } finally {
    isSwitching.value = false;
  }
}

watch(() => route.query, initSong, { immediate: true });

onUnmounted(() => {
  if (sound.value) {
    sound.value.unload();
  }
  if (audioUrl.value) {
    URL.revokeObjectURL(audioUrl.value);
  }
  stopProgressTracking();
  stopSpectrum();
  stopLyricsSync();
});
</script>

<template>
  <div class="min-h-screen relative flex flex-col overflow-hidden">
    <!-- 模糊背景 -->
    <div v-if="song?.cover_url" class="absolute inset-0 bg-cover bg-center bg-no-repeat"
      :style="{ backgroundImage: `url(${song.cover_url})` }">
      <div class="absolute inset-0 backdrop-blur-3xl bg-black/50"></div>
    </div>
    <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900"></div>

    <!-- 顶栏 -->
    <div class="relative z-10 flex justify-between items-center px-6 py-4">
      <button @click="goBack" class="flex items-center gap-2 text-white/70 hover:text-white transition-colors">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        {{ t('musicPlayerWasm.backToList') }}
      </button>
      <span class="text-xs text-purple-300/80 bg-purple-500/10 px-3 py-1 rounded-full">{{ t('musicPlayerWasm.wasmZeroCopy') }}</span>
    </div>

    <!-- 主体：横向布局 -->
    <div class="relative z-10 flex-1 flex items-center justify-center px-8">
      <div class="flex items-center gap-10 max-w-4xl w-full">
        <!-- 封面 -->
        <div class="relative flex-shrink-0">
          <div class="w-72 h-72 rounded-2xl overflow-hidden shadow-2xl">
            <img v-if="song?.cover_url" :src="song.cover_url" :alt="song?.title" class="w-full h-full object-cover" />
            <div v-else
              class="w-full h-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
              <svg class="w-20 h-20 text-white/50" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
              </svg>
            </div>
          </div>
          <div v-if="isPlaying"
            class="absolute inset-0 rounded-2xl bg-gradient-to-r from-purple-500 to-pink-500 opacity-30 animate-pulse">
          </div>
        </div>

        <!-- 右侧：信息 + 频谱 + 进度 + 控制 -->
        <div class="flex-1 min-w-0">
          <div class="mb-6">
            <h2 class="text-3xl font-bold text-white mb-1 truncate">{{ song?.title || t('musicPlayerWasm.unknownSong') }}</h2>
            <p class="text-white/50 text-lg">{{ song?.artist || t('musicPlayerWasm.unknownArtist') }} - {{ song?.album || t('musicPlayerWasm.unknownAlbum') }}</p>
          </div>

          <!-- 歌词固定区域 -->
          <div ref="lyricsContainerRef"
            class="mb-4 h-44 overflow-y-auto rounded-xl bg-white/5 backdrop-blur px-4 py-2 text-sm select-none
                   [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
            <p v-if="!parsedLyrics.length" class="text-white/20 text-center leading-[9rem]">{{ t('musicPlayerWasm.noLyrics') }}</p>
            <p v-for="(line, i) in parsedLyrics" :key="i"
              :class="['py-0.5 transition-all duration-300',
                       i === activeLyricIndex ? 'lyric-active text-purple-300 font-bold text-base' : 'text-white/50']">
              {{ line.text }}
            </p>
          </div>

          <!-- 频谱 -->
          <div class="mb-6">
            <div class="flex items-end gap-[2px] h-20">
              <div v-for="(val, i) in spectrumData" :key="i"
                class="flex-1 rounded-b-sm"
                :style="{
                  height: Math.max(val * 100, 1) + '%',
                  background: `hsl(${260 + (i / BAR_COUNT) * 40}, 80%, ${55 + val * 35}%)`,
                  opacity: 0.4 + val * 0.6,
                }" />
            </div>
            <div class="flex justify-between text-[10px] text-white/25 mt-1 px-0.5">
              <span>20</span><span>200</span><span>2k</span><span>20k</span>
            </div>
          </div>

          <!-- 进度条 -->
          <div class="mb-6">
            <input type="range" min="0" max="100" :value="progress" @input="seek"
              class="w-full h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer
                     [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3.5 [&::-webkit-slider-thumb]:h-3.5
                     [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full
                     [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:cursor-pointer" />
            <div class="flex justify-between text-xs text-white/40 mt-1.5">
              <span>{{ formattedCurrentTime }}</span>
              <span>{{ formattedDuration }}</span>
            </div>
          </div>

          <!-- 控制栏 -->
          <div class="flex items-center gap-5">
            <button @click="playPrevious" :disabled="!hasPrevious || isSwitching"
              class="text-white/60 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
              <svg class="w-7 h-7" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" />
              </svg>
            </button>

            <button @click="togglePlay" :disabled="isLoading || isSwitching"
              class="w-14 h-14 bg-white rounded-full flex items-center justify-center shadow-lg hover:shadow-xl
                     transform hover:scale-105 transition-all
                     disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none">
              <div v-if="isLoading || isSwitching"
                class="w-7 h-7 border-4 border-gray-300 border-t-gray-600 rounded-full animate-spin"></div>
              <svg v-else-if="!isPlaying" class="w-7 h-7 text-gray-900 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z" />
              </svg>
              <svg v-else class="w-7 h-7 text-gray-900" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
              </svg>
            </button>

            <button @click="playNext" :disabled="!hasNext || isSwitching"
              class="text-white/60 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
              <svg class="w-7 h-7" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" />
              </svg>
            </button>

            <!-- 音量 -->
            <div class="flex items-center gap-2 ml-2">
              <button @click="toggleMute" class="text-white/60 hover:text-white transition-colors flex-shrink-0">
                <svg v-if="!isMuted" class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z" />
                </svg>
                <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45h.05V12zM9 11.35V7.5h2l-.01.01L9 5.54v-.01c0-.45-.54-.67-.85-.35L3.7 9.65c-.2.2-.2.51 0 .71l4.45 4.45c.31.31.85.09.85-.35v-3.11l4.19 4.19c.2.2.51.2.71 0l1.41-1.41c.2-.2.2-.51 0-.71L11 11.33V12c0 1.77 1.02 3.29 2.5 4.03v2.21l2.45 2.45c.2.2.51.2.71 0l1.41-1.41c.2-.2.2-.51 0-.71l-2.55-2.55c1.11-.21 2.03-1.14 2.37-2.33h.01c.08-.32.13-.65.13-1 .09-.36-.02-.69-.13-1H19c-.34-1.92-1.74-3.5-3.5-4.07v2.05c.62.29 1.1.86 1.33 1.54h.01c.08.23.14.47.16.73H17v-.01zM7.11 9l-.6-.61L9.23 5.67c.31-.31.85-.09.85.35v3.33l-.01.01L7.11 9z" />
                </svg>
              </button>
              <input type="range" min="0" max="1" step="0.01" :value="isMuted ? 0 : volume" @input="changeVolume"
                class="w-24 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full" />
            </div>

          </div>
        </div>
      </div>
    </div>
  </div>
</template>
