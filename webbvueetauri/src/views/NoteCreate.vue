<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { save_note_wasm } from "../types/wasm-typed";
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const { t } = useI18n();
const router = useRouter();

const content = ref('');
const filename = ref('');
const noteId = ref<string | null>(null);
const isLoading = ref(false);
const error = ref('');

const saveStatus = ref('');
const lastSavedTime = ref<Date | null>(null);

let saveTimer: number | undefined = undefined;
let isSaving = false;

const generateUUID = (): string => {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
};

const startEditing = () => {
  if (!noteId.value) {
    noteId.value = generateUUID();
    console.log('生成新笔记ID:', noteId.value);
  }
};

const onInput = () => {
  clearTimeout(saveTimer);

  saveStatus.value = t('noteCreate.savingProgress');

  saveTimer = window.setTimeout(() => {
    autoSave();
  }, 500);
};

const autoSave = async () => {
  if (isSaving) {
    console.log('正在保存中，跳过本次保存');
    return;
  }

  if (!content.value.trim()) {
    console.log('内容为空，跳过保存');
    saveStatus.value = '';
    return;
  }

  isSaving = true;
  try {
    const result = await save_note_wasm(
      noteId.value,
      content.value,
      filename.value.trim() || undefined
    );

    console.log('自动保存成功:', result);

    noteId.value = result.id;
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
    isSaving = false;
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

onMounted(() => {
  startEditing();
});

onUnmounted(() => {
  if (saveTimer !== undefined) {
    clearTimeout(saveTimer);
  }
});
</script>

<template>
  <div class="min-h-screen bg-[#f5f5f5]">
    <div class="bg-white py-4 px-8 shadow-md flex items-center justify-between sticky top-0 z-[100]">
      <button @click="goBack"
        class="py-2 px-4 bg-gray-500 text-white border-none rounded cursor-pointer text-sm transition-colors duration-200 hover:bg-gray-600">
        &larr; {{ t('noteCreate.back') }}
      </button>
      <h1 class="m-0 text-2xl text-gray-800 font-semibold">{{ t('noteCreate.title') }}</h1>
      <div class="flex items-center min-w-[120px] justify-end">
        <span v-if="isLoading" class="flex items-center text-gray-500 text-sm">
          <span
            class="w-4 h-4 border-2 border-gray-200 border-t-blue-500 rounded-full animate-[spin_0.6s_linear_infinite] mr-2"></span>
          {{ t('noteCreate.saving') }}
        </span>
        <span v-else-if="saveStatus"
          :class="[saveStatus === t('common.error.saveFailed') ? 'text-red-500' : 'text-emerald-500', 'text-sm transition-colors duration-200']">
          {{ saveStatus }}
        </span>
      </div>
    </div>

    <div class="p-8 max-w-[1400px] mx-auto">
      <div v-if="error"
        class="bg-red-100 text-red-800 p-4 rounded-lg mb-6 border border-red-200">
        {{ error }}
      </div>

      <div class="bg-white p-6 rounded-lg shadow-sm mb-4">
        <label for="filename" class="block text-sm font-medium text-gray-700 mb-2">{{ t('noteCreate.filenamePlaceholder') }}</label>
        <input id="filename" v-model="filename" type="text" :placeholder="t('noteCreate.filenamePlaceholder')"
          class="w-full p-3 border border-gray-300 rounded text-base transition-[border-color,box-shadow] duration-200 focus:outline-none focus:border-blue-500 focus:shadow-[0_0_0_3px_rgba(59,130,246,0.1)]"
          @input="onInput" />
      </div>

      <div class="bg-white rounded-lg shadow-sm overflow-hidden">
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
        ]" :preview-only="false" :show-toolbar-name="false" language="zh-CN" :placeholder="t('noteCreate.filenamePlaceholder')"
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
