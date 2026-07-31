<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 音符定义（C2 - C7，采样文件名 a48.mp3 等，与 AutoPiano 一致）
const WHITE_NOTES = [
  { note: 'C2', file: 'a49', key: '1' },
  { note: 'D2', file: 'a50', key: '2' },
  { note: 'E2', file: 'a51', key: '3' },
  { note: 'F2', file: 'a52', key: '4' },
  { note: 'G2', file: 'a53', key: '5' },
  { note: 'A2', file: 'a54', key: '6' },
  { note: 'B2', file: 'a55', key: '7' },
  { note: 'C3', file: 'a56', key: '8' },
  { note: 'D3', file: 'a57', key: '9' },
  { note: 'E3', file: 'a48', key: '0' },
  { note: 'F3', file: 'a81', key: 'Q' },
  { note: 'G3', file: 'a87', key: 'W' },
  { note: 'A3', file: 'a69', key: 'E' },
  { note: 'B3', file: 'a82', key: 'R' },
  { note: 'C4', file: 'a84', key: 'T' },
  { note: 'D4', file: 'a89', key: 'Y' },
  { note: 'E4', file: 'a85', key: 'U' },
  { note: 'F4', file: 'a73', key: 'I' },
  { note: 'G4', file: 'a79', key: 'O' },
  { note: 'A4', file: 'a80', key: 'P' },
  { note: 'B4', file: 'a65', key: 'A' },
  { note: 'C5', file: 'a83', key: 'S' },
  { note: 'D5', file: 'a68', key: 'D' },
  { note: 'E5', file: 'a70', key: 'F' },
  { note: 'F5', file: 'a71', key: 'G' },
  { note: 'G5', file: 'a72', key: 'H' },
  { note: 'A5', file: 'a74', key: 'J' },
  { note: 'B5', file: 'a75', key: 'K' },
  { note: 'C6', file: 'a76', key: 'L' },
  { note: 'D6', file: 'a90', key: 'Z' },
  { note: 'E6', file: 'a88', key: 'X' },
  { note: 'F6', file: 'a67', key: 'C' },
  { note: 'G6', file: 'a86', key: 'V' },
  { note: 'A6', file: 'a66', key: 'B' },
  { note: 'B6', file: 'a78', key: 'N' },
  { note: 'C7', file: 'a77', key: 'M' },
];

const BLACK_NOTES: Record<string, string> = {
  'C#2': 'b49', 'D#2': 'b50', 'F#2': 'b52', 'G#2': 'b53', 'A#2': 'b54',
  'C#3': 'b56', 'D#3': 'b57', 'F#3': 'b81', 'G#3': 'b87', 'A#3': 'b69',
  'C#4': 'b84', 'D#4': 'b89', 'F#4': 'b73', 'G#4': 'b79', 'A#4': 'b80',
  'C#5': 'b83', 'D#5': 'b68', 'F#5': 'b71', 'G#5': 'b72', 'A#5': 'b74',
  'C#6': 'b76', 'D#6': 'b90', 'F#6': 'b67', 'G#6': 'b86', 'A#6': 'b66',
};

// 黑键相对其左侧白键的偏移（第几个白键之后）
const BLACK_AFTER: Record<string, number> = {
  'C#2': 0, 'D#2': 1, 'F#2': 3, 'G#2': 4, 'A#2': 5,
  'C#3': 7, 'D#3': 8, 'F#3': 10, 'G#3': 11, 'A#3': 12,
  'C#4': 14, 'D#4': 15, 'F#4': 17, 'G#4': 18, 'A#4': 19,
  'C#5': 21, 'D#5': 22, 'F#5': 24, 'G#5': 25, 'A#5': 26,
  'C#6': 28, 'D#6': 29, 'F#6': 31, 'G#6': 32, 'A#6': 33,
};

const WHITE_KEYS: Record<string, string> = {};
WHITE_NOTES.forEach(n => { WHITE_KEYS[n.key] = n.note; });

const sampleBase = import.meta.env.BASE_URL + 'samples/piano/';

const audioCtx = ref<AudioContext | null>(null);
const buffers = new Map<string, AudioBuffer>();
const loading = ref(0);
const totalSamples = 61;
const ready = computed(() => loading.value >= totalSamples);
const errorMsg = ref('');
const activeKeys = ref<Set<string>>(new Set()); // 按下的音符名

const keyToNote = new Map<string, { note: string; file: string }>();
WHITE_NOTES.forEach(n => keyToNote.set(n.key, { note: n.note, file: n.file }));

// 黑键的 Shift 映射：黑键 = 对应白键左侧的白键 + Shift
const shiftMap = new Map<string, { note: string; file: string }>();
Object.entries(BLACK_NOTES).forEach(([note, file]) => {
  const afterIndex = BLACK_AFTER[note];
  const leftWhite = WHITE_NOTES[afterIndex];
  if (leftWhite) shiftMap.set(leftWhite.key, { note, file });
});

function getAudioCtx(): AudioContext | null {
  if (audioCtx.value) return audioCtx.value;
  try {
    audioCtx.value = new (window.AudioContext || (window as any).webkitAudioContext)();
    return audioCtx.value;
  } catch (e) {
    errorMsg.value = t('piano.audioUnsupported');
    return null;
  }
}

