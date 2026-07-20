<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { search_xiandaihanyu, search_collins, search_ldoce, get_top_words, get_recent_history } from "../types/wasm-typed";
import type { Word, WordHistory } from "../types/models";

const { t, locale } = useI18n();
const inputRef = ref<HTMLInputElement | null>(null);
const topWords = ref<Word[]>([]);
const recentHistory = ref<WordHistory[]>([]);
const freqLoading = ref(true);
const activeTab = ref<"search" | "stats">("search");
const searchCount = ref(0);

const maxFrequency = computed(() => {
  if (topWords.value.length === 0) return 1;
  return Math.max(...topWords.value.map(w => w.hasSearchedTimes));
});

function formatTime(ts: string) {
  try { return new Date(ts).toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US'); } catch { return ts; }
}

onMounted(async () => {
  setTimeout(() => inputRef.value?.focus(), 100);
  try {
    const [freq, hist] = await Promise.all([
      get_top_words(),
      get_recent_history(BigInt(20)),
    ]);
    topWords.value = freq.data || [];
    recentHistory.value = hist.data || [];
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

  // 刷新统计 + 获取搜索次数
  try {
    const [freq, hist] = await Promise.all([
      get_top_words(),
      get_recent_history(BigInt(20)),
    ]);
    topWords.value = freq.data || [];
    recentHistory.value = hist.data || [];
    searchCount.value = topWords.value.find(w => w.word === q)?.hasSearchedTimes ?? 0;
  } catch {}

  loading.value = false;
}

function hasResults() {
  return dicts.value.some(d => d.html !== null);
}
</script>

<template>
  <div class="p-4 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('unifiedDict.title') }}</h1>

    <!-- Tab 切换 -->
    <div class="flex gap-4 mb-4 border-b">
      <button @click="activeTab = 'search'"
              class="pb-1.5 text-sm font-medium border-b-2 transition-colors"
              :class="activeTab === 'search' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500'">
        {{ t('unifiedDict.search') }}
      </button>
      <button @click="activeTab = 'stats'"
             class="pb-1.5 text-sm font-medium border-b-2 transition-colors"
             :class="activeTab === 'stats' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500'">
        {{ t('unifiedDict.stats') }}
      </button>
    </div>

    <!-- 搜索 Tab -->
    <template v-if="activeTab === 'search'">
    <div class="flex gap-2 mb-4">
      <input ref="inputRef" v-model="query" :placeholder="t('unifiedDict.placeholder')"
             class="flex-1 px-4 py-2 border rounded text-sm" @keyup.enter="searchAll" />
      <button @click="searchAll" :disabled="loading || !query.trim()"
              class="px-4 py-2 bg-blue-500 text-white rounded text-sm disabled:opacity-50">
        {{ loading ? t('common.loading') : t('unifiedDict.searchBtn') }}
      </button>
    </div>

    <div v-if="searchCount > 0 && !loading && query" class="text-xs text-gray-400 mb-2">
      {{ t('unifiedDict.searchCount', { word: query, count: searchCount }) }}
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-400">{{ t('common.loading') }}...</div>

    <div v-else-if="hasResults()" class="space-y-4">
      <div v-for="d in dicts" :key="d.key">
        <div v-if="d.html" class="border rounded-lg overflow-hidden">
          <div class="bg-gray-100 px-3 py-1.5 text-sm font-medium border-b">{{ d.label }}</div>
          <iframe :srcdoc="d.html" class="w-full border-none"
                  style="height: min(60vh, 500px)" sandbox="allow-scripts"></iframe>
        </div>
        <div v-else-if="d.error" class="text-xs text-gray-400 px-1">{{ d.label }}: {{ d.error }}</div>
      </div>
    </div>

    <div v-else-if="query && !loading" class="text-center py-8 text-gray-400">
      {{ t('unifiedDict.noResults') }}
    </div>
    </template>

    <!-- 统计 Tab -->
    <template v-if="activeTab === 'stats'">
      <div v-if="freqLoading" class="text-center py-8 text-gray-400">{{ t('common.loading') }}...</div>
      <template v-else>
        <!-- 查询历史 -->
        <section class="mb-6">
          <h2 class="text-sm font-semibold text-gray-600 mb-2">{{ t('unifiedDict.history') }}</h2>
          <div v-if="recentHistory.length" class="space-y-1">
            <div v-for="(h, i) in recentHistory" :key="i"
                 class="flex items-center justify-between px-2 py-1 text-sm hover:bg-gray-50 rounded cursor-pointer"
                 @click="query = h.word; activeTab = 'search'; searchAll()">
              <span class="font-medium">{{ h.word }}</span>
              <span class="text-xs text-gray-400">{{ formatTime(h.time) }}</span>
            </div>
          </div>
          <p v-else class="text-sm text-gray-400 px-2">{{ t('searchHistory.noHistory') }}</p>
        </section>

        <!-- 词频 -->
        <section>
          <h2 class="text-sm font-semibold text-gray-600 mb-2">{{ t('unifiedDict.frequency') }}</h2>
          <div class="space-y-1">
            <div v-for="w in topWords.slice(0, 30)" :key="w.id"
                 class="flex items-center gap-2 text-sm cursor-pointer hover:bg-gray-50 px-2 py-1 rounded"
                 @click="query = w.word; activeTab = 'search'; searchAll()">
              <span class="w-24 truncate font-medium">{{ w.word }}</span>
              <div class="flex-1 h-3 bg-gray-100 rounded-full overflow-hidden">
                <div class="h-full bg-blue-400 rounded-full" :style="{ width: `${(w.hasSearchedTimes / maxFrequency) * 100}%` }"></div>
              </div>
              <span class="w-16 text-right text-xs text-gray-400">{{ w.hasSearchedTimes }}{{ t('searchHistory.timesUnit') }}</span>
            </div>
            <p v-if="!topWords.length" class="text-sm text-gray-400 px-2">{{ t('searchHistory.noFrequency') }}</p>
          </div>
        </section>
      </template>
    </template>
  </div>
</template>
