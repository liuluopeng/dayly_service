<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { scan_qr_from_image_wasm, detect_image_format } from "../types/wasm-typed";

const { t } = useI18n();

const file = ref<File | null>(null);
const previewUrl = ref("");
const scanning = ref(false);
const result = ref("");
const error = ref("");

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const f = input.files?.[0];
  if (!f) return;
  file.value = f;
  previewUrl.value = URL.createObjectURL(f);
  result.value = "";
  error.value = "";
  scanImage(f);
}

async function scanImage(f: File) {
  scanning.value = true;
  error.value = "";
  result.value = "";

  try {
    const buf = new Uint8Array(await f.arrayBuffer());
    const fmt = detect_image_format(buf);
    if (!fmt) {
      error.value = t('common.error.loadFailed');
      return;
    }
    const text = scan_qr_from_image_wasm(buf);
    result.value = text;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (msg.includes("NoQrFound") || msg.includes("未检测到")) {
      error.value = t('menu.tools.qrscan.noQr');
    } else {
      error.value = msg;
    }
  } finally {
    scanning.value = false;
  }
}

function copyResult() {
  if (!result.value) return;
  navigator.clipboard.writeText(result.value);
}
</script>

<template>
  <div class="p-4 max-w-xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('menu.tools.qrscan.title') }}</h1>

    <div class="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center mb-4"
         :class="{ 'border-green-400': result }">
      <input type="file" accept="image/*" @change="onFileChange" class="hidden" id="scanInput" />
      <label for="scanInput" class="cursor-pointer block">
        <div class="text-4xl mb-2">📷</div>
        <p class="text-gray-500">{{ file ? file.name : (t('menu.tools.qrscan.select') + '...') }}</p>
      </label>
    </div>

    <div v-if="previewUrl" class="mb-4 flex justify-center">
      <img :src="previewUrl" class="max-h-64 rounded border" />
    </div>

    <div v-if="scanning" class="text-center py-4 text-gray-500">
      {{ t('common.loading') }}...
    </div>

    <div v-else-if="result"
         class="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
      <p class="text-sm font-medium text-green-800 mb-2">{{ t('menu.tools.qrscan.result') }}</p>
      <p class="text-sm break-all mb-2">{{ result }}</p>
      <button @click="copyResult"
              class="px-3 py-1 text-xs bg-green-500 text-white rounded">
        {{ t('common.copy') }}
      </button>
    </div>

    <div v-if="error"
         class="bg-red-100 border border-red-300 text-red-700 px-4 py-3 rounded mb-4">
      {{ error }}
    </div>
  </div>
</template>