async function loadSamples() {
  const ctx = getAudioCtx();
  if (!ctx) return;

  const files = new Set<string>();
  WHITE_NOTES.forEach(n => files.add(n.file));
  Object.values(BLACK_NOTES).forEach(f => files.add(f));

  for (const file of files) {
    try {
      const res = await fetch(`${sampleBase}${file}.mp3`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const arrayBuf = await res.arrayBuffer();
      const audioBuf = await ctx.decodeAudioData(arrayBuf);
      buffers.set(file, audioBuf);
    } catch (e) {
      console.error(`加载采样失败: ${file}.mp3`, e);
      errorMsg.value = t('piano.sampleLoadFailed', { file: `${file}.mp3` });
    } finally {
      loading.value++;
    }
  }
}

function playNote(note: string) {
  const ctx = getAudioCtx();
  if (!ctx) return;
  if (ctx.state === 'suspended') ctx.resume();

  const file = WHITE_NOTES.find(n => n.note === note)?.file ?? BLACK_NOTES[note];
  if (!file) return;
  const buffer = buffers.get(file);
  if (!buffer) return;

  const source = ctx.createBufferSource();
  source.buffer = buffer;
  source.connect(ctx.destination);
  source.start();
}

function pressNote(note: string) {
  if (activeKeys.value.has(note)) return;
  activeKeys.value.add(note);
  playNote(note);
}

function releaseNote(note: string) {
  activeKeys.value.delete(note);
}

function onKeyDown(e: KeyboardEvent) {
  if (e.repeat) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  const target = e.target as HTMLElement | null;
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT')) return;

  if (e.shiftKey) {
    const entry = shiftMap.get(e.key.toUpperCase());
    if (entry) { e.preventDefault(); pressNote(entry.note); return; }
  }
  const entry = keyToNote.get(e.key.toUpperCase());
  if (entry) { e.preventDefault(); pressNote(entry.note); }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.shiftKey) {
    const entry = shiftMap.get(e.key.toUpperCase());
    if (entry) { releaseNote(entry.note); return; }
  }
  const entry = keyToNote.get(e.key.toUpperCase());
  if (entry) releaseNote(entry.note);
}

const whiteNotes = WHITE_NOTES.map(n => n.note);
const blackNotes = Object.entries(BLACK_AFTER)
  .sort((a, b) => a[1] - b[1])
  .map(([note]) => note);

function isActive(note: string): boolean {
  return activeKeys.value.has(note);
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  loadSamples();
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('keyup', onKeyUp);
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-[#1a1a2e] to-[#16213e] p-4 md:p-8 flex flex-col items-center">
    <h1 class="text-2xl md:text-3xl font-bold text-white mb-2">{{ t('piano.title') }}</h1>
    <p class="text-white/60 text-sm mb-4">{{ t('piano.subtitle') }}</p>

    <!-- 状态栏 -->
    <div class="mb-4 text-sm">
      <span v-if="!ready" class="text-white/70">
        {{ t('piano.loading', { current: loading, total: totalSamples }) }}
      </span>
      <span v-else class="text-emerald-400">{{ t('piano.ready') }}</span>
      <span v-if="errorMsg" class="text-red-400 ml-3">{{ errorMsg }}</span>
    </div>

    <!-- 钢琴 -->
    <div class="w-full max-w-4xl select-none" @mousedown.prevent>
      <div class="relative h-56 md:h-72 rounded-b-lg overflow-hidden shadow-2xl">
        <!-- 白键 -->
        <div class="flex h-full">
          <div
            v-for="note in whiteNotes"
            :key="note"
            class="flex-1 relative border border-gray-300 rounded-b-md transition-all duration-75 cursor-pointer"
            :class="isActive(note)
              ? 'bg-gradient-to-b from-amber-200 to-amber-400 shadow-inner'
              : 'bg-gradient-to-b from-white to-gray-200 hover:from-gray-50'"
            @pointerdown="pressNote(note)"
            @pointerup="releaseNote(note)"
            @pointerleave="releaseNote(note)"
            @contextmenu.prevent
          >
            <span class="absolute bottom-1 left-0 right-0 text-center text-xs text-gray-500 select-none">
              {{ WHITE_NOTES.find(n => n.note === note)?.key }}
            </span>
          </div>
        </div>

        <!-- 黑键 -->
        <div class="absolute inset-0 pointer-events-none">
          <div
            v-for="note in blackNotes"
            :key="note"
            class="absolute w-[6%] h-[62%] rounded-b-md transition-all duration-75 cursor-pointer pointer-events-auto"
            :class="isActive(note)
              ? 'bg-gradient-to-b from-amber-600 to-amber-800 shadow-[0_0_12px_rgba(245,158,11,0.8)]'
              : 'bg-gradient-to-b from-[#333] to-black border border-black/60 shadow-[inset_0_-2px_4px_rgba(255,255,255,0.15),0_3px_6px_rgba(0,0,0,0.6)]'"
            :style="{ left: `calc(${(BLACK_AFTER[note] + 1) * (100 / whiteNotes.length)}% - 3%)` }"
            @pointerdown="pressNote(note)"
            @pointerup="releaseNote(note)"
            @pointerleave="releaseNote(note)"
          >
            <span class="absolute bottom-1 left-0 right-0 text-center text-[10px] text-white/60 select-none">
              {{ WHITE_NOTES[BLACK_AFTER[note]]?.key }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 按键提示 -->
    <div class="mt-6 max-w-4xl w-full text-center text-white/50 text-xs leading-6">
      {{ t('piano.keyHint') }}
    </div>
  </div>
</template>
