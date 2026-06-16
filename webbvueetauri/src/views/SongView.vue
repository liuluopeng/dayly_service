<script setup lang="ts">
import { ref, computed, onMounted,  } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import * as wasm from '../types/wasm-typed';
import { useMusicPlayer } from '../composables/useMusicPlayer';
import { useAuth } from '../composables/useAuth';
import type { SongWithUrl, PaginatedResponse } from '../types/models';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { setPlaylist } = useMusicPlayer();
const { logout } = useAuth();

const isWasmMode = computed(() => route.query.wasm === '1');
const isLyricsMode = computed(() => route.query.lyrics === '1');

function handleLogout() {
  logout();
  router.push('/login');
}

const playSong = async (song: SongWithUrl, index: number) => {
  await setPlaylist(songs.value, index, totalSongs.value, 1);
  router.push({
    path: isWasmMode.value ? '/player-wasm' : isLyricsMode.value ? '/player-lyrics' : '/player',
    query: { song: encodeURIComponent(JSON.stringify(song)) }
  });
};

const songs = ref<SongWithUrl[]>([]);
const totalSongs = ref(0);
const loading = ref(false);
const loadingMore = ref(false);
const error = ref<string | null>(null);
const currentPage = ref(0);
const hasMore = ref(true);
const pageSize = 48;

async function fetchSongs(reset = false) {
  if (reset) {
    loading.value = true;
    songs.value = [];
    currentPage.value = 0;
    hasMore.value = true;
  } else {
    if (!hasMore.value || loadingMore.value) return;
    loadingMore.value = true;
  }
  error.value = null;

  try {
    const page = reset ? 1 : currentPage.value + 1;
    const result = await wasm.get_all_songs_wasm(page, pageSize) as PaginatedResponse<SongWithUrl>;
    const newSongs = result.data || [];
    if (reset) {
      songs.value = newSongs;
    } else {
      songs.value.push(...newSongs);
    }
    totalSongs.value = result.total;
    currentPage.value = result.page;
    hasMore.value = result.page < result.total_pages;
  } catch (err) {
    error.value = t('songView.fetchFailed', { error: String(err) });
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function onScroll(e: Event) {
  const el = e.target as HTMLElement;
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
    fetchSongs();
  }
}

onMounted(() => {
  fetchSongs(true);
});
</script>

<template>
  <div class="h-screen bg-gray-950 flex flex-col overflow-hidden">
    <!-- 顶栏 -->
    <header class="sticky top-0 z-20 bg-gray-950/80 backdrop-blur-xl border-b border-white/10">
      <div class="mx-auto px-6 py-3 flex justify-between items-center max-w-[1800px]">
        <h1 class="text-lg font-bold text-white">
          {{ isWasmMode ? '⚡ ' + t('songView.wasmTitle') : isLyricsMode ? '🎶 ' + t('songView.lyricsTitle') : '🎵 ' + t('songView.musicTitle') }}
        </h1>
        <div class="flex items-center gap-3">
          <span v-if="totalSongs > 0" class="text-xs text-white/30">{{ totalSongs }} {{ t('songView.songsUnit') }}</span>
          <button @click="() => fetchSongs(true)" :disabled="loading"
            class="px-3 py-1.5 text-xs bg-white/10 text-white/80 rounded-full hover:bg-white/20 transition-colors disabled:opacity-30">
            {{ t('common.refresh') }}
          </button>
          <button @click="handleLogout"
            class="px-3 py-1.5 text-xs bg-red-500/30 text-red-300 rounded-full hover:bg-red-500/50 transition-colors">
            {{ t('songView.logout') }}
          </button>
        </div>
      </div>
    </header>

    <!-- 可滚动区域 -->
    <div class="flex-1 overflow-y-auto" @scroll="onScroll">
      <div class="mx-auto px-6 py-6 max-w-[1800px]">

        <div v-if="error" class="mb-6 p-3 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm">
          {{ error }}
        </div>

        <div v-if="songs.length === 0 && loading" class="flex items-center justify-center h-64 text-white/20 text-lg">
          {{ t('common.loading') }}
        </div>

        <div v-else-if="songs.length === 0" class="flex items-center justify-center h-64 text-white/20 text-lg">
          {{ t('songView.noSongs') }}
        </div>

        <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-4">
          <div v-for="(song, index) in songs" :key="song.id"
            @click="playSong(song, index)"
            class="group cursor-pointer rounded-xl overflow-hidden bg-white/5 hover:bg-white/10 transition-all duration-200 hover:scale-[1.02] active:scale-95">
            <div class="aspect-square relative overflow-hidden">
              <img v-if="song.cover_url" :src="song.cover_url" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
              <div v-else
                class="w-full h-full flex items-center justify-center"
                :style="{
                  background: `linear-gradient(135deg, hsl(${(index * 47) % 360}, 70%, 30%), hsl(${((index * 47) + 60) % 360}, 70%, 20%))`
                }">
                <svg class="w-12 h-12 text-white/20" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
                </svg>
              </div>
              <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                <div class="w-12 h-12 rounded-full bg-white/90 flex items-center justify-center shadow-2xl">
                  <svg class="w-5 h-5 text-gray-900 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M8 5v14l11-7z" />
                  </svg>
                </div>
              </div>
            </div>
            <div class="p-3">
              <p class="text-sm font-medium text-white truncate">{{ song.title }}</p>
              <p class="text-xs text-white/40 truncate mt-0.5">{{ song.artist || t('songView.unknownArtist') }}</p>
            </div>
          </div>
        </div>

        <!-- 加载更多指示器 -->
        <div v-if="loadingMore" class="py-6 text-center text-white/30 text-sm">
          {{ t('common.loadingMore') }}
        </div>
        <div v-else-if="!hasMore && songs.length > 0" class="py-6 text-center text-white/20 text-xs">
          {{ t('songView.allSongsLoaded', { total: totalSongs }) }}
        </div>
      </div>
    </div>
  </div>
</template>
