<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { get_note_wasm, save_note_wasm } from "../types/wasm-typed";
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const noteId = route.params.id as string;
const content = ref('');
const filename = ref('');
const isLoading = ref(true);
const error = ref('');
const saveStatus = ref('');
const lastSavedTime = ref<Date | null>(null);

let saveTimer: number | undefined = undefined;
let isSaving = ref(false);

onMounted(async () => {
  await loadNoteDetail();
});

onUnmounted(() => {
  if (saveTimer !== undefined) {
    clearTimeout(saveTimer);
  }
});

const loadNoteDetail = async () => {
  try {
    const result = await get_note_wasm(noteId);
    content.value = result.text || '';
    filename.value = result.filename || '';
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('noteDetail.loadFailed');
    console.error('加载笔记详情失败:', err);
  } finally {
    isLoading.value = false;
  }
};

const onInput = () => {
  clearTimeout(saveTimer);

  saveStatus.value = t('common.saving');

  saveTimer = window.setTimeout(() => {
    autoSave();
  }, 500);
};

const autoSave = async () => {
  if (isSaving.value) {
    console.log('正在保存中，跳过本次保存');
    return;
  }

  if (!content.value.trim()) {
    console.log('内容为空，跳过保存');
    saveStatus.value = '';
    return;
  }

  isSaving.value = true;
  try {
    const result = await save_note_wasm(
      noteId,
      content.value,
      filename.value.trim() || undefined
    );

    console.log('自动保存成功:', result);

    lastSavedTime.value = new Date();
    saveStatus.value = t('noteDetail.saved') + ' ' + formatTime(lastSavedTime.value);
    error.value = '';

    setTimeout(() => {
      saveStatus.value = '';
    }, 3000);
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('common.error.saveFailed');
    saveStatus.value = t('common.error.saveFailed');
    console.error('自动保存失败:', err);

    setTimeout(() => {
      saveStatus.value = '';
    }, 3000);
  } finally {
    isSaving.value = false;
  }
};

const formatTime = (date: Date): string => {
  const now = new Date();
  const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

  if (diff < 60) {
    return t('common.timeAgo.justNow');
  } else if (diff < 3600) {
    return t('common.timeAgo.minutesAgo', { n: Math.floor(diff / 60) });
  } else if (diff < 86400) {
    return t('common.timeAgo.hoursAgo', { n: Math.floor(diff / 3600) });
  } else {
    return t('common.timeAgo.daysAgo', { n: Math.floor(diff / 86400) });
  }
};

const goBack = () => {
  router.push('/notes');
};
</script>

<template>
  <div class="min-h-screen bg-[#f5f5f5]">
    <div class="bg-white py-4 px-8 shadow-[0_2px_4px_rgba(0,0,0,0.1)] flex items-center justify-between sticky top-0 z-[100]">
      <button @click="goBack" class="py-2 px-4 bg-[#6b7280] text-white border-0 rounded-md cursor-pointer text-sm transition-colors duration-200 hover:bg-[#4b5563]">
        &larr; {{ t('noteDetail.backToList') }}
      </button>
      <h1 class="m-0 text-2xl text-[#1f2937] font-semibold">{{ t('noteDetail.title') }}</h1>
      <div class="flex items-center min-w-[120px] justify-end">
        <span v-if="isLoading" class="flex items-center text-[#6b7280] text-sm">
          <span class="w-4 h-4 border-2 border-[#e5e7eb] border-t-[#3b82f6] rounded-full mr-2 animate-[spin_0.6s_linear_infinite]"></span>
          {{ t('common.loading') }}
        </span>
        <span v-else-if="saveStatus" :class="['text-sm transition-colors duration-200', saveStatus === t('common.error.saveFailed') ? 'text-[#ef4444]' : 'text-[#10b981]']">
          {{ saveStatus }}
        </span>
      </div>
    </div>

    <div v-if="isLoading" class="flex flex-col items-center justify-center p-16">
      <div class="w-12 h-12 border-4 border-[#e5e7eb] border-t-[#3b82f6] rounded-full mb-4 animate-[spin_0.6s_linear_infinite]"></div>
      <p>{{ t('common.loading') }}</p>
    </div>

    <div v-else-if="error" class="p-8 text-center text-[#ef4444]">
      <p>{{ error }}</p>
      <button @click="loadNoteDetail" class="mt-4 py-2 px-4 bg-[#3b82f6] text-white border-0 rounded-md cursor-pointer hover:bg-[#2563eb]">{{ t('common.retry') }}</button>
    </div>

    <div v-else class="p-8 max-w-[1400px] mx-auto">
      <div class="bg-white p-6 rounded-lg shadow-[0_1px_3px_rgba(0,0,0,0.1)] mb-4">
        <label for="filename" class="block text-sm font-medium text-[#374151] mb-2">{{ t('noteDetail.filenamePlaceholder') }}</label>
        <input id="filename" v-model="filename" type="text" :placeholder="t('noteDetail.filenamePlaceholder')" class="w-full p-3 border border-[#d1d5db] rounded-md text-base transition-[border-color] duration-200 focus:outline-none focus:border-[#3b82f6] focus:shadow-[0_0_0_3px_rgba(59,130,246,0.1)]"
          @input="onInput" />
      </div>

      <div class="bg-white rounded-lg shadow-[0_1px_3px_rgba(0,0,0,0.1)] overflow-hidden">
        <MdEditor v-model="content" :preview="true" :toolbars="[
          'bold',
          'underline',
          'italic',
          'strikeThrough',
          'sub',
          'sup',
          'quote',
          'unorderedList',
          'orderedList',
          'codeRow',
          'code',
          'link',
          'image',
          'table',
          'mermaid',
          'katex',
          'revoke',
          'next',
          'pageFullscreen',
          'fullscreen',
          'preview',
          'htmlPreview',
          'catalog'
        ]" :preview-only="false" :show-toolbar-name="false" language="zh-CN" :placeholder="t('noteDetail.filenamePlaceholder')"
          class="w-full min-h-[600px]" @input="onInput" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.md-editor :deep(.md-editor-container) {
  min-height: 600px;
}

.md-editor :deep(.md-editor-content) {
  min-height: 550px;
}
</style>
