<script setup lang="ts">
import { ref, shallowRef, computed, onUnmounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Howl } from 'howler';
import * as wasm from '../types/wasm-typed';
import { useMusicPlayer } from '../composables/useMusicPlayer';
import { LyricPlayer } from '@applemusic-like-lyrics/vue';
import type { LyricLine } from '@applemusic-like-lyrics/core';
import '@applemusic-like-lyrics/core/style.css';
import { parseTTML } from '@applemusic-like-lyrics/ttml';
import { parseLrc, parseEslrc, parseQrc, parseYrc, parseLys } from '@applemusic-like-lyrics/lyric';
import type { SongWithUrl, AllLyricsResponse } from '../types/models';

const { t } = useI18n();

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

// 音频信息
const audioInfo = ref<{ format: string; bitrate: string; sampleRate: string } | null>(null);

// AMLL 歌词相关 — shallowRef 避免 structuredClone 克隆不了 Vue 响应式代理
const amllLyricLines = shallowRef<LyricLine[]>([]);
const lyricCurrentTime = ref(0);
const lyricPlaying = ref(false);

// 多歌词格式支持
type LyricType = 'ttml' | 'auto_ttml' | 'lrc' | 'eslrc' | 'qrc' | 'yrc' | 'lys';
const lyricTypeLabels: Record<LyricType, string> = {
  ttml: 'TTML',
  auto_ttml: '自动TTML',
  lrc: 'LRC',
  eslrc: 'ESLRC',
  qrc: 'QRC',
  yrc: 'YRC',
  lys: 'LYS',
};
const availableLyricTypes = ref<LyricType[]>([]);
const currentLyricType = ref<LyricType | null>(null);
const allLyricsData = ref<AllLyricsResponse | null>(null);

let progressInterval: number | null = null;
let rAFId: number | null = null;

const BAR_COUNT = 48;
const spectrumData = ref<number[]>(new Array(BAR_COUNT).fill(0));
let animFrameId: number | null = null;

const formattedCurrentTime = computed(() => formatTime(currentTime.value));
const formattedDuration = computed(() => formatTime(duration.value));
const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0);

// 频谱频率标注
const freqLabels = computed(() => {
  const logMin = Math.log(20);
  const logMax = Math.log(20000);
  const marks = [50, 100, 200, 500, 1000, 2000, 5000, 10000];
  return marks.map(f => {
    const percent = ((Math.log(f) - logMin) / (logMax - logMin)) * 100;
    const label = f >= 1000 ? `${f / 1000}k` : `${f}`;
    return { percent, label };
  });
});

async function loadLyrics() {
  if (!song.value) return;
  amllLyricLines.value = [];
  availableLyricTypes.value = [];
  currentLyricType.value = null;
  allLyricsData.value = null;

  try {
    const data = await wasm.get_all_lyrics_wasm(song.value.id) as AllLyricsResponse;
    allLyricsData.value = data;

    // 检查哪些格式有内容
    const types: LyricType[] = [];
    if (data.ttml) types.push('ttml');
    if (data.auto_ttml) types.push('auto_ttml');
    if (data.lrc) types.push('lrc');
    if (data.eslrc) types.push('eslrc');
    if (data.qrc) types.push('qrc');
    if (data.yrc) types.push('yrc');
    if (data.lys) types.push('lys');

    availableLyricTypes.value = types;

    // 按优先级选择默认歌词类型
    if (types.includes('ttml')) {
      switchLyricType('ttml');
    } else if (types.includes('lrc')) {
      switchLyricType('lrc');
    } else if (types.length > 0) {
      switchLyricType(types[0]);
    }
  } catch (e) {
    console.warn('获取歌词失败:', e);
  }
}

