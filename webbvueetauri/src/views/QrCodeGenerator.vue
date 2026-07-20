<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { generate_qr_png_wasm, qr_info_wasm } from "../types/wasm-typed";

const { t } = useI18n();

const text = ref("");
const scale = ref(10);
const margin = ref(2);
const generating = ref(false);
const qrUrl = ref("");
const qrVersion = ref(0);
const error = ref("");

const scalePresets = [
  { label: "S", value: 6 },
  { label: "M", value: 10 },
  { label: "L", value: 16 },
  { label: "XL", value: 24 },
];

async function generate() {
  if (!text.value.trim()) return;
  generating.value = true;
  error.value = "";
  qrUrl.value = "";

  try {
    const png = generate_qr_png_wasm(text.value, scale.value, margin.value);
    const blob = new Blob([png], { type: "image/png" });
    qrUrl.value = URL.createObjectURL(blob);

    const info = qr_info_wasm(text.value);
    qrVersion.value = info[0];
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    generating.value = false;
  }
}

function download() {
  if (!qrUrl.value) return;
  const a = document.createElement("a");
  a.href = qrUrl.value;
  a.download = "qrcode.png";
  a.click();
}

function handlePaste(e: ClipboardEvent) {
  const pasted = e.clipboardData?.getData("text");
  if (pasted && !text.value) {
    text.value = pasted;
    generate();
  }
}
</script>

<template>
  <div class="p-4 max-w-xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('menu.tools.qrcode.title') }}</h1>

    <div class="space-y-4">
      <div>
        <label class="text-sm text-gray-600 block mb-1">{{ t('menu.tools.qrcode.input') }}</label>
        <textarea
          v-model="text"
          :placeholder="t('menu.tools.qrcode.placeholder')"
          rows="4"
          class="w-full px-3 py-2 border rounded text-sm resize-none"
          @paste="handlePaste"
        ></textarea>
      </div>

      <div class="flex gap-4 items-end">
        <div>
          <label class="text-sm text-gray-600 block mb-1">{{ t('menu.tools.qrcode.size') }}: {{ scale }}</label>
          <div class="flex gap-1">
            <button v-for="p in scalePresets" :key="p.value"
                    @click="scale = p.value"
                    class="px-3 py-1 text-xs rounded"
                    :class="scale === p.value ? 'bg-blue-500 text-white' : 'bg-gray-200'">
              {{ p.label }}
            </button>
          </div>
        </div>
        <div>
          <label class="text-sm text-gray-600 block mb-1">{{ t('menu.tools.qrcode.margin') }}</label>
          <select v-model.number="margin" class="px-3 py-1 border rounded text-sm">
            <option :value="0">0</option>
            <option :value="1">1</option>
            <option :value="2">2</option>
            <option :value="4">4</option>
          </select>
        </div>
      </div>

      <button @click="generate" :disabled="generating || !text.trim()"
              class="px-4 py-2 bg-blue-500 text-white rounded text-sm disabled:opacity-50">
        {{ generating ? t('common.loading') : t('menu.tools.qrcode.generate') }}
      </button>
    </div>

    <div v-if="qrUrl" class="mt-6 space-y-3">
      <div class="flex justify-center">
        <img :src="qrUrl" class="max-w-full rounded border" alt="QR Code" />
      </div>
      <p v-if="qrVersion > 0" class="text-xs text-gray-400 text-center">
        {{ t('menu.tools.qrcode.version') }} {{ qrVersion }}
      </p>
      <div class="text-center">
        <button @click="download"
                class="px-4 py-2 bg-green-500 text-white rounded text-sm">
          {{ t('common.download') }} PNG
        </button>
      </div>
    </div>

    <div v-if="error" class="mt-4 bg-red-100 border border-red-300 text-red-700 px-4 py-3 rounded">
      {{ error }}
    </div>
  </div>
</template>
