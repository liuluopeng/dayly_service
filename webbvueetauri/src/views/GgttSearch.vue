<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from 'vue-i18n';
import { search_ggtt } from "../types/wasm-typed";
import { useSearchHistory } from "../composables/useSearchHistory";
import type { GgttResult } from "../types/models";

const { t } = useI18n();
const { addSearchHistory } = useSearchHistory();

const ggttSearchCode = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

const ggttSearchResult = ref<string | GgttResult>("");
const ggttSearchLoading = ref(false);
const ggttSearchError = ref("");

onMounted(() => {
  setTimeout(() => {
    searchInput.value?.focus();
  }, 100);
});

async function testGgttSearch() {
  ggttSearchLoading.value = true;
  ggttSearchError.value = "";
  try {
    const term = ggttSearchCode.value;
    addSearchHistory(term);
    const result = await search_ggtt(term);
    ggttSearchResult.value = result;
    console.log('返回结果:', result);
    console.log('结果类型:', typeof result);
    console.log('是否为对象:', result && typeof result === 'object');
    console.log('是否为字符串:', typeof result === 'string');
    console.log('是否为null:', result === null);
    console.log('是否为undefined:', result === undefined);

    if (result && typeof result === 'object') {
      console.log('返回对象:', result);
      console.log('对象键:', Object.keys(result));
      console.log('字符:', result.char);
      console.log('GGTT代码:', result.code_86);
      console.log('是否有图解:', result.has_diagram);
    } else if (typeof result === 'string') {
      console.log('返回字符串:', result);
      try {
        const parsed = JSON.parse(result);
        console.log('解析后的对象:', parsed);
        console.log('解析后是否为对象:', typeof parsed === 'object');
      } catch (e) {
        console.log('字符串不是JSON:', e);
      }
    }
  } catch (error) {
    console.error("GGTT search failed:", error);
    ggttSearchError.value = error instanceof Error ? error.message : t('common.error.loadFailed');
  } finally {
    ggttSearchLoading.value = false;
  }
}

function sanitizeSvg(svg: string) {
  return svg
    .replace(/<script[^>]*>.*?<\/script>/gi, '')
    .replace(/on\w+\s*=\s*["'].*?["']/gi, '')
    .replace(/`/g, '');
}

</script>

<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold text-center text-blue-600 mb-8">{{ t('ggttSearch.title') }}</h1>

    <div class="bg-white p-6 rounded-lg shadow-md">
      <h2 class="text-xl font-semibold mb-4">{{ t('ggttSearch.title') }}</h2>
      <p class="mb-4">使用Rust WASM API查询GGTT编码</p>

      <div class="mb-4">
        <input ref="searchInput" v-model="ggttSearchCode" :placeholder="t('ggttSearch.placeholder')"
          class="px-4 py-2 border border-gray-300 rounded mr-2" @keyup.enter="testGgttSearch" />

        <button @click="testGgttSearch" :disabled="ggttSearchLoading" class="px-4 py-2 bg-blue-500 text-white rounded">
          {{ ggttSearchLoading ? t('common.loading') : t('ggttSearch.search') }}
        </button>
      </div>

      <div v-if="ggttSearchError" class="mb-4 p-3 bg-red-100 text-red-700 rounded">
        {{ ggttSearchError }}
      </div>

      <div v-if="ggttSearchResult" class="mt-4">
        <h3 class="font-semibold mb-2">搜索结果:</h3>
        <pre class="bg-gray-100 p-4 rounded overflow-auto text-sm">{{ ggttSearchResult }}</pre>

        <!-- SVG Display -->
        <div v-if="typeof ggttSearchResult === 'object' && ggttSearchResult.svg1" class="mt-6">
          <h4 class="font-medium mb-3">SVG图解:</h4>
          <div class="grid grid-cols-2 gap-4">
            <div v-if="ggttSearchResult.svg1" class="bg-gray-50 p-2 rounded">
              <h5 class="text-sm text-gray-600 mb-1">SVG 1:</h5>
              <div v-html="sanitizeSvg(ggttSearchResult.svg1)" class="inline-block"></div>
            </div>
            <div v-if="ggttSearchResult.svg2" class="bg-gray-50 p-2 rounded">
              <h5 class="text-sm text-gray-600 mb-1">SVG 2:</h5>
              <div v-html="sanitizeSvg(ggttSearchResult.svg2)" class="inline-block"></div>
            </div>
            <div v-if="ggttSearchResult.svg3" class="bg-gray-50 p-2 rounded">
              <h5 class="text-sm text-gray-600 mb-1">SVG 3:</h5>
              <div v-html="sanitizeSvg(ggttSearchResult.svg3)" class="inline-block"></div>
            </div>
            <div v-if="ggttSearchResult.svg4" class="bg-gray-50 p-2 rounded">
              <h5 class="text-sm text-gray-600 mb-1">SVG 4:</h5>
              <div v-html="sanitizeSvg(ggttSearchResult.svg4)" class="inline-block"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
