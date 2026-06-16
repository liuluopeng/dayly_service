<template>
  <div class="h-screen flex flex-col">
    <iframe
      v-if="readerUrl"
      :src="readerUrl"
      class="flex-1 w-full border-0"
      allow="fullscreen"
    />
    <div v-else-if="error" class="flex-1 flex items-center justify-center text-red-400">
      {{ error }}
    </div>
    <div v-else class="flex-1 flex items-center justify-center text-gray-400">
      {{ t('common.loading') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { get_file_url_wasm } from '../types/wasm-typed';

const { t } = useI18n();
const route = useRoute();

const readerUrl = ref('');
const error = ref('');

onMounted(async () => {
  const path = route.query.path as string;
  if (!path) {
    error.value = t('epubReader.missingPath');
    return;
  }

  const apiUrl = get_file_url_wasm(path);

  try {
    const res = await fetch(apiUrl);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const blob = await res.blob();
    const blobUrl = URL.createObjectURL(blob);

    const foliateBase = `${window.location.origin}/foliate/reader.html`;
    readerUrl.value = `${foliateBase}?url=${encodeURIComponent(blobUrl)}`;
  } catch (e) {
    console.error('Failed to load EPUB:', e);
    error.value = t('epubReader.openFailed');
  }
});
</script>
