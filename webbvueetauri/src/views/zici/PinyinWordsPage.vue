<script setup>
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const pinyin = computed(() => route.query.pinyin || '');

const generatedWords = ref([]);
const isLoadingWords = ref(false);
const wordsScrollRef = ref(null);

const goBack = () => {
  router.back();
};

const navigateToDetail = (word) => {
  router.push({
    path: '/zici/word-detail',
    query: { word: word, pinyin: pinyin.value }
  });
};

const generateWords = async () => {
  if (!pinyin.value) return;
  isLoadingWords.value = true;
  try {
    const apiUrl = import.meta.env.VITE_API_BASE_URL || '';
    const token = localStorage.getItem('token') || '';
    const response = await fetch(`${apiUrl}/api/pinyin/dict?ori=${encodeURIComponent(pinyin.value)}`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {}
    });
    if (!response.ok) {
      throw new Error(`加载数据失败: ${response.status}`);
    }
    const wordsData = await response.json();
    if (!wordsData || (Array.isArray(wordsData) && wordsData.length === 0)) {
      generatedWords.value = [];
      return;
    }

    const processedWords = Array.isArray(wordsData) ? wordsData.map(wordObj => {
      const updatedExplanation = wordObj.explanation ? wordObj.explanation.replace(/href="hycd\.css"/g, 'href="cidian.css"') : '';
      return {
        word: wordObj.word,
        pinyin: wordObj.pinyin_array ? wordObj.pinyin_array.join(' ') : '',
        hasExplanation: true,
        explanation: updatedExplanation,
        frequency: wordObj.frequency || 0
      };
    }) : [];

    processedWords.sort((a, b) => a.frequency - b.frequency);
    generatedWords.value = processedWords;

    if (wordsScrollRef.value) {
      wordsScrollRef.value.scrollTo({ top: 0, behavior: 'smooth' });
    }
  } catch (error) {
    console.error('词语生成失败:', error);
    generatedWords.value = [];
  } finally {
    isLoadingWords.value = false;
  }
};

onMounted(() => {
  generateWords();
});
</script>

<template>
  <div class="w-full min-h-full p-[0.8rem] max-w-full m-0 flex flex-col overflow-hidden bg-[#f5f5f5]">
    <div class="flex gap-4 flex-1 min-h-0 overflow-hidden">
      <div class="flex flex-col bg-[#f8f9fa] rounded-2xl shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1),inset_0_0_0_1px_rgba(255,255,255,0.8)] overflow-hidden relative flex-1">
        <div class="bg-gradient-to-br from-[#4caf50] to-[#45a049] shrink-0 select-none flex items-center py-[0.6rem] px-4 text-center text-base font-bold text-white">
          <button class="bg-white/20 border-none text-white py-[0.4rem] px-[0.8rem] rounded-[0.4rem] cursor-pointer text-[0.9rem] transition-all duration-200 mr-4 hover:bg-white/30 hover:scale-105" @click="goBack">← {{ t('common.back') }}</button>
          <span class="flex-1 text-center">{{ t('pinyinWordsPage.listTitle', { pinyin: pinyin, count: generatedWords.length }) }}</span>
        </div>
        <div class="flex-1 flex flex-col items-center p-6 gap-[0.8rem] min-h-0 w-full">
          <div class="w-full flex-1 flex flex-col min-h-0">
            <div class="grid grid-cols-[repeat(auto-fill,minmax(110px,1fr))] gap-[0.8rem] flex-1 overflow-y-auto p-[0.8rem] bg-[#f8f9fa] rounded-lg min-h-0 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden" ref="wordsScrollRef">
              <div v-for="(item, index) in generatedWords" :key="index"
                class="flex flex-col items-center p-[0.8rem] bg-white rounded-lg shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)] transition-all duration-300 text-center cursor-pointer relative select-none min-h-[80px] justify-center touch-manipulation [-webkit-tap-highlight-color:transparent] hover:-translate-y-0.5 hover:shadow-[0_0.4rem_0.8rem_rgba(0,0,0,0.15)] hover:bg-[#f8f9ff]"
                @click="navigateToDetail(item.word)">
                <div v-if="item.hasExplanation" class="absolute top-[0.4rem] right-[0.4rem] w-2 h-2 bg-[#4caf50] rounded-full shadow-[0_0_4px_rgba(76,175,80,0.5)] animate-[dotPulse_2s_infinite]"></div>
                <div class="text-[0.8rem] text-[#666] mb-[0.4rem]">{{ item.pinyin }}</div>
                <div class="text-[1.1rem] font-bold text-[#333]">{{ item.word }}</div>
                <div class="absolute bottom-[0.2rem] right-[0.2rem] text-[0.6rem] text-[#999] font-['Arial',sans-serif]" v-if="item.frequency">{{ (1 - item.frequency / 56000).toFixed(6) }}</div>
              </div>
              <div v-if="generatedWords.length === 0 && !isLoadingWords" class="text-center text-[#999] p-6 italic col-span-full">
                {{ t('pinyinWordsPage.noRelatedWords') }}
              </div>
              <div v-if="isLoadingWords" class="text-center text-[#666] p-6 col-span-full flex items-center justify-center gap-2 [&::before]:content-[''] [&::before]:inline-block [&::before]:w-[14px] [&::before]:h-[14px] [&::before]:border-2 [&::before]:border-[#4caf50] [&::before]:border-t-transparent [&::before]:rounded-full [&::before]:animate-spin">
                {{ t('pinyinWordsPage.loadingData') }}
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
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.8;
  }
}
</style>
