<script setup lang="ts">
import { shallowRef, onMounted, markRaw } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import * as wasm from '../../types/wasm-typed';

interface WordItem {
  word: string;
  pinyin: string;
  pinyin_flat: string;
  frequency: number;
  hasExplanation: boolean;
}

const { t } = useI18n();
const router = useRouter();
const wordList = shallowRef<WordItem[]>([]);
const isLoading = shallowRef(true);

// Entry layout (56 bytes): 16(word) + 32(pinyin) + 4(freq) + 1(hasExp) + 3(pad)
const ENTRY = 56;

const loadWordFrequencyData = () => {
  isLoading.value = true;
  const [ptr] = wasm.get_words_data() as [number, number];
  const mem = wasm.get_wasm_memory();
  const buf = (mem as any).buffer as ArrayBuffer;
  const count = wasm.get_word_count() as number;
  const td = new TextDecoder();

  const items: WordItem[] = new Array(count);
  for (let i = 0; i < count; i++) {
    const off = i * ENTRY;
    let end = off;
    while (end < off + 16 && new Uint8Array(buf, ptr + end, 1)[0] !== 0) end++;
    const word = td.decode(new Uint8Array(buf, ptr + off, end - off));

    let pe = off + 16;
    while (pe < off + 48 && new Uint8Array(buf, ptr + pe, 1)[0] !== 0) pe++;
    const pinyin = td.decode(new Uint8Array(buf, ptr + off + 16, pe - off - 16));

    const dv = new DataView(buf, ptr + off + 48, 4);
    const freq = dv.getUint32(0, true);
    const hasExp = new Uint8Array(buf, ptr + off + 52, 1)[0] === 1;

    items[i] = markRaw({ word, pinyin, pinyin_flat: pinyin, frequency: freq, hasExplanation: hasExp });
  }
  wordList.value = items;
  isLoading.value = false;
};

onMounted(loadWordFrequencyData);

const navigateToDetail = (item: WordItem) => {
  router.push({
    path: '/zici/word-detail',
    query: { word: item.word, pinyin: item.pinyin_flat }
  });
};
</script>

<template>
  <div class="w-full min-h-full p-[0.8rem] max-w-full m-0 flex flex-col overflow-hidden bg-[#f5f5f5]">
    <div class="flex gap-4 flex-1 min-h-0 overflow-hidden">
      <div class="flex flex-col bg-[#f8f9fa] rounded-2xl shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1),inset_0_0_0_1px_rgba(255,255,255,0.8)] overflow-hidden relative flex-1">
        <div class="p-[0.8rem] text-center text-base font-bold text-white bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] shrink-0 select-none">{{ t('wordFrequencyPage.listTitle', { count: wordList.length }) }}</div>
        <div class="flex-1 flex flex-col items-center p-6 gap-[0.8rem] min-h-0 w-full">
          <div class="w-full flex-1 flex flex-col min-h-0">
            <div class="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-[1.2rem] flex-1 overflow-y-auto p-[1.2rem] bg-[#f8f9fa] rounded-lg min-h-0 [&::-webkit-scrollbar]:hidden [scrollbar-width:none]" ref="wordsScrollRef">
              <div v-for="(item, index) in wordList" :key="index" class="[content-visibility:auto] [contain-intrinsic-size:auto_100px] flex flex-col items-center p-[1.2rem] bg-white rounded-[0.8rem] shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)] transition-all duration-300 text-center cursor-pointer relative select-none min-h-[100px] justify-center [touch-action:manipulation] [-webkit-tap-highlight-color:transparent] hover:-translate-y-0.5 hover:shadow-[0_0.4rem_0.8rem_rgba(0,0,0,0.15)] hover:bg-[#f8f9ff]" @click="navigateToDetail(item)">
                <div v-if="item.hasExplanation" class="absolute top-[0.4rem] right-[0.4rem] w-2 h-2 bg-[#4caf50] rounded-full shadow-[0_0_4px_rgba(76,175,80,0.5)] [animation:dotPulse_2s_infinite]"></div>
                <div class="text-base text-[#666] mb-2">{{ item.pinyin }}</div>
                <div class="text-[1.4rem] font-bold text-[#333]">{{ item.word }}</div>
                <div v-if="item.frequency" class="absolute bottom-[0.2rem] right-[0.2rem] text-[0.6rem] text-[#999] font-['Arial',sans-serif]">{{ (1 - item.frequency / 56000).toFixed(6) }}</div>
              </div>
              <div v-if="wordList.length === 0 && !isLoading" class="text-center text-[#999] p-6 italic col-span-full">
                {{ t('wordFrequencyPage.noData') }}
              </div>
              <div v-if="isLoading" class="text-center text-[#666] p-6 col-span-full flex items-center justify-center gap-2 [&::before]:content-[''] [&::before]:inline-block [&::before]:w-[14px] [&::before]:h-[14px] [&::before]:border-2 [&::before]:border-[#4caf50] [&::before]:border-t-transparent [&::before]:rounded-full [&::before]:[animation:spin_1s_linear_infinite]">
                {{ t('wordFrequencyPage.loadingData') }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
@keyframes dotPulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.8;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
