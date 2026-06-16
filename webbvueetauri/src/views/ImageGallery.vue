<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import * as wasm from '../types/wasm-typed';
import { useFiles } from '../composables/useFiles';
import { getApiUrl } from '../utils/apiUrl';
import type { ImageItem, PaginatedResponse } from '../types/models';

const { t } = useI18n();
const { formatSize } = useFiles();

const folders = ref<string[]>([]);
const images = ref<ImageItem[]>([]);
const selectedFolder = ref<string | null>(null);
const loading = ref(false);
const loadingMore = ref(false);
const scanning = ref(false);
const error = ref<string | null>(null);
const currentPage = ref(0);
const hasMore = ref(true);
const pageSize = 48;

async function fetchFolders() {
  loading.value = true;
  error.value = null;
  try {
    const result = await wasm.get_image_folders_wasm() as string[];
    folders.value = result || [];
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function fetchImages(reset = false) {
  if (reset) {
    loading.value = true;
    images.value = [];
    currentPage.value = 0;
    hasMore.value = true;
  } else {
    if (!hasMore.value || loadingMore.value) return;
    loadingMore.value = true;
  }
  error.value = null;

  try {
    const page = reset ? 1 : currentPage.value + 1;
    const result = await wasm.list_images_wasm(
      selectedFolder.value,
      page,
      pageSize
    ) as PaginatedResponse<ImageItem>;
    const newImages = result.data || [];
    if (reset) {
      images.value = newImages;
    } else {
      images.value.push(...newImages);
    }
    currentPage.value = result.page;
    hasMore.value = result.page < result.total_pages;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function selectFolder(folder: string) {
  selectedFolder.value = folder;
  fetchImages(true);
}

function goBack() {
  selectedFolder.value = null;
  images.value = [];
}

async function scanImages() {
  scanning.value = true;
  try {
    await wasm.scan_images_wasm();
    await fetchFolders();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    scanning.value = false;
  }
}

function getFolderName(folder: string): string {
  const parts = folder.split('/');
  return parts[parts.length - 1] || folder;
}

function getThumbnailUrl(img: ImageItem): string {
  const base = getApiUrl();
  const token = localStorage.getItem('token') || '';
  return `${base}/api/images/thumbnail/${img.id}?token=${token}`;
}

onMounted(() => {
  fetchFolders();
});
</script>

<template>
  <div class="p-4 max-w-[1200px] mx-auto">
    <div class="flex items-center gap-3 mb-4">
      <button v-if="selectedFolder" class="py-2 px-4 border border-[#ddd] rounded-[6px] bg-white cursor-pointer text-sm hover:bg-[#f5f5f5]" @click="goBack">
        ← {{ t('imageGallery.back') }}
      </button>
      <h2 class="flex-1 m-0 text-[1.4em]">{{ selectedFolder ? getFolderName(selectedFolder) : t('imageGallery.title') }}</h2>
      <button class="py-2 px-4 border border-[#ddd] rounded-[6px] bg-white cursor-pointer text-sm hover:bg-[#f5f5f5] disabled:opacity-60 disabled:cursor-not-allowed" @click="scanImages" :disabled="scanning">
        {{ scanning ? t('imageGallery.scanning') : t('imageGallery.scan') }}
      </button>
    </div>

    <div v-if="loading" class="text-center p-8 text-[#666]">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="text-center p-8 text-[#e74c3c]">{{ error }}</div>

    <!-- 文件夹列表 -->
    <div v-else-if="!selectedFolder" class="flex flex-col gap-2">
      <div v-if="folders.length === 0" class="text-center p-8 text-[#666]">{{ t('imageGallery.noImageFolders') }}</div>
      <div
        v-for="folder in folders"
        :key="folder"
        class="flex items-center gap-3 py-3 px-4 border border-[#eee] rounded-lg cursor-pointer transition-colors duration-200 hover:bg-[#f8f9fa]"
        @click="selectFolder(folder)"
      >
        <span class="text-2xl">📁</span>
        <span class="font-medium text-base">{{ getFolderName(folder) }}</span>
        <span class="text-[#999] text-[13px] ml-auto overflow-hidden text-ellipsis whitespace-nowrap max-w-[300px]">{{ folder }}</span>
      </div>
    </div>

    <!-- 图片网格 -->
    <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3">
      <div v-if="images.length === 0" class="text-center p-8 text-[#666]">{{ t('imageGallery.noImagesInFolder') }}</div>
      <div
        v-for="img in images"
        :key="img.id"
        class="border border-[#eee] rounded-lg overflow-hidden transition-transform duration-200 hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(0,0,0,0.1)]"
      >
        <img
          :src="getThumbnailUrl(img)"
          :alt="img.name"
          loading="lazy"
          class="w-full h-[160px] object-cover block bg-[#f5f5f5]"
          @error="($event.target as HTMLImageElement).src = 'data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%22100%22 height=%22100%22><text y=%2250%25%22 x=%2250%25%22 text-anchor=%22middle%22 dominant-baseline=%22middle%22 font-size=%2240%22>🖼️</text></svg>'"
        />
        <div class="p-2 flex flex-col gap-0.5">
          <span class="text-[13px] overflow-hidden text-ellipsis whitespace-nowrap">{{ img.name }}</span>
          <span class="text-xs text-[#999]">{{ formatSize(img.size) }}</span>
        </div>
      </div>
      <div v-if="loadingMore" class="text-center p-8 text-[#666]">{{ t('common.loadingMore') }}</div>
      <div v-else-if="hasMore" class="text-center p-8 text-[#666] col-span-full cursor-pointer text-[#3498db] hover:underline" @click="fetchImages(false)">{{ t('imageGallery.loadMore') }}</div>
    </div>
  </div>
</template>
