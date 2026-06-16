<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 暂时使用类型断言来解决导入问题
const base64_encode_wasm = (window as any).base64_encode_wasm || ((input: string) => {
  return btoa(unescape(encodeURIComponent(input)));
});

const base64_decode_wasm = (window as any).base64_decode_wasm || ((input: string) => {
  try {
    return decodeURIComponent(escape(atob(input)));
  } catch {
    return null;
  }
});

const inputText = ref('');
const encodedText = ref('');
const decodedText = ref('');
const inputRef = ref<HTMLTextAreaElement | null>(null);
const errorMessage = ref('');

// 编码文本
async function encodeText() {
  try {
    errorMessage.value = '';
    encodedText.value = base64_encode_wasm(inputText.value);
  } catch (err) {
    console.error('编码失败:', err);
    // Missing key: base64Tool.encodeFailed
    errorMessage.value = t('common.error.loadFailed');
  }
}

// 解码文本
async function decodeText() {
  try {
    errorMessage.value = '';
    const result = base64_decode_wasm(inputText.value);
    // Missing key: base64Tool.decodeFailedInvalid
    decodedText.value = result || t('common.error.loadFailed');
  } catch (err) {
    console.error('解码失败:', err);
    // Missing key: base64Tool.decodeFailed
    errorMessage.value = t('common.error.loadFailed');
  }
}

// 复制到剪贴板
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    alert(t('common.copied'));
  } catch (err) {
    console.error('复制失败:', err);
    // Missing key: base64Tool.copyFailed
    alert(t('common.error.loadFailed'));
  }
}

// 自动对焦到输入框
onMounted(() => {
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100);
});
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <header class="bg-white shadow-sm">
      <div class="container mx-auto px-4 py-3 flex justify-between items-center">
        <h1 class="text-xl font-bold text-blue-600">{{ t('base64Tool.title') }}</h1>
      </div>
    </header>

    <div class="container mx-auto px-4 py-8">
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h2 class="text-xl font-semibold mb-6">{{ t('base64Tool.title') }}</h2>

        <!-- 错误信息 -->
        <div v-if="errorMessage" class="p-4 bg-red-100 text-red-700 rounded-md mb-4">
          {{ errorMessage }}
        </div>

        <!-- 输入区域 -->
        <div class="mb-4">
          <label class="block text-gray-700 mb-2">{{ t('base64Tool.input') }}</label>
          <textarea ref="inputRef" v-model="inputText"
            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            rows="4" :placeholder="t('base64Tool.inputPlaceholder')"></textarea>
        </div>

        <!-- 操作按钮 -->
        <div class="flex space-x-4 mb-6">
          <button @click="encodeText"
            class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
            {{ t('base64Tool.encode') }}
          </button>
          <button @click="decodeText"
            class="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 transition-colors">
            {{ t('base64Tool.decode') }}
          </button>
        </div>

        <!-- 编码结果 -->
        <div class="mb-6" v-if="encodedText">
          <div class="flex justify-between items-center mb-2">
            <label class="block text-gray-700">{{ t('base64Tool.output') }}</label>
            <button @click="copyToClipboard(encodedText)" class="text-sm text-blue-600 hover:text-blue-800">
              {{ t('common.copy') }}
            </button>
          </div>
          <textarea v-model="encodedText" class="w-full px-4 py-2 border border-gray-300 rounded-md bg-gray-50 readonly"
            rows="3" readonly></textarea>
        </div>

        <!-- 解码结果 -->
        <div class="mb-6" v-if="decodedText">
          <div class="flex justify-between items-center mb-2">
            <label class="block text-gray-700">{{ t('base64Tool.output') }}</label>
            <button @click="copyToClipboard(decodedText)" class="text-sm text-blue-600 hover:text-blue-800">
              {{ t('common.copy') }}
            </button>
          </div>
          <textarea v-model="decodedText" class="w-full px-4 py-2 border border-gray-300 rounded-md bg-gray-50 readonly"
            rows="3" readonly></textarea>
        </div>

        <!-- 说明 -->
        <div class="mt-8 p-4 bg-blue-50 rounded-md">
          <h3 class="text-lg font-medium text-blue-700 mb-2">{{ t('base64Tool.title') }}</h3>
          <ul class="list-disc pl-5 text-gray-600 space-y-1">
            <!-- Missing keys: base64Tool.helpText1-4 -->
            <li>{{ t('base64Tool.inputPlaceholder') }}</li>
            <li>{{ t('base64Tool.encode') }}</li>
            <li>{{ t('base64Tool.decode') }}</li>
            <li>{{ t('common.copy') }}</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
