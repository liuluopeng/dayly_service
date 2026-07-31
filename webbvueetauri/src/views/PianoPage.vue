<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// ─── 音符定义（C2 - C7，采样文件名与 AutoPiano 一致） ───

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

// 黑键八度组：相对钢琴整体的 left 偏移（参考 AutoPiano：0/19.5/39/58.3/77.7%）
const BLACK_OCTAVES = [
  { left: '0%', keys: ['C#2', 'D#2', 'F#2', 'G#2', 'A#2'] },
  { left: '19.5%', keys: ['C#3', 'D#3', 'F#3', 'G#3', 'A#3'] },
  { left: '39%', keys: ['C#4', 'D#4', 'F#4', 'G#4', 'A#4'] },
  { left: '58.3%', keys: ['C#5', 'D#5', 'F#5', 'G#5', 'A#5'] },
  { left: '77.7%', keys: ['C#6', 'D#6', 'F#6', 'G#6', 'A#6'] },
];

// 黑键在组内的 left（相对组宽 20%，参考 AutoPiano：9/23/50/65/79%）
const BLACK_IN_GROUP_LEFT = ['9%', '23%', '50%', '65%', '79%'];

// 每个黑键左侧的白键索引（用于取键盘键名与高亮关联）
const BLACK_LEFT_WHITE_INDEX: Record<string, number> = {
  'C#2': 0, 'D#2': 1, 'F#2': 3, 'G#2': 4, 'A#2': 5,
  'C#3': 7, 'D#3': 8, 'F#3': 10, 'G#3': 11, 'A#3': 12,
  'C#4': 14, 'D#4': 15, 'F#4': 17, 'G#4': 18, 'A#4': 19,
  'C#5': 21, 'D#5': 22, 'F#5': 24, 'G#5': 25, 'A#5': 26,
  'C#6': 28, 'D#6': 29, 'F#6': 31, 'G#6': 32, 'A#6': 33,
};

// 键盘映射：白键 key → 音符；黑键 = Shift + 左侧白键的 key
const keyToWhite = new Map<string, string>(); // key → 白键音符
WHITE_NOTES.forEach(n => keyToWhite.set(n.key, n.note));
const shiftKeyToBlack = new Map<string, string>(); // 白键 key → 黑键音符
Object.entries(BLACK_LEFT_WHITE_INDEX).forEach(([note, idx]) => {
  shiftKeyToBlack.set(WHITE_NOTES[idx].key, note);
});

// ─── 音频 ───

const sampleBase = import.meta.env.BASE_URL + 'samples/piano/';

const audioCtx = ref<AudioContext | null>(null);
const buffers = new Map<string, AudioBuffer>();
const loading = ref(0);
const totalSamples = 61;
const ready = computed(() => loading.value >= totalSamples);
const errorMsg = ref('');
const activeKeys = ref<Set<string>>(new Set()); // 按下的音符名

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

// 键盘路径：即使音符仍处于按下状态（keyup 丢失/快速连按）也重新发声，
// 与鼠标每次点击都能触发的行为保持一致
function pressNoteFromKeyboard(note: string) {
  activeKeys.value.add(note);
  playNote(note);
}

function releaseNote(note: string) {
  activeKeys.value.delete(note);
}

// 物理键 → 实际按下的音符：keyup 时按记录释放，
// 避免 Shift 状态在按下/抬起之间变化导致误释放（键一直亮）
const pressedByKey = new Map<string, string>();

function onKeyDown(e: KeyboardEvent) {
  if (e.repeat) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  const target = e.target as HTMLElement | null;
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT')) return;

  const key = e.key.toUpperCase();
  let note: string | undefined;
  if (e.shiftKey) {
    note = shiftKeyToBlack.get(key);
  }
  if (!note) note = keyToWhite.get(key);
  if (note) {
    e.preventDefault();
    pressedByKey.set(key, note);
    pressNoteFromKeyboard(note);
  }
}

function onKeyUp(e: KeyboardEvent) {
  const key = e.key.toUpperCase();
  const note = pressedByKey.get(key);
  if (note) {
    pressedByKey.delete(key);
    releaseNote(note);
  }
}

// 窗口失焦兜底：清空所有按下的键，避免 keyup 丢失导致键一直亮
function clearAllNotes() {
  activeKeys.value.clear();
  pressedByKey.clear();
}

const whiteNotes = WHITE_NOTES.map(n => n.note);

