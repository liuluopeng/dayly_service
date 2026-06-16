<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  search_notes_wasm,
} from "../types/wasm-typed";

const { t, locale } = useI18n();

const searchQuery = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const searchResults = ref<any[]>([]);
const isLoading = ref(false);
const error = ref('');
const wasmInitialized = ref(false);

// 初始化wasm
onMounted(async () => {
  try {
    wasmInitialized.value = true;
    // 自动对焦到搜索框
    setTimeout(() => {
      searchInput.value?.focus();
    }, 100);
  } catch (err) {
    console.error('初始化wasm失败:', err);
    error.value = t('noteSearch.initFailed');
  }
});

const searchNotes = async () => {
  if (!searchQuery.value.trim() || !wasmInitialized.value) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;
  error.value = '';

  try {
    const results = await search_notes_wasm(searchQuery.value);
    searchResults.value = results as any[];
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('noteSearch.searchFailed');
    console.error('搜索笔记失败:', err);
  } finally {
    isLoading.value = false;
  }
};

// 处理键盘回车事件
// const handleKeyPress = (event: KeyboardEvent) => {
//   if (event.key === 'Enter') {
//     searchNotes();
//   }
// };
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold mb-6 text-gray-800">{{ t('noteSearch.title') }}</h1>

    <!-- WASM初始化加载 -->
    <div v-if="!wasmInitialized" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      <p class="mt-2 text-gray-600">{{ t('noteSearch.initializing') }}</p>
    </div>

    <div v-else>
      <div class="mb-6">
        <div class="flex">
          <input ref="searchInput" v-model="searchQuery" @keyup.enter="searchNotes" type="text" :placeholder="t('noteSearch.placeholder')"
            class="flex-1 px-4 py-2 border border-gray-300 rounded-l-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
          <button @click="searchNotes" :disabled="isLoading"
            class="px-6 py-2 bg-blue-600 text-white rounded-r-md hover:bg-blue-700 disabled:bg-gray-400 transition-colors">
            <span v-if="!isLoading">{{ t('noteSearch.search') }}</span>
            <span v-else>{{ t('noteSearch.searching') }}</span>
          </button>
        </div>
      </div>

      <div v-if="error" class="mb-4 p-4 bg-red-100 text-red-700 rounded-md">
        {{ error }}
      </div>

      <div v-if="searchResults.length > 0" class="space-y-4">
        <div v-for="(note, index) in searchResults" :key="note.id"
          class="bg-white p-4 rounded-md shadow-sm border border-gray-200 hover:shadow-md transition-shadow">
          <div class="flex justify-between items-start mb-2">
            <h3 class="font-medium text-gray-800">{{ note.filename || t('noteSearch.noteIndex', { index: index + 1 }) }}</h3>
            <span class="text-sm text-gray-500">{{ new Date(note.created_at).toLocaleString(locale === 'zh' ? 'zh-CN' : 'en-US') }}</span>
          </div>
          <div class="text-gray-600 text-sm mb-2">
            <p v-if="note.simple_text" class="line-clamp-3">{{ note.simple_text }}</p>
            <p v-else-if="note.text" class="line-clamp-3">{{ note.text }}</p>
            <p v-else class="text-gray-400">{{ t('common.noContent') }}</p>
          </div>
          <div class="flex justify-end">
            <router-link :to="`/note/${note.id}`" class="text-blue-600 hover:text-blue-800 text-sm">
              {{ t('noteSearch.viewDetail') }}
            </router-link>
          </div>
        </div>
      </div>

      <div v-else-if="searchQuery && !isLoading" class="text-center py-12 text-gray-500">
        {{ t('noteSearch.noResults') }}
      </div>

      <div v-else-if="!searchQuery && !isLoading" class="text-center py-12 text-gray-500">
        {{ t('noteSearch.enterKeyword') }}
      </div>
    </div>
  </div>
</template>
