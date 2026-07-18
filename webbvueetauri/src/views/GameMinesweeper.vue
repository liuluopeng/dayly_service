<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { ms_init, ms_click, ms_toggle_flag, ms_cells, ms_revealed, ms_flagged, ms_over, ms_won, ms_flag_count } from "../types/wasm-typed";

const { t } = useI18n();
const cells = ref<number[]>([]);
const revealed = ref<number[]>([]);
const flagged = ref<number[]>([]);
const over = ref(false);
const won = ref(false);
const flags = ref(0);
const flagMode = ref(false);

function refresh() {
  cells.value = ms_cells(); revealed.value = ms_revealed(); flagged.value = ms_flagged();
  over.value = ms_over(); won.value = ms_won(); flags.value = ms_flag_count();
}

function onClick(x: number, y: number) { if (over.value || won.value) return; if (flagMode.value) ms_toggle_flag(x, y); else ms_click(x, y); refresh(); }
function newGame() { ms_init(); refresh(); }

const cellColor = (v: number) => v === 9 ? "#ff4444" : v === 0 ? "#aaa" : ["#0000ff","#008000","#ff0000","#000080","#800000","#008080","#000000","#808080"][v - 1] || "#000";

onMounted(() => { ms_init(); refresh(); });
</script>

<template>
  <div class="flex flex-col items-center p-4 select-none">
    <h1 class="text-2xl font-bold mb-1">{{ t('menu.minesweeper.title') }}</h1>
    <div class="flex gap-3 mb-2 text-sm items-center">
      <span>{{ t('menu.game2048.score') }} 💣{{ flags }}</span>
      <button @click="flagMode = !flagMode" class="px-2 py-0.5 rounded text-xs" :class="flagMode ? 'bg-red-500 text-white' : 'bg-gray-200'">🚩 {{ t('menu.minesweeper.flag') }}</button>
      <button @click="newGame" class="px-2 py-0.5 bg-blue-500 text-white rounded text-xs">{{ t('menu.game2048.newGame') }}</button>
    </div>
    <div v-if="over" class="text-red-500 font-bold mb-1">{{ t('menu.game2048.over') }}</div>
    <div v-if="won" class="text-green-500 font-bold mb-1">{{ t('menu.game2048.won') }}</div>
    <div class="bg-gray-300 p-px rounded" style="width:270px;height:270px">
      <div v-if="cells.length" class="grid gap-px" style="grid-template:repeat(9,1fr)/repeat(9,1fr);width:100%;height:100%">
        <div v-for="(c,i) in cells" :key="i" @click="onClick(i%9, Math.floor(i/9))"
             :style="{background:revealed[i]? '#e0e0e0' : flagged[i] ? '#ffd700' : '#b0b0b0', cursor:'pointer', display:'flex', alignItems:'center', justifyContent:'center', fontSize:'11px', fontWeight:'bold', color:cellColor(c)}">
          {{ revealed[i] && c > 0 ? c : flagged[i] ? '🚩' : '' }}
        </div>
      </div>
    </div>
  </div>
</template>