function switchLyricType(type: LyricType) {
  if (!allLyricsData.value) return;
  currentLyricType.value = type;

  const content = allLyricsData.value[type];
  if (!content) {
    amllLyricLines.value = [];
    return;
  }

  try {
    switch (type) {
      case 'ttml':
      case 'auto_ttml': {
        const result = parseTTML(content);
        if (result?.lines?.length) {
          amllLyricLines.value = result.lines as unknown as LyricLine[];
        }
        break;
      }
      case 'lrc':
        amllLyricLines.value = parseLrc(content);
        break;
      case 'eslrc':
        amllLyricLines.value = parseEslrc(content);
        break;
      case 'qrc':
        amllLyricLines.value = parseQrc(content);
        break;
      case 'yrc':
        amllLyricLines.value = parseYrc(content);
        break;
      case 'lys':
        amllLyricLines.value = parseLys(content);
        break;
    }
  } catch (e) {
    console.error(`解析 ${type} 歌词失败:`, e);
    amllLyricLines.value = [];
  }
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// 频谱
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

// rAF 驱动歌词时间同步 (AMLL 需要每帧调用)
function startLyricsRAF() {
  stopLyricsRAF();
  lyricPlaying.value = true;
  function onFrame() {
    if (sound.value && isPlaying.value) {
      lyricCurrentTime.value = Math.round(sound.value.seek() as number * 1000);
    }
    if (lyricPlaying.value) {
      rAFId = requestAnimationFrame(onFrame);
    }
  }
  rAFId = requestAnimationFrame(onFrame);
}

function stopLyricsRAF() {
  lyricPlaying.value = false;
  if (rAFId) {
    cancelAnimationFrame(rAFId);
    rAFId = null;
  }
}

function initSong() {
  const songData = route.query.song;
  if (songData && typeof songData === 'string') {
    song.value = JSON.parse(decodeURIComponent(songData));
    loadSong();
  }
}

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
  amllLyricLines.value = [];
  lyricCurrentTime.value = 0;

  isLoading.value = true;

  try {
    const baseUrl = wasm.get_base_url_wasm();
    const songUrl = `${baseUrl}/api/songs/file/${song.value.id}`;

    loadLyrics();

    const response = await fetch(songUrl);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const blob = await response.blob();
    audioUrl.value = URL.createObjectURL(blob);

    // 推断格式：blob URL 无扩展名，优先从 Content-Type 读取；否则回退常见格式
    const mimeType = blob?.type;
    let format: string[] | undefined;
    let formatName = '未知';
    if (mimeType) {
      const ext = mimeType.split('/')[1];
      if (ext === 'mpeg' || ext === 'mp3') { format = ['mp3']; formatName = 'MP3'; }
      else if (ext === 'flac') { format = ['flac']; formatName = 'FLAC'; }
      else if (ext === 'ogg' || ext === 'vorbis') { format = ['ogg']; formatName = 'OGG'; }
      else if (ext === 'wav' || ext === 'wave') { format = ['wav']; formatName = 'WAV'; }
    }
    const fileSize = blob.size;

    sound.value = new Howl({
      src: [audioUrl.value!],
      format,
      html5: true,
      autoplay: true,
      volume: volume.value,
      onload: () => {
        duration.value = sound.value?.duration() || 0;
        isLoading.value = false;
        setupSpectrum();
        if (isPlaying.value) startSpectrum();

        // 计算音频信息
        const dur = duration.value;
        const bitrateKbps = dur > 0 ? Math.round((fileSize * 8) / dur / 1000) : 0;
        const sampleRate = globalCtx?.sampleRate || 44100;
        audioInfo.value = {
          format: formatName,
          bitrate: `${bitrateKbps} kbps`,
          sampleRate: `${(sampleRate / 1000).toFixed(1)} kHz`,
        };
      },
      onplay: () => {
        isPlaying.value = true;
        startProgressTracking();
        startSpectrum();
        startLyricsRAF();
      },
      onpause: () => {
        isPlaying.value = false;
        stopProgressTracking();
        stopSpectrum();
        stopLyricsRAF();
      },
      onend: () => {
        isPlaying.value = false;
        currentTime.value = 0;
        lyricCurrentTime.value = 0;
        stopProgressTracking();
        stopSpectrum();
        stopLyricsRAF();
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
    lyricCurrentTime.value = Math.round(seekTime * 1000);
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

function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen();
  } else {
    document.exitFullscreen();
  }
}

function goBack() {
  if (sound.value) {
    sound.value.unload();
  }
  reset();
  router.push('/songs?lyrics=1');
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
  stopLyricsRAF();
});
</script>

<template>
  <div class="h-screen relative flex flex-col overflow-hidden">
    <!-- 模糊背景 -->
    <div v-if="song?.cover_url" class="absolute inset-0 bg-cover bg-center bg-no-repeat"
      :style="{ backgroundImage: `url(${song.cover_url})` }">
      <div class="absolute inset-0 backdrop-blur-3xl bg-black/60"></div>
    </div>
    <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900"></div>

    <!-- 顶栏 -->
    <div class="relative z-10 flex justify-between items-center px-6 py-3 flex-shrink-0">
      <button @click="goBack" class="flex items-center gap-2 text-white/70 hover:text-white transition-colors">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        {{ t('musicPlayerLyrics.backToList') }}
      </button>
      <!-- 歌词类型切换 -->
      <div v-if="availableLyricTypes.length > 1" class="flex items-center gap-1.5">
        <button v-for="type in availableLyricTypes" :key="type"
          @click="switchLyricType(type)"
          :class="[
            'px-2.5 py-1 rounded-full text-xs font-medium transition-all',
            currentLyricType === type
              ? 'bg-pink-500/80 text-white'
              : 'bg-white/10 text-white/60 hover:bg-white/20 hover:text-white'
          ]">
          {{ lyricTypeLabels[type] || type.toUpperCase() }}
        </button>
      </div>
      <span v-else class="text-xs text-pink-300/80 bg-pink-500/10 px-3 py-1 rounded-full">{{ t('musicPlayerLyrics.appleMusicLyrics') }}</span>

      <!-- 全屏按钮 -->
      <button @click="toggleFullscreen"
        class="p-2 text-white/60 hover:text-white transition-colors"
        :title="t('musicPlayerLyrics.fullscreen')">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        </svg>
      </button>
    </div>

    <!-- 主体：左右分栏 -->
    <div class="relative z-10 flex-1 flex min-h-0">

      <!-- 左侧：封面 + 控制 -->
      <div class="w-[340px] flex-shrink-0 flex flex-col items-center justify-center px-6 py-4">

        <!-- 封面 -->
        <div class="w-64 h-64 rounded-2xl overflow-hidden shadow-2xl mb-6">
          <img v-if="song?.cover_url" :src="song.cover_url" :alt="song?.title" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
            <svg class="w-16 h-16 text-white/50" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
            </svg>
          </div>
        </div>

        <!-- 歌曲信息 -->
        <div class="text-center mb-2 w-full">
          <h2 class="text-xl font-bold text-white mb-1 truncate">{{ song?.title || t('musicPlayerLyrics.unknownSong') }}</h2>
          <p class="text-white/50 text-sm truncate">{{ song?.artist || t('musicPlayerLyrics.unknownArtist') }} - {{ song?.album || t('musicPlayerLyrics.unknownAlbum') }}</p>
        </div>

        <!-- 音频信息 -->
        <div v-if="audioInfo" class="text-center mb-3 w-full">
          <span class="text-[10px] text-white/30">{{ audioInfo.format }} · {{ audioInfo.bitrate }} · {{ audioInfo.sampleRate }}</span>
        </div>

        <!-- 频谱 -->
        <div class="mb-4 w-full">
          <div class="flex items-end gap-[2px] h-10">
            <div v-for="(val, i) in spectrumData" :key="i"
              class="flex-1 rounded-b-sm"
              :style="{
                height: Math.max(val * 100, 1) + '%',
                background: `hsl(${260 + (i / BAR_COUNT) * 40}, 80%, ${55 + val * 35}%)`,
                opacity: 0.4 + val * 0.6,
              }" />
          </div>
          <!-- 频率标注 -->
          <div class="relative h-4 mt-1">
            <span v-for="f in freqLabels" :key="f.label"
              class="absolute text-[9px] text-white/30 -translate-x-1/2"
              :style="{ left: f.percent + '%' }">{{ f.label }}</span>
          </div>
        </div>

        <!-- 进度条 -->
        <div class="mb-4 w-full">
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

        <!-- 播放控制 -->
        <div class="flex items-center gap-5 mb-3">
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
        </div>

        <!-- 音量 -->
        <div class="flex items-center gap-2">
          <button @click="toggleMute" class="text-white/60 hover:text-white transition-colors flex-shrink-0">
            <svg v-if="!isMuted" class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z" />
            </svg>
            <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45h.05V12zM9 11.35V7.5h2l-.01.01L9 5.54v-.01c0-.45-.54-.67-.85-.35L3.7 9.65c-.2.2-.2.51 0 .71l4.45 4.45c.31.31.85.09.85-.35v-3.11l4.19 4.19c.2.2.51.2.71 0l1.41-1.41c.2-.2.2-.51 0-.71L11 11.33V12c0 1.77 1.02 3.29 2.5 4.03v2.21l2.45 2.45c.2.2.51.2.71 0l1.41-1.41c.2-.2.2-.51 0-.71l-2.55-2.55c1.11-.21 2.03-1.14 2.37-2.33h.01c.08-.32.13-.65.13-1 .09-.36-.02-.69-.13-1H19c-.34-1.92-1.74-3.5-3.5-4.07v2.05c.62.29 1.1.86 1.33 1.54h.01c.08.23.14.47.16.73H17v-.01zM7.11 9l-.6-.61L9.23 5.67c.31-.31.85-.09.85.35v3.33l-.01.01L7.11 9z" />
            </svg>
          </button>
          <input type="range" min="0" max="1" step="0.01" :value="isMuted ? 0 : volume" @input="changeVolume"
            class="w-28 h-1 bg-white/20 rounded-full appearance-none cursor-pointer
                   [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                   [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full" />
        </div>
      </div>

      <!-- 右侧：AMLL 歌词 -->
      <div class="flex-1 min-w-0 relative">
        <LyricPlayer
          class="absolute inset-0"
          :lyric-lines="amllLyricLines"
          :current-time="lyricCurrentTime"
          :playing="lyricPlaying"
        />
      </div>

    </div>
  </div>
</template>

