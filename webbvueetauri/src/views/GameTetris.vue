<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { tetris_init, tetris_tick, tetris_move, tetris_board, tetris_score, tetris_over } from "../types/wasm-typed";

const { t } = useI18n();
const board = ref<number[]>([]);
const score = ref(0);
const over = ref(false);
let timer: number | null = null;

function refresh() { board.value = tetris_board(); score.value = tetris_score(); over.value = tetris_over(); }
function tick() { if (over.value) return; tetris_tick(); refresh(); }
function move(dir: string) { tetris_move(dir); refresh(); }

function onKeydown(e: KeyboardEvent) {
  const m: Record<string, string> = { ArrowLeft: "left", ArrowRight: "right", ArrowDown: "down", ArrowUp: "rotate", " ": "drop", z: "rotate" };
  if (m[e.key]) { e.preventDefault(); move(m[e.key]); }
}

const colors = ["", "#0ff", "#ff0", "#a0f", "#f80", "#08f", "#0f0", "#f00"];

function start() { tetris_init(); refresh(); timer = window.setInterval(tick, 400); }
function stop() { if (timer) { clearInterval(timer); timer = null; } }

onMounted(() => { window.addEventListener("keydown", onKeydown); start(); });
onUnmounted(() => { stop(); window.removeEventListener("keydown", onKeydown); });
</script>

<template>
  <div class="flex flex-col items-center p-4 select-none">
    <h1 class="text-2xl font-bold mb-1">{{ t('menu.tetris.title') }}</h1>
    <div class="text-sm mb-1">{{ t('menu.game2048.score') }}: {{ score }}</div>
    <div v-if="over" class="text-red-500 font-bold mb-1">{{ t('menu.game2048.over') }}</div>
    <div class="bg-gray-900 p-px rounded" style="width:200px;height:400px">
      <div v-if="board.length" class="grid gap-px" style="grid-template:repeat(20,1fr)/repeat(10,1fr);width:100%;height:100%">
        <div v-for="(c,i) in board" :key="i"
             :style="{background:colors[c]||'#1a1a2e', borderRadius:'2px'}"></div>
      </div>
    </div>
    <p class="mt-2 text-xs text-gray-400">{{ t('menu.game2048.hint') }} · Space: {{ t('menu.tetris.drop') }}</p>
    <button @click="start" class="mt-2 px-3 py-1 bg-blue-500 text-white rounded text-sm">{{ t('menu.game2048.newGame') }}</button>
  </div>
</template>
