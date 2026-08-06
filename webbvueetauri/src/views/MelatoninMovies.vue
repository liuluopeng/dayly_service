<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import * as wasm from '../types/wasm-typed';

interface MelatoninMovie {
  id: string;
  title: string;
  cover_url: string | null;
  video_urls: string[];
}

interface PaginatedMelatonin {
  data: MelatoninMovie[];
  total: number;
  page: number;
  total_pages: number;
}

const movies = ref<MelatoninMovie[]>([]);
const loading = ref(false);
const loadingMore = ref(false);
const error = ref<string | null>(null);
const currentPage = ref(0);
const hasMore = ref(true);
const pageSize = 24;

const baseUrl = wasm.get_base_url_wasm();
const token = localStorage.getItem('token') || '';

function coverSrc(movie: MelatoninMovie): string {
  if (!movie.cover_url) return '';
  const url = movie.cover_url.startsWith('http') ? movie.cover_url : `${baseUrl}${movie.cover_url}`;
  return `${url}?token=${encodeURIComponent(token)}`;
}

function playMovie(movie: MelatoninMovie) {
  const url = movie.video_urls && movie.video_urls.length > 0 ? movie.video_urls[0] : null;
  if (!url) return;
  const full = url.startsWith('http') ? url : `${baseUrl}${url}`;
  window.open(`${full}?token=${encodeURIComponent(token)}`, '_blank');
}

async function fetchMovies(reset = false) {
  if (reset) {
    loading.value = true;
    movies.value = [];
    currentPage.value = 0;
    hasMore.value = true;
  } else {
    if (!hasMore.value || loadingMore.value) return;
    loadingMore.value = true;
  }
  error.value = null;
  try {
    const page = reset ? 1 : currentPage.value + 1;
    const res = await fetch(`${baseUrl}/api/melatonin/list?page=${page}&page_size=${pageSize}`, {
      headers: { Authorization: `Bearer ${token}` }
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const result = await res.json() as PaginatedMelatonin;
    const data = result.data || [];
    if (reset) movies.value = data;
    else movies.value.push(...data);
    currentPage.value = result.page;
    hasMore.value = result.page < result.total_pages;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function onScroll() {
  const scrollBottom = window.innerHeight + window.scrollY;
  if (scrollBottom >= document.body.offsetHeight - 400) {
    fetchMovies(false);
  }
}

onMounted(() => {
  fetchMovies(true);
  window.addEventListener('scroll', onScroll, { passive: true });
});

onBeforeUnmount(() => {
  window.removeEventListener('scroll', onScroll);
});
</script>

<template>
  <div class="p-4 max-w-[1400px] mx-auto">
    <div class="flex items-center gap-3 mb-4">
      <h2 class="flex-1 m-0 text-[1.4em]">褪黑素电影</h2>
    </div>

    <div v-if="loading" class="text-center p-8 text-[#666]">加载中...</div>
    <div v-else-if="error" class="text-center p-8 text-[#e74c3c]">{{ error }}</div>
    <div v-else-if="movies.length === 0" class="text-center p-8 text-[#666]">暂无电影，请先在服务端扫描褪黑素目录</div>

    <div v-else class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));">
      <div
        v-for="movie in movies"
        :key="movie.id"
        class="group rounded-lg overflow-hidden cursor-pointer transition-all duration-200 bg-white hover:-translate-y-0.5 hover:shadow-[0_4px_16px_rgba(0,0,0,0.1)]"
        @click="playMovie(movie)"
        :title="movie.title"
      >
        <div class="aspect-[2/3] bg-[#1a1a2e] relative flex items-center justify-center overflow-hidden">
          <span class="text-[36px] opacity-50">🎬</span>
          <img
            v-if="coverSrc(movie)"
            :src="coverSrc(movie)"
            :alt="movie.title"
            loading="lazy"
            class="w-full h-full object-cover block absolute top-0 left-0"
            @error="($event.target as HTMLImageElement).style.display='none'"
          />
          <div class="absolute inset-0 flex items-center justify-center bg-[rgba(0,0,0,0.3)] opacity-0 transition-opacity duration-200 group-hover:opacity-100">
            <svg class="w-12 h-12 text-white/90 drop-shadow-[0_2px_4px_rgba(0,0,0,0.3)]" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          </div>
        </div>
        <div class="py-2 px-2.5">
          <div class="font-medium text-sm overflow-hidden text-ellipsis whitespace-nowrap">{{ movie.title }}</div>
        </div>
      </div>

      <div v-if="loadingMore" class="col-span-full text-center p-8 text-[#666]">加载更多...</div>
      <div v-else-if="hasMore" class="col-span-full text-center p-8 cursor-pointer text-[#3498db] hover:underline" @click="fetchMovies(false)">加载更多</div>
    </div>
  </div>
</template>
