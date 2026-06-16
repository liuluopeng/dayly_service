<template>
  <div class="h-screen flex flex-col bg-white text-[#333]" @contextmenu.prevent>
    <div class="px-4 py-3 border-b border-[#e0e0e0]">
      <div class="flex items-center gap-3 mb-2">
        <button class="bg-[#f5f5f5] text-[#1976d2] border border-[#ddd] px-3 py-1 rounded cursor-pointer text-[13px] hover:bg-[#e3f2fd] disabled:text-[#bbb] disabled:cursor-default disabled:hover:bg-[#f5f5f5]" :disabled="currentPath === '/' || currentPath === ''" @click="goBack">
          &larr; {{ t('common.back') }}
        </button>
        <h2 class="m-0 text-lg flex-1">{{ t('fileManager.title') }}</h2>
        <div class="flex gap-0.5 bg-[#f5f5f5] rounded p-0.5">
          <button class="border-none px-2.5 py-1 rounded-[3px] cursor-pointer text-base leading-none" :class="viewMode === 'list' ? 'bg-white text-[#1976d2] shadow-[0_1px_3px_rgba(0,0,0,0.1)]' : 'bg-transparent text-[#888] hover:text-[#333]'" @click="viewMode = 'list'" :title="t('fileManager.listView')">&#9776;</button>
          <button class="border-none px-2.5 py-1 rounded-[3px] cursor-pointer text-base leading-none" :class="viewMode === 'grid' ? 'bg-white text-[#1976d2] shadow-[0_1px_3px_rgba(0,0,0,0.1)]' : 'bg-transparent text-[#888] hover:text-[#333]'" @click="viewMode = 'grid'" :title="t('fileManager.gridView')">&#8862;</button>
        </div>
      </div>
      <div class="text-sm text-[#666]">
        <span class="cursor-pointer text-[#1976d2] hover:underline" @click="navigateTo('/')">/</span>
        <template v-for="(part, i) in pathParts" :key="i">
          <span class="mx-0.5">/</span>
          <span class="cursor-pointer text-[#1976d2] hover:underline" @click="navigateToPath(i)">{{ part }}</span>
        </template>
      </div>
    </div>

    <div v-if="isLoading" class="p-6 text-center text-[#888]">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="p-6 text-center text-[#e53935]">{{ error }}</div>
    <div v-else class="flex-1 flex overflow-hidden">
      <!-- 列表视图 -->
      <div v-if="viewMode === 'list'" class="flex-1 overflow-y-auto min-w-0" @scroll="onScroll">
        <div
          v-for="entry in entries"
          :key="entry.path"
          class="flex items-center px-4 py-2 cursor-pointer gap-2 border-b border-[#f0f0f0]"
          :class="selectedFile?.path === entry.path ? 'bg-[#e3f2fd]' : 'hover:bg-[#f5f5f5]'"
          @click="selectFile(entry)"
          @dblclick="entry.is_dir ? navigateTo(entry.path) : openFile(entry)"
          @contextmenu.stop.prevent="onContextMenu($event, entry)"
        >
          <span class="text-lg w-6 text-center">{{ entry.is_dir ? '📁' : getFileIcon(entry.name) }}</span>
          <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ entry.name }}</span>
          <span class="text-[#888] text-[13px] min-w-[70px] text-right">{{ entry.is_dir ? '' : formatSize(entry.size) }}</span>
          <span class="text-[#999] text-xs min-w-[140px] text-right">{{ entry.last_modified || '' }}</span>
        </div>
        <div v-if="isLoadingMore" class="p-3 text-center text-[#1976d2] text-[13px]">{{ t('common.loadingMore') }}</div>
        <div v-if="entries.length === 0" class="p-6 text-center text-[#888]">{{ t('common.emptyDirectory') }}</div>
      </div>

      <!-- 网格视图 -->
      <div v-else class="flex-1 min-w-0 overflow-y-auto p-4 flex flex-wrap gap-4 content-start" @scroll="onScroll">
        <div
          v-for="entry in entries"
          :key="entry.path"
          class="w-[200px] bg-[#fafafa] rounded-lg overflow-hidden cursor-pointer border-2 shrink-0 transition-colors duration-150"
          :class="selectedFile?.path === entry.path ? 'border-[#1976d2]' : 'border-transparent hover:border-[#ddd]'"
          @click="selectFile(entry)"
          @dblclick="entry.is_dir ? navigateTo(entry.path) : openFile(entry)"
          @contextmenu.stop.prevent="onContextMenu($event, entry)"
        >
          <div class="w-full aspect-square bg-[#f0f0f0] flex items-center justify-center overflow-hidden">
            <img
              v-if="!entry.is_dir && getFileType(entry.name) === 'image'"
              :src="getAuthUrl(entry.path)"
              class="w-full h-full object-cover"
              loading="lazy"
            />
            <video
              v-else-if="!entry.is_dir && getFileType(entry.name) === 'video'"
              :src="getAuthUrl(entry.path)"
              class="w-full h-full object-cover"
              preload="metadata"
              muted
              @loadeddata="onVideoThumbLoaded"
            />
            <span v-else class="text-[36px]">{{ entry.is_dir ? '📁' : getFileIcon(entry.name) }}</span>
          </div>
          <div class="px-2 pt-1.5 pb-0.5 text-xs overflow-hidden text-ellipsis whitespace-nowrap" :title="entry.name">{{ entry.name }}</div>
          <div class="px-2 pb-1.5 text-[11px] text-[#888]">{{ entry.is_dir ? '' : formatSize(entry.size) }}</div>
        </div>
        <div v-if="isLoadingMore" class="p-3 text-center text-[#1976d2] text-[13px]">{{ t('common.loadingMore') }}</div>
        <div v-if="entries.length === 0" class="p-6 text-center text-[#888]">{{ t('common.emptyDirectory') }}</div>
      </div>

      <!-- 预览面板 -->
      <div v-if="selectedFile && !selectedFile.is_dir" class="w-1/2 min-w-[400px] border-l border-[#e0e0e0] flex flex-col overflow-hidden">
        <div class="px-3 py-2 border-b border-[#e0e0e0] flex justify-between items-center text-[13px]">
          <span class="overflow-hidden text-ellipsis whitespace-nowrap flex-1 mr-2">{{ selectedFile.name }}</span>
          <a :href="getAuthUrl(selectedFile.path)" target="_blank" class="text-[#1976d2] no-underline text-xs whitespace-nowrap">{{ t('common.newWindowOpen') }}</a>
        </div>
        <div class="flex-1 overflow-auto flex items-center justify-center p-2 bg-[#fafafa]">
          <img
            v-if="fileType === 'image'"
            :src="getAuthUrl(selectedFile.path)"
            class="max-w-full max-h-full object-contain"
          />
          <audio
            v-else-if="fileType === 'audio'"
            :src="getAuthUrl(selectedFile.path)"
            controls
            autoplay
            class="w-[90%]"
          />
          <iframe
            v-else-if="fileType === 'pdf'"
            :src="getAuthUrl(selectedFile.path)"
            class="w-full h-full border-none"
          />
          <pre v-else-if="fileType === 'text'" class="w-full h-full overflow-auto text-xs font-['LXGWWenKaiMono',monospace] whitespace-pre-wrap break-all m-0 p-2 bg-white text-[#333]">{{ textContent }}</pre>
          <div v-else class="text-[#999] text-sm">{{ t('common.unsupportedPreview') }}</div>
        </div>
      </div>
    </div>

    <!-- 自定义右键菜单 -->
    <div
      v-if="contextMenu.show"
      class="fixed bg-white border border-[#ddd] rounded-md shadow-[0_4px_12px_rgba(0,0,0,0.15)] min-w-[160px] z-[1000] py-1"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    >
      <div
        v-if="contextMenu.entry?.is_dir"
        class="px-4 py-2 cursor-pointer text-sm text-[#333] hover:bg-[#e3f2fd]"
        @click="generateTree(contextMenu.entry!)"
      >
        &#127795; {{ t('common.generateTree') }}
      </div>
      <div class="px-4 py-2 cursor-pointer text-sm text-[#333] hover:bg-[#e3f2fd]" @click="contextMenu.show = false">
        {{ t('common.cancel') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useFiles, type FileEntry } from '../composables/useFiles';
import { generate_tree_wasm } from '../types/wasm-typed';

const { t } = useI18n();

const {
  entries,
  currentPath,
  isLoading,
  isLoadingMore,
  error,
  fetchDirectory,
  loadMore,
  getFileUrl,
  getAuthenticatedFileUrl,
  formatSize,
  getFileType,
  navigateTo,
} = useFiles();

const router = useRouter();

const selectedFile = ref<FileEntry | null>(null);
const textContent = ref('');
const viewMode = ref<'list' | 'grid'>('list');

const contextMenu = ref<{ show: boolean; x: number; y: number; entry: FileEntry | null }>({
  show: false,
  x: 0,
  y: 0,
  entry: null,
});

const pathParts = computed(() => {
  const p = currentPath.value.replace(/^\//, '').replace(/\/$/, '');
  return p ? p.split('/') : [];
});

const fileType = computed(() => {
  if (!selectedFile.value) return '';
  return getFileType(selectedFile.value.name);
});

function getAuthUrl(path: string): string {
  return getAuthenticatedFileUrl(path);
}

function navigateToPath(index: number) {
  const parts = pathParts.value.slice(0, index + 1);
  navigateTo('/' + parts.join('/'));
}

function goBack() {
  const parts = pathParts.value;
  if (parts.length <= 1) {
    navigateTo('/');
  } else {
    navigateTo('/' + parts.slice(0, -1).join('/'));
  }
}

function openVideo(path: string) {
  router.push({ path: '/video', query: { path: encodeURIComponent(path) } });
}

function selectFile(entry: FileEntry) {
  const type = getFileType(entry.name);
  if (!entry.is_dir && type === 'video') {
    openVideo(entry.path);
    return;
  }
  if (!entry.is_dir && type === 'epub') {
    router.push({ path: '/epub', query: { path: entry.path } });
    return;
  }
  if (!entry.is_dir && type === 'pdf') {
    router.push({ path: '/pdf', query: { path: entry.path } });
    return;
  }
  selectedFile.value = entry;
  if (type === 'text' && !entry.is_dir) {
    loadTextPreview(entry.path);
  }
}

function openFile(entry: FileEntry) {
  const type = getFileType(entry.name);
  if (type === 'video') {
    openVideo(entry.path);
  } else if (type === 'epub') {
    router.push({ path: '/epub', query: { path: entry.path } });
  } else if (type === 'pdf') {
    router.push({ path: '/pdf', query: { path: entry.path } });
  } else {
    window.open(getAuthenticatedFileUrl(entry.path), '_blank');
  }
}

async function loadTextPreview(path: string) {
  try {
    const res = await fetch(getFileUrl(path));
    if (res.ok) {
      textContent.value = await res.text();
    }
  } catch {
    textContent.value = t('fileManager.loadContentFailed');
  }
}

function getFileIcon(name: string): string {
  const type = getFileType(name);
  switch (type) {
    case 'video': return '🎬';
    case 'image': return '🖼️';
    case 'audio': return '🎵';
    case 'pdf': return '📄';
    case 'epub': return '📖';
    case 'text': return '📝';
    default: return '📄';
  }
}

function onVideoThumbLoaded(e: Event) {
  const video = e.target as HTMLVideoElement;
  video.currentTime = 1;
}

function onScroll(e: Event) {
  const el = e.target as HTMLElement;
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
    loadMore();
  }
}

function onContextMenu(e: MouseEvent, entry: FileEntry) {
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    entry,
  };
}

function closeContextMenu() {
  contextMenu.value.show = false;
}

async function generateTree(entry: FileEntry) {
  closeContextMenu();
  try {
    const result = await generate_tree_wasm(entry.path);
    alert(t('common.treeGenerated', { filename: result.filename }));
    fetchDirectory(currentPath.value);
  } catch (err) {
    alert(t('common.treeFailed', { error: String(err) }));
  }
}

onMounted(() => {
  fetchDirectory('/');
  document.addEventListener('click', closeContextMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu);
});

watch(currentPath, () => {
  selectedFile.value = null;
  textContent.value = '';
});
</script>
