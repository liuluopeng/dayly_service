<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { convert_image_wasm, convert_image_with_size_wasm, crop_image_wasm, detect_image_format } from "../types/wasm-typed";

const { t } = useI18n();

const file = ref<File | null>(null);
const originalUrl = ref("");
const originalSize = ref(0);
const originalFormat = ref("");
const imgDimensions = ref({ w: 0, h: 0 });

const targetFormat = ref("jpeg");
const quality = ref(85);
const converting = ref(false);
const resultUrl = ref("");
const resultSize = ref(0);
const error = ref("");

// 裁剪
const cropX = ref(0);
const cropY = ref(0);
const cropW = ref(0);
const cropH = ref(0);
const showCrop = ref(false);

// 缩放
const showResize = ref(false);
const resizeW = ref(0);
const resizeH = ref(0);
const keepRatio = ref(true);

const qualityPresets = [
  { label: "low", value: 30 },
  { label: "medium", value: 60 },
  { label: "high", value: 85 },
  { label: "max", value: 100 },
];

const resizePresets = [
  { label: "640×360", w: 640, h: 360 },
  { label: "HD 720p", w: 1280, h: 720 },
  { label: "FHD 1080p", w: 1920, h: 1080 },
  { label: "4K", w: 3840, h: 2160 },
];

const formats = ["jpeg", "png", "webp", "bmp", "gif"];

const formatExtensions: Record<string, string> = {
  jpeg: "jpg", png: "png", webp: "webp", bmp: "bmp", gif: "gif",
};

const fileName = computed(() => {
  if (!file.value) return "";
  return `${file.value.name.replace(/\.[^.]+$/, "")}.${formatExtensions[targetFormat.value] || targetFormat.value}`;
});

const qualityLabel = computed(() => {
  const preset = qualityPresets.find(p => p.value === quality.value);
  return preset ? `menu.imageConverter.${preset.label}` : `${quality.value}%`;
});

function setQuality(val: number) { quality.value = val; }

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const f = input.files?.[0];
  if (!f) return;
  file.value = f;
  originalUrl.value = URL.createObjectURL(f);
  originalSize.value = f.size;
  resultUrl.value = "";
  error.value = "";

  const reader = new FileReader();
  reader.onload = async () => {
    const buf = new Uint8Array(reader.result as ArrayBuffer);
    const fmt = detect_image_format(buf);
    originalFormat.value = fmt || t('common.unknown');
    const img = new Image();
    img.onload = () => {
      imgDimensions.value = { w: img.naturalWidth, h: img.naturalHeight };
      cropW.value = img.naturalWidth;
      cropH.value = img.naturalHeight;
      resizeW.value = img.naturalWidth;
      resizeH.value = img.naturalHeight;
    };
    img.src = URL.createObjectURL(f);
  };
}

function applyResizePreset(w: number, h: number) {
  resizeW.value = w;
  resizeH.value = h;
}

function resetCrop() {
  cropX.value = 0; cropY.value = 0;
  cropW.value = imgDimensions.value.w;
  cropH.value = imgDimensions.value.h;
}

function resetResize() {
  resizeW.value = imgDimensions.value.w;
  resizeH.value = imgDimensions.value.h;
}

// 宽度变化时按比例调整高度
function onResizeWChange() {
  if (keepRatio.value && imgDimensions.value.w > 0) {
    resizeH.value = Math.round(resizeW.value * imgDimensions.value.h / imgDimensions.value.w);
  }
}

function onResizeHChange() {
  if (keepRatio.value && imgDimensions.value.h > 0) {
    resizeW.value = Math.round(resizeH.value * imgDimensions.value.w / imgDimensions.value.h);
  }
}

async function convert() {
  if (!file.value) return;
  converting.value = true;
  error.value = "";
  resultUrl.value = "";

  try {
    let buf = new Uint8Array(await file.value.arrayBuffer());

    if (showCrop.value && cropW.value > 0 && cropH.value > 0) {
      buf = crop_image_wasm(buf, cropX.value, cropY.value, cropW.value, cropH.value);
    }

    if (showResize.value && resizeW.value > 0 && resizeH.value > 0) {
      buf = convert_image_with_size_wasm(buf, targetFormat.value, quality.value, resizeW.value, resizeH.value);
    } else {
      buf = convert_image_wasm(buf, targetFormat.value, quality.value);
    }

    const blob = new Blob([buf], {
      type: `image/${targetFormat.value === "jpeg" ? "jpeg" : targetFormat.value}`,
    });
    resultUrl.value = URL.createObjectURL(blob);
    resultSize.value = blob.size;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    converting.value = false;
  }
}

