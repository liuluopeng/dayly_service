<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import * as pdfjsLib from 'pdfjs-dist';
import pdfjsWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker;

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const loading = ref(true);
const error = ref('');
const pdfPath = ref('');
const fileName = ref('');
const totalPages = ref(0);
const currentPage = ref(1);
const scale = ref(1.5);
const containerRef = ref<HTMLDivElement | null>(null);

let pdfDoc: pdfjsLib.PDFDocumentProxy | null = null;

function getApiUrl(): string {
  if (import.meta.env.DEV) {
    return `http://${window.location.hostname}:23001`;
  }
  return `http://${window.location.hostname}:23000`;
}

function getAuthUrl(path: string): string {
  const token = localStorage.getItem('token');
  const apiUrl = getApiUrl();
  return `${apiUrl}/api/files/serve?path=${encodeURIComponent(path)}&token=${token}`;
}

async function loadPdf() {
  loading.value = true;
  error.value = '';

  try {
    const url = getAuthUrl(pdfPath.value);
    const loadingTask = pdfjsLib.getDocument(url);
    pdfDoc = await loadingTask.promise;
    totalPages.value = pdfDoc.numPages;
    currentPage.value = 1;
    await renderPage(1);
  } catch (e) {
    error.value = e instanceof Error ? e.message : t('pdfViewer.loadFailed');
  } finally {
    loading.value = false;
  }
}

async function renderPage(pageNum: number) {
  if (!pdfDoc) return;

  const page = await pdfDoc.getPage(pageNum);
  const viewport = page.getViewport({ scale: scale.value });

  await nextTick();
  const container = containerRef.value;
  if (!container) return;

  // 清空容器
  container.innerHTML = '';

  const canvas = document.createElement('canvas');
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  canvas.style.display = 'block';
  canvas.style.margin = '0 auto';
  container.appendChild(canvas);

  const ctx = canvas.getContext('2d')!;
  await page.render({ canvas, canvasContext: ctx, viewport }).promise;
}

function goToPage(page: number) {
  if (page < 1 || page > totalPages.value) return;
  currentPage.value = page;
  renderPage(page);
}

function prevPage() {
  if (currentPage.value > 1) goToPage(currentPage.value - 1);
}

function nextPage() {
  if (currentPage.value < totalPages.value) goToPage(currentPage.value + 1);
}

function zoomIn() {
  scale.value = Math.min(scale.value + 0.25, 4);
  renderPage(currentPage.value);
}

function zoomOut() {
  scale.value = Math.max(scale.value - 0.25, 0.5);
  renderPage(currentPage.value);
}

function fitWidth() {
  if (!containerRef.value) return;
  const containerWidth = containerRef.value.clientWidth - 40;
  // 估算：标准 A4 宽度约 595pt
  scale.value = Math.round((containerWidth / 595) * 100) / 100;
  renderPage(currentPage.value);
}

onMounted(() => {
  const path = route.query.path as string;
  if (!path) {
    error.value = t('pdfViewer.missingPath');
    loading.value = false;
    return;
  }
  pdfPath.value = path;
  fileName.value = path.split('/').pop() || 'PDF';
  loadPdf();
});
</script>

<template>
  <div class="h-screen flex flex-col bg-[#525659]">
    <div class="flex items-center gap-2 py-2 px-4 bg-[#323639] text-white shrink-0">
      <button
        class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
        @click="router.back()" :title="t('common.back')"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 12H5M12 19l-7-7 7-7"/>
        </svg>
      </button>
      <span class="text-sm overflow-hidden text-ellipsis whitespace-nowrap max-w-[300px]">{{ fileName }}</span>
      <div class="flex-1" />
      <button
        class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
        @click="zoomOut" :title="t('pdfViewer.zoomOut')"
      >A-</button>
      <span class="text-xs min-w-[40px] text-center">{{ Math.round(scale * 100) }}%</span>
      <button
        class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
        @click="zoomIn" :title="t('pdfViewer.zoomIn')"
      >A+</button>
      <button
        class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
        @click="fitWidth" :title="t('pdfViewer.fitWidth')"
      >{{ t('pdfViewer.fitWidthLabel') }}</button>
      <div class="flex items-center gap-1.5 text-[13px]">
        <button
          class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
          @click="prevPage" :disabled="currentPage <= 1"
        >&lt;</button>
        <input
          type="number"
          :value="currentPage"
          @change="goToPage(Number(($event.target as HTMLInputElement).value))"
          :min="1"
          :max="totalPages"
          class="w-12 text-center bg-[#222] border border-[#555] text-white rounded py-[3px] text-[13px]"
        />
        <span>/ {{ totalPages }}</span>
        <button
          class="bg-transparent border border-[#555] text-[#ddd] py-1 px-2.5 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#444] disabled:opacity-40 disabled:cursor-default"
          @click="nextPage" :disabled="currentPage >= totalPages"
        >&gt;</button>
      </div>
    </div>
    <div class="flex-1 overflow-auto p-5" ref="containerRef">
      <div v-if="loading" class="text-center p-10 text-[#aaa] text-base">{{ t('common.loading') }}</div>
      <div v-if="error" class="text-center p-10 text-[#f66] text-base">{{ error }}</div>
    </div>
  </div>
</template>
