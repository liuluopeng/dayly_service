<template>
  <div class="min-h-screen bg-gray-50 p-6">
    <div class="max-w-3xl mx-auto bg-white rounded-lg shadow-md p-6">
      <h1 class="text-2xl font-semibold text-gray-800 mb-6">{{ t('timestampTool.title') }}</h1>

      <!-- 当前时间 -->
      <div class="mb-8">
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('timestampTool.currentTime') }}</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="bg-gray-50 p-4 rounded-md">
            <!-- Missing key: timestampTool.currentTimestampSec ("当前时间戳（秒）") -->
            <p class="text-sm text-gray-600">{{ t('timestampTool.timestamp') }}（秒）</p>
            <p class="text-xl font-medium text-blue-600">{{ currentTimestamp }}</p>
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <!-- Missing key: timestampTool.currentLocalTime ("当前本地时间") -->
            <p class="text-sm text-gray-600">{{ t('timestampTool.datetime') }}</p>
            <p class="text-xl font-medium text-blue-600">{{ currentLocalTime }}</p>
          </div>
          <div class="bg-gray-50 p-4 rounded-md md:col-span-2">
            <!-- Missing key: timestampTool.currentUtcTime ("当前 UTC 时间") -->
            <p class="text-sm text-gray-600">UTC {{ t('timestampTool.datetime') }}</p>
            <p class="text-xl font-medium text-blue-600">{{ currentUtcTime }}</p>
          </div>
        </div>
      </div>

      <!-- 时间戳转换 -->
      <div class="mb-8">
        <!-- Missing key: timestampTool.timestampConvert ("时间戳转换") -->
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('timestampTool.convert') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('timestampTool.timestamp') }}（秒）</label>
            <input
              type="number"
              v-model="timestampInput"
              class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              :placeholder="t('timestampTool.timestamp')"
            />
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="bg-gray-50 p-4 rounded-md">
              <!-- Missing key: timestampTool.localtime ("本地时间") -->
              <p class="text-sm text-gray-600">{{ t('timestampTool.datetime') }}</p>
              <p class="text-xl font-medium text-blue-600">{{ timestampToLocalResult }}</p>
            </div>
            <div class="bg-gray-50 p-4 rounded-md">
              <p class="text-sm text-gray-600">UTC {{ t('timestampTool.datetime') }}</p>
              <p class="text-xl font-medium text-blue-600">{{ timestampToUtcResult }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- 时间字符串转换 -->
      <div>
        <!-- Missing key: timestampTool.stringConvert ("时间字符串转换") -->
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('timestampTool.datetime') }} {{ t('timestampTool.convert') }}</h2>
        <div class="space-y-4">
          <div>
            <!-- Missing key: timestampTool.localTimeString ("本地时间字符串 (YYYY-MM-DD HH:MM:SS)") -->
            <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('timestampTool.datetime') }} (YYYY-MM-DD HH:MM:SS)</label>
            <input
              type="text"
              v-model="localTimeInput"
              class="w-full px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="2023-01-01 12:00:00"
            />
          </div>
          <div class="bg-gray-50 p-4 rounded-md">
            <p class="text-sm text-gray-600">{{ t('timestampTool.timestamp') }}（秒）</p>
            <p class="text-xl font-medium text-blue-600">{{ localToTimestampResult }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 暂时使用浏览器内置方法作为 fallback
const timestamp_to_local = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

const timestamp_to_utc = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toUTCString();
};

const local_to_timestamp = (time_str: string): number | null => {
  const date = new Date(time_str);
  return isNaN(date.getTime()) ? null : Math.floor(date.getTime() / 1000);
};

const get_current_timestamp = (): number => {
  return Math.floor(Date.now() / 1000);
};

const get_current_local_time = (): string => {
  return new Date().toLocaleString();
};

const get_current_utc_time = (): string => {
  return new Date().toUTCString();
};

// 响应式数据
const currentTimestamp = ref('');
const currentLocalTime = ref('');
const currentUtcTime = ref('');
const timestampInput = ref('');
const timestampToLocalResult = ref('');
const timestampToUtcResult = ref('');
const localTimeInput = ref('');
const localToTimestampResult = ref('');

// 更新当前时间
const updateCurrentTime = () => {
  currentTimestamp.value = get_current_timestamp().toString();
  currentLocalTime.value = get_current_local_time();
  currentUtcTime.value = get_current_utc_time();
};

// 监听时间戳输入变化
watch(timestampInput, (newValue) => {
  if (newValue) {
    const timestamp = parseInt(newValue);
    if (!isNaN(timestamp)) {
      timestampToLocalResult.value = timestamp_to_local(timestamp);
      timestampToUtcResult.value = timestamp_to_utc(timestamp);
    } else {
      // Missing key: timestampTool.invalidTimestamp ("无效的时间戳")
      timestampToLocalResult.value = t('common.error.loadFailed');
      timestampToUtcResult.value = t('common.error.loadFailed');
    }
  } else {
    timestampToLocalResult.value = '';
    timestampToUtcResult.value = '';
  }
});

// 监听本地时间输入变化
watch(localTimeInput, (newValue) => {
  if (newValue) {
    const timestamp = local_to_timestamp(newValue);
    // Missing key: timestampTool.invalidTimeString ("无效的时间字符串")
    localToTimestampResult.value = timestamp !== null ? timestamp.toString() : t('common.error.loadFailed');
  } else {
    localToTimestampResult.value = '';
  }
});

// 组件挂载时初始化
onMounted(() => {
  updateCurrentTime();
  // 每秒更新一次当前时间
  setInterval(updateCurrentTime, 1000);
});
</script>