function download() {
  if (!resultUrl.value) return;
  const a = document.createElement("a");
  a.href = resultUrl.value;
  a.download = fileName.value;
  a.click();
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatLabel(fmt: string) {
  const map: Record<string, string> = { jpeg: "JPEG", png: "PNG", webp: "WebP", bmp: "BMP", gif: "GIF" };
  return map[fmt] || fmt.toUpperCase();
}
</script>

<template>
  <div class="p-4 max-w-3xl mx-auto">
    <h1 class="text-2xl font-bold mb-4">{{ t('menu.imageConverter.title') }}</h1>

    <div class="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center mb-4"
         :class="{ 'border-blue-400': file }">
      <input type="file" accept="image/*" @change="onFileChange" class="hidden" id="fileInput" />
      <label for="fileInput" class="cursor-pointer block">
        <div class="text-4xl mb-2">🖼️</div>
        <p class="text-gray-500">{{ file ? file.name : (t('common.select') + '...') }}</p>
      </label>
    </div>

    <div v-if="file" class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
      <div>
        <p class="text-sm text-gray-500 mb-1">{{ t('menu.imageConverter.original') }}</p>
        <img :src="originalUrl" class="max-w-full max-h-64 rounded border object-contain" />
        <p class="text-xs text-gray-400 mt-1">
          {{ originalFormat.toUpperCase() }} · {{ formatFileSize(originalSize) }} ·
          {{ imgDimensions.w }}×{{ imgDimensions.h }}
        </p>
      </div>
      <div v-if="resultUrl">
        <p class="text-sm text-gray-500 mb-1">{{ t('menu.imageConverter.result') }}</p>
        <img :src="resultUrl" class="max-w-full max-h-64 rounded border object-contain" />
        <p class="text-xs text-gray-400 mt-1">{{ formatFileSize(resultSize) }}</p>
      </div>
    </div>

    <div v-if="file" class="space-y-4 mb-4">
      <!-- 格式 + 档位 -->
      <div class="flex flex-wrap gap-4 items-end">
        <div>
          <label class="text-sm text-gray-600 block mb-1">{{ t('menu.imageConverter.format') }}</label>
          <select v-model="targetFormat" class="px-3 py-1.5 border rounded text-sm">
            <option v-for="f in formats" :key="f" :value="f">{{ formatLabel(f) }}</option>
          </select>
        </div>
        <div v-if="targetFormat === 'jpeg' || targetFormat === 'webp'" class="flex-1 min-w-[200px]">
          <label class="text-sm text-gray-600 block mb-1">{{ t('menu.imageConverter.quality') }}: {{ qualityLabel }}</label>
          <input type="range" min="1" max="100" v-model.number="quality" class="w-full" />
          <div class="flex gap-1 mt-1">
            <button v-for="p in qualityPresets" :key="p.value"
                    @click="setQuality(p.value)"
                    class="px-2 py-0.5 text-xs rounded"
                    :class="quality === p.value ? 'bg-blue-500 text-white' : 'bg-gray-200'">
              {{ t(`menu.imageConverter.${p.label}`) }}
            </button>
          </div>
        </div>
      </div>

      <!-- 缩放 -->
      <div>
        <label class="inline-flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="showResize" class="accent-blue-500" />
          <span class="text-sm font-medium">{{ t('menu.imageConverter.resize') }}</span>
        </label>
      </div>
      <div v-if="showResize" class="p-3 bg-gray-50 rounded-lg space-y-2">
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="text-xs text-gray-500 block">{{ t('menu.imageConverter.width') }}</label>
            <input type="number" v-model.number="resizeW" min="1"
                   @input="onResizeWChange"
                   class="w-full px-2 py-1 border rounded text-sm" />
          </div>
          <div>
            <label class="text-xs text-gray-500 block">{{ t('menu.imageConverter.height') }}</label>
            <input type="number" v-model.number="resizeH" min="1"
                   @input="onResizeHChange"
                   class="w-full px-2 py-1 border rounded text-sm" />
          </div>
        </div>
        <label class="inline-flex items-center gap-1 text-xs">
          <input type="checkbox" v-model="keepRatio" class="accent-blue-500" />
          {{ t('menu.imageConverter.keepRatio') }}
        </label>
        <div class="flex flex-wrap gap-1">
          <button v-for="p in resizePresets" :key="p.label"
                  @click="applyResizePreset(p.w, p.h)"
                  class="px-2 py-0.5 text-xs bg-gray-200 rounded hover:bg-gray-300">
            {{ p.label }}
          </button>
          <button @click="resetResize" class="px-2 py-0.5 text-xs bg-gray-200 rounded hover:bg-gray-300">
            {{ t('common.reset') }}
          </button>
        </div>
      </div>

      <!-- 裁剪 -->
      <div>
        <label class="inline-flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="showCrop" class="accent-blue-500" />
          <span class="text-sm font-medium">{{ t('menu.imageConverter.crop') }}</span>
        </label>
      </div>
      <div v-if="showCrop" class="grid grid-cols-2 md:grid-cols-4 gap-3 p-3 bg-gray-50 rounded-lg">
        <div><label class="text-xs text-gray-500 block">X</label>
          <input type="number" v-model.number="cropX" min="0" :max="imgDimensions.w - 1" class="w-full px-2 py-1 border rounded text-sm" /></div>
        <div><label class="text-xs text-gray-500 block">Y</label>
          <input type="number" v-model.number="cropY" min="0" :max="imgDimensions.h - 1" class="w-full px-2 py-1 border rounded text-sm" /></div>
        <div><label class="text-xs text-gray-500 block">W</label>
          <input type="number" v-model.number="cropW" min="1" :max="imgDimensions.w" class="w-full px-2 py-1 border rounded text-sm" /></div>
        <div><label class="text-xs text-gray-500 block">H</label>
          <input type="number" v-model.number="cropH" min="1" :max="imgDimensions.h" class="w-full px-2 py-1 border rounded text-sm" /></div>
        <div class="col-span-4"><button @click="resetCrop" class="px-3 py-1 text-xs bg-gray-200 rounded">{{ t('menu.imageConverter.resetCrop') }}</button></div>
      </div>

      <!-- 按钮 -->
      <div class="flex flex-wrap gap-2">
        <button @click="convert" :disabled="converting"
                class="px-4 py-2 bg-blue-500 text-white rounded text-sm disabled:opacity-50">
          {{ converting ? t('common.loading') : t('menu.imageConverter.convert') }}
        </button>
        <button v-if="resultUrl" @click="download"
                class="px-4 py-2 bg-green-500 text-white rounded text-sm">
          {{ t('common.download') }} {{ fileName }}
        </button>
      </div>
    </div>

    <div v-if="error" class="bg-red-100 border border-red-300 text-red-700 px-4 py-3 rounded">
      {{ error }}
    </div>
  </div>
</template>
