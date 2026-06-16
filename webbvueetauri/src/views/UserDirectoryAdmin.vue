<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import * as wasm from '../types/wasm-typed';
import type { UserDirectory } from '../types/models';

const { t } = useI18n();

const directories = ref<UserDirectory[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const newPath = ref('');
const newLabel = ref('');
const newAllowList = ref('');
const adding = ref(false);

async function fetchDirectories() {
  loading.value = true;
  error.value = null;
  try {
    const result = await wasm.list_user_directories_wasm() as any;
    directories.value = result || [];
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function addDirectory() {
  if (!newPath.value.trim()) return;
  adding.value = true;
  error.value = null;
  try {
    await wasm.add_user_directory_wasm(
      newPath.value.trim(),
      newLabel.value.trim() || null,
      newAllowList.value.trim() || null
    );
    newPath.value = '';
    newLabel.value = '';
    newAllowList.value = '';
    await fetchDirectories();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    adding.value = false;
  }
}

async function deleteDirectory(dir: UserDirectory) {
  if (!confirm(t('userDirectoryAdmin.confirmDelete', { path: dir.path }))) return;
  error.value = null;
  try {
    await wasm.delete_user_directory_wasm(dir.id);
    await fetchDirectories();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function updateAllowList(dir: UserDirectory) {
  const input = prompt(t('userDirectoryAdmin.editAllowListPrompt'), (dir.allow_list || []).join(', '));
  if (input === null) return;
  error.value = null;
  try {
    await wasm.add_user_directory_wasm(
      dir.path,
      dir.label || null,
      input.trim() || null
    );
    await fetchDirectories();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(fetchDirectories);
</script>

<template>
  <div class="p-4 max-w-[900px] mx-auto">
    <div class="flex items-center gap-3 mb-4">
      <h2 class="flex-1 m-0 text-[1.4em]">{{ t('userDirectoryAdmin.directoryAdmin') }}</h2>
    </div>

    <div v-if="loading" class="text-center p-8 text-[#666]">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="text-center p-8 text-[#e74c3c]">{{ error }}</div>

    <!-- 添加表单 -->
    <div class="flex gap-2 flex-wrap mb-4">
      <input v-model="newPath" :placeholder="t('userDirectoryAdmin.pathPlaceholder')" class="flex-1 min-w-[180px] px-3 py-2 border border-[#ddd] rounded-md text-sm" />
      <input v-model="newLabel" :placeholder="t('userDirectoryAdmin.labelPlaceholder')" class="flex-1 min-w-[180px] px-3 py-2 border border-[#ddd] rounded-md text-sm" />
      <input v-model="newAllowList" :placeholder="t('userDirectoryAdmin.allowListPlaceholder')" class="flex-1 min-w-[180px] px-3 py-2 border border-[#ddd] rounded-md text-sm" />
      <button @click="addDirectory" :disabled="adding || !newPath.trim()" class="px-5 py-2 bg-[#1976d2] text-white border-0 rounded-md cursor-pointer text-sm disabled:opacity-60 disabled:cursor-not-allowed">
        {{ adding ? t('userDirectoryAdmin.adding') : t('userDirectoryAdmin.add') }}
      </button>
    </div>

    <div v-if="directories.length === 0 && !loading" class="text-center p-8 text-[#666]">{{ t('userDirectoryAdmin.noDirectories') }}</div>

    <div class="flex flex-col gap-2">
      <div v-for="dir in directories" :key="dir.id" class="flex items-center justify-between px-4 py-3 border border-[#eee] rounded-lg gap-3">
        <div class="flex items-center gap-3 flex-1 min-w-0">
          <span class="text-2xl">📁</span>
          <div class="flex flex-col min-w-0">
            <span class="font-medium break-all">{{ dir.path }}</span>
            <span v-if="dir.label" class="text-[13px] text-[#999]">{{ dir.label }}</span>
          </div>
        </div>
        <div class="flex items-center gap-1 flex-wrap">
          <span
            v-for="user in (dir.allow_list || [])"
            :key="user"
            class="inline-block px-2 py-0.5 bg-[#e8f4fd] text-[#2980b9] rounded text-xs"
          >{{ user }}</span>
          <span v-if="!dir.allow_list || dir.allow_list.length === 0" class="text-xs text-[#999]">{{ t('userDirectoryAdmin.noAllowUsers') }}</span>
          <button class="px-2.5 py-1 bg-[#f5f5f5] border border-[#ddd] rounded cursor-pointer text-xs ml-1 hover:bg-[#e8e8e8]" @click="updateAllowList(dir)">{{ t('common.edit') }}</button>
        </div>
        <button class="px-3.5 py-1.5 bg-[#e74c3c] text-white border-0 rounded-md cursor-pointer text-[13px] shrink-0 hover:bg-[#c0392b]" @click="deleteDirectory(dir)">{{ t('common.delete') }}</button>
      </div>
    </div>
  </div>
</template>
