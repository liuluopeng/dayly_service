<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from "vue";
import { useI18n } from 'vue-i18n';
import { search_ldoce } from "../types/wasm-typed";

const { t } = useI18n();
const word = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const searchResult = ref<string>("");
const searchLoading = ref(false);
const searchError = ref("");
const iframeRef = ref<HTMLIFrameElement | null>(null);

onMounted(() => {
  setTimeout(() => {
    searchInput.value?.focus();
  }, 100);
});

async function searchDict() {
  searchLoading.value = true;
  searchError.value = "";
  try {
    const result = await search_ldoce(word.value);
    searchResult.value = result;
    console.log('返回结果:', result);
    console.log('结果类型:', typeof result);
    console.log('是否为字符串:', typeof result === 'string');
  } catch (error) {
    console.error("朗文词典搜索失败:", error);
    searchError.value = error instanceof Error ? error.message : t('common.error.loadFailed');
  } finally {
    searchLoading.value = false;
  }
}

function loadIframeContent(content: string) {
  if (iframeRef.value) {
    const iframe = iframeRef.value;

    try {
      const blob = new Blob([content], { type: 'text/html' });
      const url = URL.createObjectURL(blob);
      iframe.src = url;
      console.log('iframe内容加载成功');
    } catch (error) {
      console.error('iframe内容加载失败:', error);
    }
  } else {
    console.error('iframe元素未找到');
  }
}

watch(searchResult, async (newResult) => {
  if (newResult) {
    await nextTick();
    loadIframeContent(newResult);
  }
});

</script>

<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold text-center text-blue-600 mb-8">{{ t('ldoceSearch.title') }}</h1>

    <div class="bg-white p-6 rounded-lg shadow-md">
      <h2 class="text-xl font-semibold mb-4">{{ t('ldoceSearch.title') }}</h2>
      <p class="mb-4">使用Rust WASM API搜索朗文词典</p>

      <div class="mb-4">
        <input ref="searchInput" v-model="word" :placeholder="t('ldoceSearch.placeholder')"
          class="px-4 py-2 border border-gray-300 rounded mr-2" @keyup.enter="searchDict" />
        <button @click="searchDict" :disabled="searchLoading" class="px-4 py-2 bg-blue-500 text-white rounded">
          {{ searchLoading ? t('common.loading') : t('ldoceSearch.search') }}
        </button>
      </div>

      <div v-if="searchError" class="mb-4 p-3 bg-red-100 text-red-700 rounded">
        {{ searchError }}
      </div>

      <div v-if="searchResult" class="mt-4">
        <h3 class="font-semibold mb-2">搜索结果:</h3>
        <div class="bg-gray-100 p-4 rounded overflow-auto">
          <iframe ref="iframeRef" class="w-full h-[600px] border-none"
            sandbox="allow-scripts allow-same-origin"></iframe>
        </div>
      </div>
    </div>
  </div>
</template>
