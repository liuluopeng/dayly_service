<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import * as wasm from '../types/wasm-typed';
import { useFiles } from '../composables/useFiles';
import type { VideoItem, MediaPathItem, PaginatedResponse, FileEntry, DirListing } from '../types/models';

const { t } = useI18n();
const router = useRouter();
const { formatSize } = useFiles();

function getPreviewUrl(videoId: string): string {
  const baseUrl = wasm.get_base_url_wasm();
  const token = localStorage.getItem('token') || '';
  return `${baseUrl}/api/videos/preview/${videoId}?token=${token}`;
}

const videos = ref<VideoItem[]>([]);
const loading = ref(false);
const loadingMore = ref(false);
const scanning = ref(false);
const error = ref<string | null>(null);
const currentPage = ref(0);
const hasMore = ref(true);
const pageSize = 50;

// 目录选择器状态
const showDirPicker = ref(false);
const dirEntries = ref<FileEntry[]>([]);
const dirLoading = ref(false);
const currentDirPath = ref('/');
const dirBreadcrumbs = ref<string[]>(['/']);
const addingDir = ref(false);

// 已有媒体路径
const mediaPaths = ref<MediaPathItem[]>([]);

async function fetchVideos(reset = false) {
  if (reset) {
    loading.value = true;
    videos.value = [];
    currentPage.value = 0;
    hasMore.value = true;
  } else {
    if (!hasMore.value || loadingMore.value) return;
    loadingMore.value = true;
  }
  error.value = null;

  try {
    const page = reset ? 1 : currentPage.value + 1;
    const result = await wasm.list_videos_wasm(null, page, pageSize) as PaginatedResponse<VideoItem>;
    const newVideos = result.data || [];
    if (reset) {
      videos.value = newVideos;
    } else {
      videos.value.push(...newVideos);
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

function playVideo(video: VideoItem) {
  router.push({ path: '/video', query: { path: video.path } });
}

async function scanVideos() {
  scanning.value = true;
  try {
    await wasm.scan_videos_wasm();
    await fetchVideos(true);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    scanning.value = false;
  }
}

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

// 目录选择器
async function openDirPicker() {
  showDirPicker.value = true;
  currentDirPath.value = '/';
  dirBreadcrumbs.value = ['/'];
  await fetchMediaPaths();
  await browseDirectory('/');
}

async function fetchMediaPaths() {
  try {
    const result = await wasm.list_media_paths_wasm('video') as any;
    mediaPaths.value = result || [];
  } catch (e) {
    console.error('获取媒体路径失败:', e);
  }
}

async function browseDirectory(path: string) {
  dirLoading.value = true;
  try {
    const result = await wasm.list_files_wasm(path, null, null) as DirListing;
    dirEntries.value = (result.entries || []).filter(e => e.is_dir);
    currentDirPath.value = result.path || path;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    dirLoading.value = false;
  }
}

function navigateToDir(entry: FileEntry) {
  dirBreadcrumbs.value.push(entry.path);
  browseDirectory(entry.path);
}

function navigateToBreadcrumb(index: number) {
  dirBreadcrumbs.value = dirBreadcrumbs.value.slice(0, index + 1);
  const path = dirBreadcrumbs.value[index];
  browseDirectory(path);
}

async function selectCurrentDir() {
  if (addingDir.value) return;
  addingDir.value = true;
  try {
    const token = localStorage.getItem('token');
    if (!token) {
      error.value = t('videoList.notLoggedIn');
      return;
    }

    const payload = JSON.parse(atob(token.split('.')[1]));
    const username = payload.username;

    const userDirs = await wasm.list_user_directories_wasm(username) as any;
    const dirs: any[] = userDirs || [];

    let bestDir: any = null;
    for (const dir of dirs) {
      if (currentDirPath.value.startsWith(dir.path) || dir.path.startsWith(currentDirPath.value)) {
        if (!bestDir || dir.path.length > bestDir.path.length) {
          bestDir = dir;
        }
      }
    }

    if (!bestDir) {
      error.value = t('videoList.noMatchingUserDir');
      return;
    }

    await wasm.add_media_path_wasm(bestDir.id, 'video', currentDirPath.value, currentDirPath.value.split('/').pop() || '');
    showDirPicker.value = false;
    await scanVideos();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    addingDir.value = false;
  }
}

async function deleteMediaPath(mp: MediaPathItem) {
  if (!confirm(t('videoList.confirmDeleteDir', { path: mp.path }))) return;
  try {
    await wasm.delete_media_path_wasm(mp.id);
    await fetchMediaPaths();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

function onScroll() {
  const scrollBottom = window.innerHeight + window.scrollY;
  if (scrollBottom >= document.body.offsetHeight - 300) {
    fetchVideos(false);
  }
}

onMounted(() => {
  fetchVideos(true);
  window.addEventListener('scroll', onScroll, { passive: true });
});

onBeforeUnmount(() => {
  window.removeEventListener('scroll', onScroll);
});
</script>

<template>
  <div class="p-4 max-w-[1400px] mx-auto">
    <div class="flex items-center gap-3 mb-4">
      <h2 class="flex-1 m-0 text-[1.4em]">{{ t('videoList.title') }}</h2>
      <button class="w-9 h-9 border border-[#ddd] rounded-md bg-[#1976d2] text-white text-[22px] cursor-pointer flex items-center justify-center leading-none hover:bg-[#1565c0]" @click="openDirPicker" :title="t('videoList.addVideoDirectory')">+</button>
      <button class="py-2 px-4 border border-[#ddd] rounded-md bg-white cursor-pointer text-sm hover:bg-[#f5f5f5] disabled:opacity-60 disabled:cursor-not-allowed" @click="scanVideos" :disabled="scanning">
        {{ scanning ? t('videoList.scanning') : t('videoList.scan') }}
      </button>
    </div>

    <!-- 已有视频目录 -->
    <div v-if="mediaPaths.length > 0" class="flex flex-wrap gap-1.5 mb-3">
      <div v-for="mp in mediaPaths" :key="mp.id" class="flex items-center gap-1 py-1 px-2.5 bg-[#e8f4fd] rounded text-[13px]">
        <span>{{ mp.label || mp.path }}</span>
        <button class="border-none bg-transparent text-[#999] cursor-pointer text-base px-0.5 hover:text-[#e74c3c]" @click.stop="deleteMediaPath(mp)">×</button>
      </div>
    </div>

    <div v-if="loading" class="text-center p-8 text-[#666]">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="text-center p-8 text-[#e74c3c]">{{ error }}</div>
    <div v-else-if="videos.length === 0" class="text-center p-8 text-[#666]">{{ t('videoList.noVideoPrompt') }}</div>

    <div v-else class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));">
      <div
        v-for="video in videos"
        :key="video.id"
        class="group rounded-lg overflow-hidden cursor-pointer transition-all duration-200 bg-white hover:-translate-y-0.5 hover:shadow-[0_4px_16px_rgba(0,0,0,0.1)]"
        @click="playVideo(video)"
      >
        <div class="aspect-video bg-[#1a1a2e] relative flex items-center justify-center overflow-hidden">
          <span class="text-[36px] opacity-50">🎬</span>
          <img
            :src="getPreviewUrl(video.id)"
            :alt="video.name"
            loading="lazy"
            class="w-full h-full object-cover block absolute top-0 left-0"
            @error="($event.target as HTMLImageElement).style.display='none'"
          />
          <span v-if="video.duration_ms" class="absolute bottom-1.5 right-1.5 bg-[rgba(0,0,0,0.75)] text-white py-px px-1.5 rounded text-xs tabular-nums">{{ formatDuration(video.duration_ms) }}</span>
          <div class="absolute inset-0 flex items-center justify-center bg-[rgba(0,0,0,0.3)] opacity-0 transition-opacity duration-200 group-hover:opacity-100">
            <svg class="w-12 h-12 text-white/90 drop-shadow-[0_2px_4px_rgba(0,0,0,0.3)]" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          </div>
        </div>
        <div class="py-2 px-2.5">
          <div class="font-medium text-sm overflow-hidden text-ellipsis whitespace-nowrap" :title="video.name">{{ video.name }}</div>
          <div class="flex gap-2 mt-1 text-xs text-[#999]">
            <span v-if="video.format" class="bg-[#e8f4fd] text-[#2980b9] py-px px-1.5 rounded text-[11px]">{{ video.format.toUpperCase() }}</span>
            <span>{{ formatSize(video.size) }}</span>
          </div>
        </div>
      </div>

      <div v-if="loadingMore" class="col-span-full text-center p-8 text-[#666]">{{ t('common.loadingMore') }}</div>
      <div v-else-if="hasMore" class="col-span-full text-center p-8 cursor-pointer text-[#3498db] hover:underline" @click="fetchVideos(false)">{{ t('videoList.loadMore') }}</div>
    </div>

    <!-- 目录选择弹窗 -->
    <div v-if="showDirPicker" class="fixed inset-0 bg-[rgba(0,0,0,0.5)] flex items-center justify-center z-[1000]" @click.self="showDirPicker = false">
      <div class="bg-white rounded-xl w-[90%] max-w-[500px] max-h-[80vh] flex flex-col overflow-hidden">
        <div class="flex items-center justify-between p-4 border-b border-[#eee]">
          <h3 class="m-0 text-base">{{ t('videoList.selectVideoDirectory') }}</h3>
          <button class="border-none bg-transparent text-2xl cursor-pointer text-[#999] px-1" @click="showDirPicker = false">×</button>
        </div>

        <div class="py-2 px-4 bg-[#f8f9fa] text-[13px] overflow-x-auto whitespace-nowrap">
          <span
            v-for="(crumb, i) in dirBreadcrumbs"
            :key="i"
            class="cursor-pointer text-[#3498db]"
            :class="{ 'text-[#333] font-medium': i === dirBreadcrumbs.length - 1 }"
            @click="navigateToBreadcrumb(i)"
          >
            {{ i === 0 ? t('videoList.root') : crumb.split('/').pop() || crumb }}
            <span v-if="i < dirBreadcrumbs.length - 1"> / </span>
          </span>
        </div>

        <div v-if="dirLoading" class="text-center p-8 text-[#666]">{{ t('common.loading') }}</div>
        <div v-else class="flex-1 overflow-y-auto py-2">
          <div v-if="dirEntries.length === 0" class="text-center p-8 text-[#666]">{{ t('videoList.noSubdirectories') }}</div>
          <div
            v-for="entry in dirEntries"
            :key="entry.path"
            class="flex items-center gap-2.5 py-2.5 px-4 cursor-pointer transition-colors duration-200 hover:bg-[#f8f9fa]"
            @click="navigateToDir(entry)"
          >
            <span class="text-xl">📁</span>
            <span class="text-sm">{{ entry.name }}</span>
          </div>
        </div>

        <div class="flex items-center justify-between py-3 px-4 border-t border-[#eee] gap-3">
          <span class="text-[13px] text-[#666] flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ currentDirPath }}</span>
          <button class="py-2 px-5 bg-[#1976d2] text-white border-none rounded-md cursor-pointer text-sm whitespace-nowrap disabled:opacity-60 disabled:cursor-not-allowed" @click="selectCurrentDir" :disabled="addingDir">
            {{ addingDir ? t('videoList.adding') : t('videoList.selectThisDirectory') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
