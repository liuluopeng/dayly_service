<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { get_clipboard_history_wasm } from "../types/wasm-typed";
import type { ClipboardEntry } from "../types/models";

const { t } = useI18n();

const entries = ref<ClipboardEntry[]>([]);
const loading = ref(false);
const error = ref("");

const filterType = ref<"all" | "text" | "image">("all");
const searchQuery = ref("");
const count = ref(50);

async function loadHistory() {
  loading.value = true;
  error.value = "";
  try {
    const type_filter = filterType.value === "all" ? null : filterType.value;
    const search = searchQuery.value || null;
    entries.value = await get_clipboard_history_wasm(count.value, type_filter, search);
  } catch (err) {
    console.error("加载剪贴板历史失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.loadFailed');
  } finally {
    loading.value = false;
  }
}

const filteredEntries = computed(() => entries.value);

function copyText(text: string) {
  navigator.clipboard.writeText(text).then(() => {
    // 复制成功提示
  });
}

function formatTime(ts: string) {
  try {
    const d = new Date(ts);
    return d.toLocaleString();
  } catch {
    return ts;
  }
}

function isTextEntry(entry: ClipboardEntry) {
  return entry.type === "text";
}

function getImageUrl(entry: ClipboardEntry) {
  return entry.image_url || "";
}

onMounted(loadHistory);
</script>

<template>
  <div class="p-4 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('menu.clipboardHistory.title') }}</h1>

    <!-- 控制栏 -->
    <div class="flex flex-wrap gap-3 mb-4 items-center">
      <div class="flex gap-2">
        <button
          class="px-3 py-1.5 rounded text-sm"
          :class="filterType === 'all' ? 'bg-blue-500 text-white' : 'bg-gray-200'"
          @click="filterType = 'all'; loadHistory()"
        >{{ t('common.all') }}</button>
        <button
          class="px-3 py-1.5 rounded text-sm"
          :class="filterType === 'text' ? 'bg-blue-500 text-white' : 'bg-gray-200'"
          @click="filterType = 'text'; loadHistory()"
        >{{ t('menu.clipboardHistory.text') }}</button>
        <button
          class="px-3 py-1.5 rounded text-sm"
          :class="filterType === 'image' ? 'bg-blue-500 text-white' : 'bg-gray-200'"
          @click="filterType = 'image'; loadHistory()"
        >{{ t('menu.clipboardHistory.image') }}</button>
      </div>

      <div class="flex gap-2 flex-1">
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('common.search') + '...'"
          class="flex-1 px-3 py-1.5 border rounded text-sm"
          @keyup.enter="loadHistory"
        />
        <button
          class="px-3 py-1.5 bg-blue-500 text-white rounded text-sm"
          @click="loadHistory"
        >{{ t('common.search') }}</button>
      </div>

      <button
        class="px-3 py-1.5 bg-green-500 text-white rounded text-sm"
        @click="loadHistory"
      >{{ t('common.refresh') }}</button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="text-center py-8 text-gray-500">{{ t('common.loading') }}</div>

    <!-- 错误提示 -->
    <div v-else-if="error" class="bg-red-100 border border-red-300 text-red-700 px-4 py-3 rounded mb-4">
      {{ error }}
    </div>

    <!-- 空状态 -->
    <div v-else-if="entries.length === 0" class="text-center py-8 text-gray-400">
      {{ t('common.noData') }}
    </div>

    <!-- 条目列表 -->
    <div v-else class="space-y-3">
      <div
        v-for="entry in filteredEntries"
        :key="entry.id"
        class="border rounded-lg p-3 hover:shadow-sm transition-shadow"
      >
        <!-- 文本条目 -->
        <template v-if="isTextEntry(entry)">
          <p class="text-sm whitespace-pre-wrap break-words mb-2">{{ entry.text_content }}</p>
          <div class="flex gap-2">
            <button
              class="text-xs px-2 py-1 bg-gray-100 rounded hover:bg-gray-200"
              @click="copyText(entry.text_content || '')"
            >{{ t('common.copy') }}</button>
          </div>
        </template>

        <!-- 图片条目 -->
        <template v-else>
          <img
            v-if="getImageUrl(entry)"
            :src="getImageUrl(entry)"
            class="max-w-full max-h-64 rounded object-contain mb-2"
            alt="clipboard image"
            loading="lazy"
          />
          <p v-else class="text-sm text-gray-500 italic">{{ t('menu.clipboardHistory.noPreview') }}</p>
        </template>

        <!-- 时间戳 -->
        <div class="text-xs text-gray-400 mt-1">{{ formatTime(entry.created_at) }}</div>
      </div>
    </div>
  </div>
</template>
