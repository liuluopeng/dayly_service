<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { search_xiandaihanyu, search_collins, search_ldoce } from "../types/wasm-typed";

const { t } = useI18n();
const inputRef = ref<HTMLInputElement | null>(null);

onMounted(() => setTimeout(() => inputRef.value?.focus(), 100));

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
