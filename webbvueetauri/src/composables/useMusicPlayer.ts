import { ref, computed } from 'vue';
import * as wasm from '../types/wasm-typed';
import type { SongWithUrl, PaginatedResponse } from '../types/models';

const playlist = ref<SongWithUrl[]>([]);
const currentIndex = ref(-1);
const totalSongs = ref(0);
const pageSize = 20;
const loadedPages = ref<Set<number>>(new Set());

const currentSong = computed(() => {
  if (currentIndex.value >= 0 && currentIndex.value < playlist.value.length) {
    return playlist.value[currentIndex.value];
  }
  return null;
});

const hasNext = computed(() => {
  return currentIndex.value < totalSongs.value - 1;
});

const hasPrevious = computed(() => {
  return currentIndex.value > 0;
});

async function loadPageIfNeeded(page: number): Promise<void> {
  if (loadedPages.value.has(page)) return;
  
  try {
    const result = await wasm.get_all_songs_wasm(page, pageSize) as PaginatedResponse<SongWithUrl>;
    const startIdx = (page - 1) * pageSize;
    
    for (let i = 0; i < result.data.length; i++) {
      playlist.value[startIdx + i] = result.data[i];
    }
    
    loadedPages.value.add(page);
  } catch (error) {
    console.error('加载歌曲页失败:', error);
  }
}

export function useMusicPlayer() {
  const setPlaylist = async (songs: SongWithUrl[], startIndex: number, total: number, startPage: number) => {
    playlist.value = new Array(total);
    totalSongs.value = total;
    currentIndex.value = startIndex;
    loadedPages.value.clear();
    
    const startIdx = (startPage - 1) * pageSize;
    for (let i = 0; i < songs.length; i++) {
      playlist.value[startIdx + i] = songs[i];
    }
    loadedPages.value.add(startPage);
  };

  const playSong = (index: number): SongWithUrl | null => {
    if (index >= 0 && index < totalSongs.value) {
      currentIndex.value = index;
      return playlist.value[index];
    }
    return null;
  };

  const next = async (): Promise<SongWithUrl | null> => {
    if (!hasNext.value) return null;
    
    const nextIdx = currentIndex.value + 1;
    const pageNeeded = Math.floor(nextIdx / pageSize) + 1;
    
    await loadPageIfNeeded(pageNeeded);
    
    currentIndex.value = nextIdx;
    return playlist.value[nextIdx];
  };

  const previous = async (): Promise<SongWithUrl | null> => {
    if (!hasPrevious.value) return null;
    
    const prevIdx = currentIndex.value - 1;
    const pageNeeded = Math.floor(prevIdx / pageSize) + 1;
    
    await loadPageIfNeeded(pageNeeded);
    
    currentIndex.value = prevIdx;
    return playlist.value[prevIdx];
  };

  const reset = () => {
    playlist.value = [];
    currentIndex.value = -1;
    totalSongs.value = 0;
    loadedPages.value.clear();
  };

  return {
    playlist,
    currentIndex,
    totalSongs,
    currentSong,
    hasNext,
    hasPrevious,
    setPlaylist,
    playSong,
    next,
    previous,
    reset
  };
}
