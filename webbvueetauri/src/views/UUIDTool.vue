<template>
  <div class="min-h-screen bg-gray-50 p-6">
    <div class="max-w-3xl mx-auto bg-white rounded-lg shadow-md p-6">
      <h1 class="text-2xl font-semibold text-gray-800 mb-6">{{ t('uuidTool.title') }}</h1>

      <!-- UUID 生成 -->
      <div class="mb-8">
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('uuidTool.generate') }}</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('uuidTool.v4Random') }}</p>
            <p class="text-lg font-medium text-blue-600 break-all">{{ uuidV4 }}</p>
            <button @click="generateUUIDv4"
              class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
              {{ t('uuidTool.generateBtn') }}
            </button>
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('uuidTool.v6TimeSorted') }}</p>
            <p class="text-lg font-medium text-blue-600 break-all">{{ uuidV6 }}</p>
            <button @click="generateUUIDv6"
              class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
              {{ t('uuidTool.generateBtn') }}
            </button>
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('uuidTool.v7Unix') }}</p>
            <p class="text-lg font-medium text-blue-600 break-all">{{ uuidV7 }}</p>
            <button @click="generateUUIDv7"
              class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
              {{ t('uuidTool.generateBtn') }}
            </button>
          </div>
        </div>
      </div>

      <!-- UUID v5 生成 -->
      <div class="mb-8">
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('uuidTool.generateV5') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('uuidTool.namespaceUuid') }}</label>
            <input type="text" v-model="namespace"
              class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="00000000-0000-0000-0000-000000000000" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('uuidTool.name') }}</label>
            <input type="text" v-model="name"
              class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              :placeholder="t('uuidTool.namePlaceholder')" />
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('uuidTool.v5') }}</p>
            <p class="text-lg font-medium text-blue-600 break-all">{{ uuidV5 }}</p>
            <button @click="generateUUIDv5"
              class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
              {{ t('uuidTool.generateBtn') }}
            </button>
          </div>
        </div>
      </div>

      <!-- UUID 验证 -->
      <div>
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('uuidTool.validate') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('uuidTool.uuidString') }}</label>
            <input type="text" v-model="uuidToValidate"
              class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              :placeholder="t('uuidTool.uuidPlaceholder')" />
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('uuidTool.validationResult') }}</p>
            <p class="text-lg font-medium" :class="isValidUUID ? 'text-green-600' : 'text-red-600'">{{ validationResult
              }}</p>
            <button @click="validateUUID"
              class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
              {{ t('uuidTool.validateBtn') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 暂时使用浏览器内置方法作为 fallback
const generate_uuid_v4 = (): string => {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
};

const generate_uuid_v5 = (namespace: string, name: string): string => {
  // 简化版 UUID v5 生成，实际应该使用加密哈希
  const combined = namespace + name;
  let hash = 0;
  for (let i = 0; i < combined.length; i++) {
    const char = combined.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  const hex = Math.abs(hash).toString(16);
  return 'xxxxxxxx-xxxx-5xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c, index) {
    const r = (hex[index] ? parseInt(hex[index], 16) : Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
};

const generate_uuid_v6 = (): string => {
  // 简化版 UUID v6 生成
  const timestamp = Date.now().toString(16);
  return `${timestamp.slice(0, 8)}-${timestamp.slice(8, 12)}-6${timestamp.slice(12, 15)}-${Math.floor(Math.random() * 16).toString(16)}${Math.floor(Math.random() * 16).toString(16)}${Math.floor(Math.random() * 16).toString(16)}-${Math.floor(Math.random() * 1000000000000).toString(16).padStart(12, '0')}`;
};

const generate_uuid_v7 = (): string => {
  // 简化版 UUID v7 生成
  const timestamp = Date.now().toString(16);
  return `${timestamp}-7${Math.floor(Math.random() * 16).toString(16)}${Math.floor(Math.random() * 16).toString(16)}${Math.floor(Math.random() * 16).toString(16)}-${Math.floor(Math.random() * 1000000000000).toString(16).padStart(12, '0')}`;
};

const validate_uuid = (uuid_str: string): boolean => {
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(uuid_str);
};

// 响应式数据
const uuidV4 = ref('');
const uuidV6 = ref('');
const uuidV7 = ref('');
const namespace = ref('00000000-0000-0000-0000-000000000000');
const name = ref('');
const uuidV5 = ref('');
const uuidToValidate = ref('');
const isValidUUID = ref(false);
const validationResult = ref('');

// 生成 UUID v4
const generateUUIDv4 = () => {
  uuidV4.value = generate_uuid_v4();
};

// 生成 UUID v5
const generateUUIDv5 = () => {
  if (namespace.value && name.value) {
    uuidV5.value = generate_uuid_v5(namespace.value, name.value);
  } else {
    // Missing key: uuidTool.enterNamespaceAndName
    uuidV5.value = t('uuidTool.namePlaceholder');
  }
};

// 生成 UUID v6
const generateUUIDv6 = () => {
  uuidV6.value = generate_uuid_v6();
};

// 生成 UUID v7
const generateUUIDv7 = () => {
  uuidV7.value = generate_uuid_v7();
};

// 验证 UUID
const validateUUID = () => {
  if (uuidToValidate.value) {
    isValidUUID.value = validate_uuid(uuidToValidate.value);
    validationResult.value = isValidUUID.value ? t('uuidTool.valid') : t('uuidTool.invalid');
  } else {
    validationResult.value = t('uuidTool.uuidPlaceholder');
  }
};

// 初始化生成
generateUUIDv4();
generateUUIDv6();
generateUUIDv7();
</script>
