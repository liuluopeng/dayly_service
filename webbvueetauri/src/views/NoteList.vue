<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { list_notes_wasm, search_notes_wasm } from "../types/wasm-typed";

const { t, locale } = useI18n();

const notes = ref<any[]>([]);
const isLoading = ref(true);
const error = ref('');
const currentPage = ref(1);
const pageSize = 20;
const searchQuery = ref('');
const isSearching = ref(false);

onMounted(async () => {
  await loadNotes();
});

const loadNotes = async () => {
  try {
    isLoading.value = true;
    error.value = '';

    const results = await list_notes_wasm(currentPage.value, pageSize);
    notes.value = results as any[];
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('noteList.loadFailed');
    console.error('加载笔记列表失败:', err);
  } finally {
    isLoading.value = false;
  }
};

const handleSearch = async () => {
  if (!searchQuery.value.trim()) {
    currentPage.value = 1;
    await loadNotes();
    return;
  }

  try {
    isSearching.value = true;
    error.value = '';
    isLoading.value = true;

    const results = await search_notes_wasm(searchQuery.value);
    notes.value = results as any[];
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('noteSearch.searchFailed');
    console.error('搜索失败:', err);
  } finally {
    isSearching.value = false;
    isLoading.value = false;
  }
};

const loadMore = () => {
  currentPage.value++;
  loadMoreNotes();
};

const loadMoreNotes = async () => {
  try {
    const results = await list_notes_wasm(currentPage.value, pageSize);
    notes.value = [...notes.value, ...(results as any[])];
  } catch (err) {
    console.error('加载更多笔记失败:', err);
    currentPage.value--;
  }
};

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US');
};
</script>

<template>
  <div class="min-h-screen bg-[#f5f5f5]">
    <div class="bg-white px-8 py-4 shadow-[0_2px_4px_rgba(0,0,0,0.1)] flex justify-between items-center">
      <h1 class="m-0 text-2xl text-[#1f2937]">{{ t('noteList.title') }}</h1>
      <div class="flex gap-4 items-center">
        <div class="flex items-center gap-2">
          <input v-model="searchQuery" type="text" :placeholder="t('noteList.searchPlaceholder')"
            class="py-2 px-3 border border-[#d1d5db] rounded-md text-sm min-w-[200px] transition-[border-color] duration-200 focus:outline-none focus:border-[#3b82f6] focus:shadow-[0_0_0_3px_rgba(59,130,246,0.1)]"
            @keyup.enter="handleSearch" />
          <button @click="handleSearch"
            class="py-2 px-3 bg-[#3b82f6] text-white border-none rounded-md cursor-pointer text-sm transition-[background-color] duration-200 flex items-center justify-center min-w-[40px] hover:not-disabled:bg-[#2563eb] disabled:bg-[#9ca3af] disabled:cursor-not-allowed"
            :disabled="isSearching">
            <span v-if="isSearching" class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-[spin_0.6s_linear_infinite]"></span>
            <span v-else>🔍</span>
          </button>
        </div>
        <router-link to="/note-create"
          class="py-2 px-4 bg-[#3b82f6] text-white border-none rounded-md cursor-pointer text-sm transition-[background-color] duration-200 no-underline hover:bg-[#2563eb]">
          + {{ t('noteList.createNote') }}
        </router-link>
      </div>
    </div>

    <div v-if="isLoading && notes.length === 0" class="flex flex-col items-center justify-center p-16">
      <div class="w-12 h-12 border-4 border-[#e5e7eb] border-t-[#3b82f6] rounded-full animate-spin"></div>
      <p>{{ t('common.loading') }}</p>
    </div>

    <div v-else-if="error" class="flex flex-col items-center justify-center p-16 text-[#ef4444]">
      <p>{{ error }}</p>
      <button @click="loadNotes"
        class="mt-4 py-2 px-4 bg-[#3b82f6] text-white border-none rounded-md cursor-pointer hover:bg-[#2563eb]">{{ t('common.retry') }}</button>
    </div>

    <div v-else>
      <div v-if="notes.length === 0" class="flex flex-col items-center justify-center p-16 text-[#6b7280]">
        <p>{{ t('noteList.noNotes') }}</p>
      </div>

      <div v-else class="max-w-[1200px] mx-auto py-8 px-4">
        <div v-for="(note, index) in notes" :key="note.id"
          class="bg-white p-6 mb-4 rounded-lg shadow-[0_1px_3px_rgba(0,0,0,0.1)] transition-shadow duration-200 hover:shadow-[0_4px_6px_rgba(0,0,0,0.1)]">
          <div class="flex justify-between items-center mb-4">
            <h3 class="m-0 text-lg font-semibold text-[#1f2937]">{{ note.filename || t('noteList.noteIndex', { index: index + 1 }) }}</h3>
            <span class="text-sm text-[#6b7280]">{{ formatDate(note.created_at) }}</span>
          </div>
          <div class="mb-4">
            <p v-if="note.text" class="text-[#4b5563] leading-[1.6] m-0 line-clamp-3"> {{ note.text }}</p>
            <p v-else class="text-[#9ca3af] italic leading-[1.6] m-0 line-clamp-3">{{ t('common.noContent') }}</p>
          </div>
          <div class="flex justify-end">
            <router-link :to="`/note/${note.id}`"
              class="text-[#3b82f6] no-underline text-sm py-2 px-4 border border-[#3b82f6] rounded-md transition-[background-color,color] duration-200 hover:bg-[#3b82f6] hover:text-white">
              {{ t('noteList.viewDetail') }}
            </router-link>
          </div>
        </div>

        <div v-if="isLoading && notes.length > 0" class="flex items-center justify-center p-8 text-[#6b7280]">
          <div class="w-8 h-8 border-[3px] border-[#e5e7eb] border-t-[#3b82f6] rounded-full animate-spin"></div>
          <p class="ml-4">{{ t('common.loadingMore') }}</p>
        </div>

        <div v-else class="flex justify-center p-8">
          <button @click="loadMore"
            class="py-3 px-8 bg-[#3b82f6] text-white border-none rounded-md cursor-pointer text-base transition-[background-color] duration-200 hover:bg-[#2563eb]">
            {{ t('common.loadMore') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
