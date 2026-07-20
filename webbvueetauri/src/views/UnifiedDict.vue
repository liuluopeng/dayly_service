<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { search_xiandaihanyu, search_collins, search_ldoce, get_top_words } from "../types/wasm-typed";
import type { Word } from "../types/models";

const { t } = useI18n();
const inputRef = ref<HTMLInputElement | null>(null);
const topWords = ref<Word[]>([]);
const freqLoading = ref(true);

const maxFrequency = computed(() => {
  if (topWords.value.length === 0) return 1;
  return Math.max(...topWords.value.map(w => w.hasSearchedTimes));
});

onMounted(async () => {
  setTimeout(() => inputRef.value?.focus(), 100);
  try {
    const res = await get_top_words();
    topWords.value = res.data || [];
  } catch {}
  freqLoading.value = false;
});

const query = ref("");
const loading = ref(false);
const dicts = ref<{ key: string; label: string; html: string | null; error: string | null }[]>([
  { key: "xiandaihanyu", label: "现代汉语", html: null, error: null },
  { key: "collins", label: "Collins", html: null, error: null },
  { key: "ldoce", label: "LDOCE", html: null, error: null },
]);
async function searchAll() {
  const q = query.value.trim();
  if (!q) return;
  loading.value = true;
  dicts.value.forEach(d => { d.html = null; d.error = null; });

  const results = await Promise.allSettled([
    search_xiandaihanyu(q).catch(e => { throw e; }),
    search_collins(q).catch(e => { throw e; }),
    search_ldoce(q).catch(e => { throw e; }),
  ]);

  dicts.value[0].html = results[0].status === "fulfilled" ? results[0].value : null;
  dicts.value[0].error = results[0].status === "rejected" ? String(results[0].reason) : null;
  dicts.value[1].html = results[1].status === "fulfilled" ? results[1].value : null;
  dicts.value[1].error = results[1].status === "rejected" ? String(results[1].reason) : null;
  dicts.value[2].html = results[2].status === "fulfilled" ? results[2].value : null;
  dicts.value[2].error = results[2].status === "rejected" ? String(results[2].reason) : null;

  loading.value = false;
}

function hasResults() {
  return dicts.value.some(d => d.html !== null);
}
</script>

<template>
  <div class="p-4 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('unifiedDict.title') }}</h1>

    <div class="flex gap-2 mb-4">
      <input ref="inputRef" v-model="query" :placeholder="t('unifiedDict.placeholder')"
             class="flex-1 px-4 py-2 border rounded text-sm" @keyup.enter="searchAll" />
      <button @click="searchAll" :disabled="loading || !query.trim()"
              class="px-4 py-2 bg-blue-500 text-white rounded text-sm disabled:opacity-50">
        {{ loading ? t('common.loading') : t('unifiedDict.search') }}
      </button>
    </div>

    <!-- 词频（初始状态） -->
    <div v-if="!query && !freqLoading && topWords.length" class="mb-6">
      <h2 class="text-sm font-medium text-gray-500 mb-2">{{ t('searchHistory.frequencyTab') }}</h2>
      <div class="space-y-1">
        <div v-for="w in topWords.slice(0, 15)" :key="w.id"
             class="flex items-center gap-2 text-sm cursor-pointer hover:bg-gray-50 px-2 py-1 rounded"
             @click="query = w.word; searchAll()">
          <span class="w-20 truncate font-medium">{{ w.word }}</span>
          <div class="flex-1 h-3 bg-gray-100 rounded-full overflow-hidden">
            <div class="h-full bg-blue-400 rounded-full transition-all"
                 :style="{ width: `${(w.hasSearchedTimes / maxFrequency) * 100}%` }"></div>
          </div>
          <span class="w-16 text-right text-xs text-gray-400">{{ w.hasSearchedTimes }}次</span>
        </div>
      </div>
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-400">{{ t('common.loading') }}...</div>

    <div v-else-if="hasResults()" class="space-y-4">
      <div v-for="d in dicts" :key="d.key">
        <div v-if="d.html" class="border rounded-lg overflow-hidden">
          <div class="bg-gray-100 px-3 py-1.5 text-sm font-medium border-b">{{ d.label }}</div>
          <iframe v-if="d.html" :srcdoc="d.html" class="w-full border-none"
                  style="height: min(60vh, 500px)" sandbox="allow-scripts"></iframe>
        </div>
        <div v-else-if="d.error" class="text-xs text-gray-400 px-1">{{ d.label }}: {{ d.error }}</div>
      </div>
    </div>

    <div v-else-if="query && !loading" class="text-center py-8 text-gray-400">
      {{ t('unifiedDict.noResults') }}
    </div>
  </div>
</template>
