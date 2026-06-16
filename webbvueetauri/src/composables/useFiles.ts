import { ref } from 'vue';
import { list_files_wasm, get_file_url_wasm } from '../types/wasm-typed';
import type { FileEntry, DirListing } from '../types/models';
export type { FileEntry, DirListing } from '../types/models';

export function useFiles() {
  const entries = ref<FileEntry[]>([]);
  const currentPath = ref('');
  const isLoading = ref(false);
  const isLoadingMore = ref(false);
  const error = ref('');
  const total = ref(0);
  const currentPage = ref(1);
  const hasMore = ref(true);

  async function fetchDirectory(path: string = '') {
    isLoading.value = true;
    error.value = '';
    entries.value = [];
    currentPage.value = 1;
    hasMore.value = true;
    try {
      const listing = await list_files_wasm(path, 1, 200) as DirListing;
      entries.value = listing.entries;
      currentPath.value = listing.path;
      total.value = listing.total;
      hasMore.value = listing.entries.length < listing.total;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      entries.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMore() {
    if (isLoadingMore.value || !hasMore.value) return;
    isLoadingMore.value = true;
    try {
      currentPage.value++;
      const listing = await list_files_wasm(currentPath.value, currentPage.value, 200) as DirListing;
      entries.value.push(...listing.entries);
      hasMore.value = entries.value.length < listing.total;
    } catch (e) {
      // ignore load-more errors
    } finally {
      isLoadingMore.value = false;
    }
  }

  function getFileUrl(path: string): string {
    return get_file_url_wasm(path);
  }

  function getAuthenticatedFileUrl(path: string): string {
    return get_file_url_wasm(path);
  }

  function formatSize(size: number): string {
    if (size === 0) return '';
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function getFileType(name: string): string {
    const ext = name.split('.').pop()?.toLowerCase() || '';
    if (['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'm4v', 'webm'].includes(ext)) return 'video';
    if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(ext)) return 'image';
    if (['mp3', 'flac', 'wav', 'ogg', 'aac', 'm4a'].includes(ext)) return 'audio';
    if (['pdf'].includes(ext)) return 'pdf';
    if (['epub'].includes(ext)) return 'epub';
    if (['txt', 'md', 'log', 'nfo', 'srt', 'ass', 'json', 'xml', 'csv'].includes(ext)) return 'text';
    return 'other';
  }

  function navigateTo(path: string) {
    fetchDirectory(path);
  }

  return {
    entries,
    currentPath,
    isLoading,
    isLoadingMore,
    error,
    total,
    hasMore,
    fetchDirectory,
    loadMore,
    getFileUrl,
    getAuthenticatedFileUrl,
    formatSize,
    getFileType,
    navigateTo,
  };
}
