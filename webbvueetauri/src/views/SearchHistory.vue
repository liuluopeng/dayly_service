<template>
  <div class="p-5 max-w-[800px] mx-auto">
    <h1 class="text-center text-[#333] mb-[30px]">{{ t('searchHistory.pageTitle') }}</h1>

    <div class="flex mb-5 border-b border-[#ddd]">
      <button :class="activeTab === 'history' ? 'px-5 py-2.5 bg-transparent border-none cursor-pointer text-base text-[#42b983] border-b-2 border-b-[#42b983] transition-all duration-300' : 'px-5 py-2.5 bg-transparent border-none cursor-pointer text-base text-[#666] border-b-2 border-transparent transition-all duration-300'" @click="activeTab = 'history'">
        {{ t('searchHistory.historyTab') }}
      </button>
      <button :class="activeTab === 'frequency' ? 'px-5 py-2.5 bg-transparent border-none cursor-pointer text-base text-[#42b983] border-b-2 border-b-[#42b983] transition-all duration-300' : 'px-5 py-2.5 bg-transparent border-none cursor-pointer text-base text-[#666] border-b-2 border-transparent transition-all duration-300'" @click="activeTab = 'frequency'">
        {{ t('searchHistory.frequencyTab') }}
      </button>
    </div>

    <div v-if="loading" class="flex flex-col items-center justify-center py-[60px]">
      <div class="w-10 h-10 mb-4 rounded-full border-4 border-[rgba(0,0,0,0.1)] border-t-[#42b983] animate-spin"></div>
      <p>{{ t('searchHistory.loading') }}</p>
    </div>

    <div v-else>
      <div v-if="activeTab === 'history'">
        <div class="mb-5">
          <div v-for="(item, index) in recentHistory" :key="index" class="flex justify-between items-center p-2.5 border-b border-[#f0f0f0]">
            <div class="flex-1">
              <span class="font-medium mr-2.5">{{ item.word }}</span>
              <span class="text-xs text-[#999]">{{ formatTime(item.time) }}</span>
            </div>
          </div>
          <div v-if="recentHistory.length === 0" class="text-center text-[#999] py-10">
            {{ t('searchHistory.noHistory') }}
          </div>
        </div>
      </div>

      <div v-else>
        <div class="mt-5">
          <div v-for="(item, index) in topWords" :key="index" class="flex items-center mb-2.5">
            <span class="w-[100px] font-medium">{{ item.word }}</span>
            <div class="flex-1 h-5 mx-2.5 bg-[#f0f0f0] rounded-[10px] overflow-hidden">
              <div class="h-full bg-[#42b983] rounded-[10px] transition-[width] duration-500 ease-[ease]" :style="{ width: `${(item.hasSearchedTimes / maxFrequency) * 100}%` }"></div>
            </div>
            <span class="w-20 text-right text-sm text-[#666]">{{ t('searchHistory.times', { count: item.hasSearchedTimes }) }}</span>
          </div>
          <div v-if="topWords.length === 0" class="text-center text-[#999] py-10">
            {{ t('searchHistory.noFrequency') }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { get_recent_history, get_top_words } from '../types/wasm-typed';
import type { WordHistory, Word } from '../types/models';

const { t, locale } = useI18n();

const activeTab = ref('history');
const recentHistory = ref<WordHistory[]>([]);
const topWords = ref<Word[]>([]);
const loading = ref(true);

// 从API加载最近历史
const loadRecentHistory = async () => {
  try {
    const result = await get_recent_history(BigInt(10));
    if (result && result.data) {
      recentHistory.value = result.data;
    }
  } catch (error) {
    console.error('Failed to load recent history:', error);
  }
};

// 从API加载高频词
const loadTopWords = async () => {
  try {
    const result = await get_top_words();
    if (result && result.data) {
      topWords.value = result.data;
    }
  } catch (error) {
    console.error('Failed to load top words:', error);
  }
};

const maxFrequency = computed(() => {
  if (topWords.value.length === 0) return 1;
  return Math.max(...topWords.value.map(item => item.hasSearchedTimes));
});

const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US');
};

onMounted(async () => {
  loading.value = true;
  await Promise.all([loadRecentHistory(), loadTopWords()]);
  loading.value = false;
});
</script>