function isActive(note: string): boolean {
  return activeKeys.value.has(note);
}

function blackHint(note: string): string {
  const idx = BLACK_LEFT_WHITE_INDEX[note];
  return idx !== undefined ? WHITE_NOTES[idx].key : '';
}

// ─── 键位参考图数据 ───

interface RefKey {
  key: string;
  note: string;
  black?: boolean;
  isDo?: boolean;
}

// do（C 音）标记：只标注最常用的中央 C（C4，键盘 T 键）
function isCentralDo(note: string): boolean {
  return note === 'C4';
}

// 唱名映射（白键 C D E F G A B → do re mi fa sol la si）
const SOLFEGE: Record<string, string> = {
  C: 'do', D: 're', E: 'mi', F: 'fa', G: 'sol', A: 'la', B: 'si',
};

function solfege(note: string): string {
  return SOLFEGE[note[0]] ?? '';
}

const refRows: RefKey[][] = [
  WHITE_NOTES.slice(0, 10).map(n => ({ key: n.key, note: n.note, isDo: isCentralDo(n.note) })),
  WHITE_NOTES.slice(10, 20).map(n => ({ key: n.key, note: n.note, isDo: isCentralDo(n.note) })),
  WHITE_NOTES.slice(20, 29).map(n => ({ key: n.key, note: n.note, isDo: isCentralDo(n.note) })),
  WHITE_NOTES.slice(29).map(n => ({ key: n.key, note: n.note, isDo: isCentralDo(n.note) })),
];

// 黑键参考：每行对应八度的黑键（Shift + 白键）
const blackRefRows: RefKey[][] = [
  ['C#2', 'D#2', 'F#2', 'G#2', 'A#2'].map(note => ({
    key: '⇧' + blackHint(note),
    note,
    black: true,
  })),
  ['C#3', 'D#3', 'F#3', 'G#3', 'A#3'].map(note => ({
    key: '⇧' + blackHint(note),
    note,
    black: true,
  })),
  ['C#4', 'D#4', 'F#4', 'G#4', 'A#4'].map(note => ({
    key: '⇧' + blackHint(note),
    note,
    black: true,
  })),
  ['C#5', 'D#5', 'F#5', 'G#5', 'A#5'].map(note => ({
    key: '⇧' + blackHint(note),
    note,
    black: true,
  })),
  ['C#6', 'D#6', 'F#6', 'G#6', 'A#6'].map(note => ({
    key: '⇧' + blackHint(note),
    note,
    black: true,
  })),
];

onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  window.addEventListener('blur', clearAllNotes);
  loadSamples();
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('keyup', onKeyUp);
  window.removeEventListener('blur', clearAllNotes);
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-[#1a1a2e] to-[#16213e] p-4 md:p-8 flex flex-col items-center">
    <h1 class="text-2xl md:text-3xl font-bold text-white mb-1">{{ t('piano.title') }}</h1>
    <p class="text-white/60 text-sm mb-3">{{ t('piano.subtitle') }}</p>

    <!-- 键位参考图 -->
    <div class="w-full max-w-6xl bg-black/40 rounded-xl p-4 mb-4 border border-white/10">
      <div class="text-white/70 text-xs mb-2 flex items-center gap-2">
        <span>{{ t('piano.keyRefTitle') }}</span>
        <span class="text-amber-400">⇧ = Shift</span>
      </div>

      <!-- 白键参考行 -->
      <div class="space-y-1.5">
        <div v-for="(row, ri) in refRows" :key="ri" class="flex gap-1">
          <div
            v-for="k in row"
            :key="k.key"
            class="flex-1 min-w-0 rounded px-0.5 py-1 text-center cursor-default border"
            :class="k.isDo
              ? 'bg-gradient-to-b from-emerald-300 to-emerald-500 border-emerald-600'
              : 'bg-gradient-to-b from-white to-gray-300 border-gray-400'"
          >
            <div class="text-[11px] md:text-sm font-bold leading-tight" :class="k.isDo ? 'text-emerald-950' : 'text-gray-700'">{{ k.key }}</div>
            <div class="text-[9px] md:text-[11px] leading-tight" :class="k.isDo ? 'text-emerald-900 font-bold' : 'text-blue-600'">{{ k.isDo ? k.note + ' do' : k.note }}</div>
          </div>
        </div>
        <!-- 黑键参考行 -->
        <div v-for="(row, ri) in blackRefRows" :key="'b' + ri" class="flex gap-1">
          <div
            v-for="k in row"
            :key="k.note"
            class="flex-1 min-w-0 rounded bg-gradient-to-b from-[#444] to-black border border-black px-0.5 py-1 text-center cursor-default"
          >
            <div class="text-[10px] md:text-sm font-bold text-white leading-tight">{{ k.key }}</div>
            <div class="text-[9px] md:text-[11px] text-amber-400 leading-tight">{{ k.note }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="mb-4 text-sm">
      <span v-if="!ready" class="text-white/70">
        {{ t('piano.loading', { current: loading, total: totalSamples }) }}
      </span>
      <span v-else class="text-emerald-400">{{ t('piano.ready') }}</span>
      <span v-if="errorMsg" class="text-red-400 ml-3">{{ errorMsg }}</span>
    </div>

    <!-- 钢琴（AutoPiano 布局：36 白键 + 5 组黑键） -->
    <div class="w-full max-w-6xl select-none rounded-b-xl overflow-hidden shadow-2xl ring-1 ring-black/60" @mousedown.prevent>
      <div class="relative h-52 md:h-72 bg-black">
        <!-- 白键 -->
        <div class="flex h-full">
          <div
            v-for="note in whiteNotes"
            :key="note"
            class="flex-1 relative border border-gray-300 rounded-b-md transition-all duration-75 cursor-pointer"
            :class="isActive(note)
              ? 'bg-gradient-to-b from-amber-200 to-amber-400 shadow-inner'
              : 'bg-gradient-to-b from-white via-white to-gray-300 hover:from-gray-100 shadow-[inset_0_-6px_8px_rgba(0,0,0,0.12)]'"
            @pointerdown="pressNote(note)"
            @pointerup="releaseNote(note)"
            @pointerleave="releaseNote(note)"
            @contextmenu.prevent
          >
            <!-- do 标记：只标注中央 C（C4，T 键） -->
            <span v-if="isCentralDo(note)"
              class="absolute top-1 left-1/2 -translate-x-1/2 flex items-center justify-center gap-0.5 px-1.5 py-0.5 rounded-full bg-emerald-500 text-white text-[9px] md:text-[11px] font-bold select-none shadow">
              do
            </span>
            <!-- 白键标注：灰色键名 + 唱名 + 音名（C4 记法） -->
            <span class="absolute bottom-1 left-0 right-0 text-center leading-tight select-none">
              <span class="block text-[10px] md:text-xs font-semibold text-gray-500">
                {{ WHITE_NOTES.find(n => n.note === note)?.key }}
              </span>
              <span class="block text-[9px] md:text-[11px] text-gray-400">{{ solfege(note) }}</span>
              <span class="block text-[9px] md:text-[11px] text-gray-500">{{ note }}</span>
            </span>
          </div>
        </div>

        <!-- 黑键（每组一个八度，组内按 AutoPiano 比例定位） -->
        <div
          v-for="oct in BLACK_OCTAVES"
          :key="oct.left"
          class="absolute top-0 h-full pointer-events-none"
          :style="{ left: oct.left, width: '20%' }"
        >
          <div
            v-for="(note, i) in oct.keys"
            :key="note"
            class="absolute top-0 rounded-b-md transition-all duration-75 cursor-pointer pointer-events-auto"
            :style="{ left: BLACK_IN_GROUP_LEFT[i], width: '10%', height: '70%' }"
            :class="isActive(note)
              ? 'bg-gradient-to-b from-amber-500 to-amber-800 shadow-[0_0_14px_rgba(245,158,11,0.7)]'
              : 'bg-gradient-to-b from-[#3a3a3a] via-black to-[#1a1a1a] border-x border-black/70 border-b-4 border-b-[#111] shadow-[inset_0_-3px_5px_rgba(255,255,255,0.12),0_3px_6px_rgba(0,0,0,0.7)]'"
            @pointerdown="pressNote(note)"
            @pointerup="releaseNote(note)"
            @pointerleave="releaseNote(note)"
          >
            <span class="absolute bottom-1 left-0 right-0 text-center text-[9px] md:text-[11px] text-white/70 select-none">
              ⇧{{ blackHint(note) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div class="mt-4 text-white/40 text-xs">
      {{ t('piano.keyHint') }}
    </div>
    <div class="mt-1 text-emerald-400/80 text-xs">
      {{ t('piano.doHint') }}
    </div>
  </div>
</template>
