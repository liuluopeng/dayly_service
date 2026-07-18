<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import {
  game2048_init, game2048_board, game2048_score, game2048_max_tile,
  game2048_won, game2048_over, game2048_move, game2048_undo,
} from "../types/wasm-typed";

const { t } = useI18n();

const board = ref<number[]>(Array(16).fill(0));
const score = ref(0);
const best = ref(0);
const won = ref(false);
const over = ref(false);

const tileColors: Record<number, string> = {
  0: "#cdc1b4", 2: "#eee4da", 4: "#ede0c8", 8: "#f2b179",
  16: "#f59563", 32: "#f67c5f", 64: "#f65e3b", 128: "#edcf72",
  256: "#edcc61", 512: "#edc850", 1024: "#edc53f", 2048: "#edc22e",
};

const tileTextColor = (v: number) => v <= 4 ? "#776e65" : "#f9f6f2";
const tileFontSize = (v: number) => v >= 1000 ? "1.2rem" : v >= 100 ? "1.5rem" : "1.8rem";

let touchStartX = 0;
let touchStartY = 0;

function refresh() {
  board.value = game2048_board();
  score.value = game2048_score();
  won.value = game2048_won();
  over.value = game2048_over();
  best.value = Math.max(best.value, score.value);
}

function move(dir: string) {
  if (over.value || won.value) return;
  if (game2048_move(dir)) refresh();
}

function undo() {
  if (game2048_undo()) refresh();
}

function newGame() {
  game2048_init();
  refresh();
}

function onKeydown(e: KeyboardEvent) {
  const map: Record<string, string> = {
    ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right",
    w: "up", s: "down", a: "left", d: "right",
  };
  const dir = map[e.key];
  if (dir) {
    e.preventDefault();
    move(dir);
  }
  if (e.key === "z" && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    undo();
  }
}

function onTouchStart(e: TouchEvent) {
  touchStartX = e.touches[0].clientX;
  touchStartY = e.touches[0].clientY;
}

function onTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].clientX - touchStartX;
  const dy = e.changedTouches[0].clientY - touchStartY;
  const absDx = Math.abs(dx);
  const absDy = Math.abs(dy);
  if (Math.max(absDx, absDy) < 30) return;
  if (absDx > absDy) {
    move(dx > 0 ? "right" : "left");
  } else {
    move(dy > 0 ? "down" : "up");
  }
}

onMounted(() => {
  best.value = parseInt(localStorage.getItem("best2048") || "0");
  newGame();
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  localStorage.setItem("best2048", best.value.toString());
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="flex flex-col items-center p-4 select-none" @touchstart="onTouchStart" @touchend="onTouchEnd">
    <h1 class="text-3xl font-bold mb-2">2048</h1>

    <div class="flex gap-4 mb-4 text-sm">
      <div class="bg-gray-700 text-white px-3 py-1 rounded text-center">
        <div class="text-xs opacity-70">{{ t('menu.game2048.score') }}</div>
        <div class="font-bold text-lg">{{ score }}</div>
      </div>
      <div class="bg-gray-700 text-white px-3 py-1 rounded text-center">
        <div class="text-xs opacity-70">{{ t('menu.game2048.best') }}</div>
        <div class="font-bold text-lg">{{ best }}</div>
      </div>
    </div>

    <div class="relative">
      <div class="bg-gray-600 rounded-lg p-2" style="width: min(85vw, 400px); height: min(85vw, 400px);">
        <div class="grid grid-cols-4 gap-2 w-full h-full">
          <div v-for="(v, i) in board" :key="i"
               class="rounded flex items-center justify-center font-bold transition-all duration-100"
               :style="{
                 backgroundColor: tileColors[v] || '#3c3a32',
                 color: tileTextColor(v),
                 fontSize: tileFontSize(v),
               }">
            {{ v || '' }}
          </div>
        </div>
      </div>

      <div v-if="over"
           class="absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center">
        <div class="text-center">
          <p class="text-white text-2xl font-bold mb-2">{{ t('menu.game2048.over') }}</p>
          <button @click="newGame" class="px-4 py-2 bg-white text-gray-800 rounded font-bold">
            {{ t('menu.game2048.retry') }}
          </button>
        </div>
      </div>

      <div v-if="won && !over"
           class="absolute inset-0 bg-yellow-500/80 rounded-lg flex items-center justify-center">
        <div class="text-center">
          <p class="text-white text-2xl font-bold mb-2">{{ t('menu.game2048.won') }}</p>
          <button @click="newGame" class="px-4 py-2 bg-white text-gray-800 rounded font-bold">
            {{ t('menu.game2048.retry') }}
          </button>
        </div>
      </div>
    </div>

    <div class="flex gap-3 mt-4">
      <button @click="undo"
              :disabled="!score"
              class="px-4 py-2 bg-gray-500 text-white rounded text-sm disabled:opacity-30">
        ↩ {{ t('menu.game2048.undo') }}
      </button>
      <button @click="newGame"
              class="px-4 py-2 bg-blue-500 text-white rounded text-sm">
        {{ t('menu.game2048.newGame') }}
      </button>
    </div>

    <p class="mt-3 text-xs text-gray-400">{{ t('menu.game2048.hint') }}</p>
  </div>
</template>
