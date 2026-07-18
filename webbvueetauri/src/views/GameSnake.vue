<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { snake_init, snake_tick, snake_set_dir, snake_board, snake_score, snake_over } from "../types/wasm-typed";

const { t } = useI18n();
const board = ref<number[]>([]);
const score = ref(0);
const over = ref(false);
let timer: number | null = null;
let tx = 0, ty = 0;

function refresh() { board.value = snake_board(); score.value = snake_score(); over.value = snake_over(); }
function tick() { if (over.value) return; snake_tick(); refresh(); }

function setDir(d: string) { snake_set_dir(d); }

function onKeydown(e: KeyboardEvent) {
  const m: Record<string, string> = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right", w: "up", s: "down", a: "left", d: "right" };
  if (m[e.key]) { e.preventDefault(); setDir(m[e.key]); }
}

function onTouchStart(e: TouchEvent) { tx = e.touches[0].clientX; ty = e.touches[0].clientY; }
function onTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].clientX - tx, dy = e.changedTouches[0].clientY - ty;
  if (Math.max(Math.abs(dx), Math.abs(dy)) < 30) return;
  setDir(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? "right" : "left") : (dy > 0 ? "down" : "up"));
}

function start() { snake_init(); refresh(); over.value = false; timer = window.setInterval(tick, 150); }
function stop() { if (timer) { clearInterval(timer); timer = null; } }

onMounted(() => { window.addEventListener("keydown", onKeydown); start(); });
onUnmounted(() => { stop(); window.removeEventListener("keydown", onKeydown); });
</script>

<template>
  <div class="flex flex-col items-center p-4 select-none" @touchstart="onTouchStart" @touchend="onTouchEnd">
    <h1 class="text-2xl font-bold mb-1">{{ t('menu.snake.title') }}</h1>
    <div class="text-sm mb-1">{{ t('menu.game2048.score') }}: {{ score }}</div>
    <div v-if="over" class="text-red-500 font-bold mb-1">{{ t('menu.game2048.over') }}</div>
    <div class="bg-gray-900 p-px rounded" style="width:304px;height:304px">
      <div v-if="board.length" class="grid gap-px" style="grid-template:repeat(20,1fr)/repeat(20,1fr);width:100%;height:100%">
        <div v-for="(c,i) in board" :key="i" :style="{background:c===2?'#f44':c===1?'#4f4':'#16213e'}"></div>
      </div>
    </div>
    <div class="flex gap-2 mt-2"><button @click="start" class="px-3 py-1 bg-blue-500 text-white rounded text-sm">{{ t('menu.game2048.newGame') }}</button></div>
    <p class="mt-2 text-xs text-gray-400">{{ t('menu.game2048.hint') }}</p>
  </div>
</template>
