<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import {
  list_short_notes_wasm,
  create_short_note_wasm,
  update_short_note_wasm,
  delete_short_note_wasm,
} from "../types/wasm-typed";
import type { ShortNote } from "../types/models";

const { t, locale } = useI18n();

const notes = ref<ShortNote[]>([]);
const loading = ref(false);
const error = ref("");
const page = ref(1);
const pageSize = ref(10);

// 表单数据
const showModal = ref(false);
const isEditing = ref(false);
const editingId = ref<string>("");
const formContent = ref("");
const formViewName = ref("");

// 加载短笔记列表
async function loadNotes() {
  loading.value = true;
  error.value = "";
  try {
    const result = await list_short_notes_wasm(page.value, pageSize.value);
    notes.value = result.data || [];
  } catch (err) {
    console.error("加载短笔记失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.loadFailed');
  } finally {
    loading.value = false;
  }
}

// 打开创建弹窗
function openCreateModal() {
  isEditing.value = false;
  editingId.value = "";
  formContent.value = "";
  formViewName.value = "";
  showModal.value = true;
}

// 打开编辑弹窗
function openEditModal(note: ShortNote) {
  isEditing.value = true;
  editingId.value = note.id;
  formContent.value = note.content || "";
  formViewName.value = note.view_name || "";
  showModal.value = true;
}

// 关闭弹窗
function closeModal() {
  showModal.value = false;
}

// 保存短笔记
async function saveNote() {
  loading.value = true;
  error.value = "";
  try {
    if (isEditing.value) {
      await update_short_note_wasm(
        editingId.value,
        formContent.value || null,
        formViewName.value || null
      );
    } else {
      await create_short_note_wasm(
        formContent.value || null,
        formViewName.value || null
      );
    }
    closeModal();
    await loadNotes();
  } catch (err) {
    console.error("保存短笔记失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.saveFailed');
  } finally {
    loading.value = false;
  }
}

// 删除短笔记
async function deleteNote(id: string) {
  if (!confirm(t('shortNote.confirmDelete'))) {
    return;
  }

  loading.value = true;
  error.value = "";
  try {
    await delete_short_note_wasm(id);
    await loadNotes();
  } catch (err) {
    console.error("删除短笔记失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.deleteFailed');
  } finally {
    loading.value = false;
  }
}

// 格式化日期
function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US');
}

onMounted(() => {
  loadNotes();
});
</script>

<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold text-center text-blue-600 mb-8">{{ t('shortNote.title') }}</h1>

    <!-- 错误提示 -->
    <div v-if="error" class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
      {{ error }}
    </div>

    <!-- 操作按钮 -->
    <div class="mb-6">
      <button
        @click="openCreateModal"
        class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        :disabled="loading"
      >
        {{ t('shortNote.createNote') }}
      </button>
      <button
        @click="loadNotes"
        class="ml-2 px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
        :disabled="loading"
      >
        {{ t('common.refresh') }}
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center py-8">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      <p class="mt-2 text-gray-600">{{ t('common.loading') }}</p>
    </div>

    <!-- 笔记列表 -->
    <div v-else-if="notes.length > 0" class="space-y-4">
      <div
        v-for="note in notes"
        :key="note.id"
        class="bg-white p-6 rounded-lg shadow-md border border-gray-200"
      >
        <div class="flex justify-between items-start">
          <div class="flex-1">
            <h3 class="text-lg font-semibold text-gray-800 mb-2">
              {{ note.view_name || t('shortNote.untitled') }}
            </h3>
            <p class="text-gray-600 whitespace-pre-wrap">{{ note.content || t('shortNote.noContent') }}</p>
            <p class="text-sm text-gray-400 mt-2">{{ formatDate(note.created_at) }}</p>
          </div>
          <div class="flex space-x-2 ml-4">
            <button
              @click="openEditModal(note)"
              class="px-3 py-1 bg-yellow-500 text-white rounded hover:bg-yellow-600 transition-colors text-sm"
            >
              {{ t('common.edit') }}
            </button>
            <button
              @click="deleteNote(note.id)"
              class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 transition-colors text-sm"
            >
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="text-center py-12 bg-gray-50 rounded-lg">
      <p class="text-gray-500 text-lg">{{ t('shortNote.noNotes') }}</p>
      <button
        @click="openCreateModal"
        class="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
      >
        {{ t('shortNote.createFirst') }}
      </button>
    </div>

    <!-- 创建/编辑弹窗 -->
    <div
      v-if="showModal"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    >
      <div class="bg-white rounded-lg shadow-xl w-full max-w-lg mx-4">
        <div class="p-6">
          <h2 class="text-xl font-bold mb-4">
            {{ isEditing ? t('shortNote.editNote') : t('shortNote.newNote') }}
          </h2>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('shortNote.titleLabel') }}</label>
              <input
                v-model="formViewName"
                type="text"
                :placeholder="t('shortNote.titlePlaceholder')"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('shortNote.contentLabel') }}</label>
              <textarea
                v-model="formContent"
                rows="6"
                :placeholder="t('shortNote.contentPlaceholder')"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              ></textarea>
            </div>
          </div>

          <div class="flex justify-end space-x-3 mt-6">
            <button
              @click="closeModal"
              class="px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="saveNote"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
              :disabled="loading"
            >
              {{ loading ? t('common.saving') : t('common.save') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
