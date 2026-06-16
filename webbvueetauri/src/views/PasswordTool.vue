<template>
  <div class="min-h-screen bg-gray-50 p-6">
    <div class="max-w-3xl mx-auto bg-white rounded-lg shadow-md p-6">
      <h1 class="text-2xl font-semibold text-gray-800 mb-6">{{ t('passwordTool.title') }}</h1>

      <!-- 密码长度设置 -->
      <div class="mb-6">
        <!-- Missing key: passwordTool.lengthLabel ("密码长度") -->
        <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('passwordTool.length') }}</label>
        <div class="flex items-center space-x-4">
          <input
            type="range"
            v-model.number="passwordLength"
            min="8"
            max="50"
            class="w-full"
          />
          <span class="text-lg font-medium text-blue-600">{{ passwordLength }}</span>
        </div>
      </div>

      <!-- 密码类型选择 -->
      <div class="mb-8">
        <!-- Missing key: passwordTool.type ("密码类型") -->
        <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('passwordTool.length') }}</label>
        <div class="flex space-x-4">
          <!-- Missing key: passwordTool.normalPassword ("普通密码") -->
          <button
            @click="passwordType = 'normal'"
            :class="[
              'px-4 py-2 rounded-md transition-colors',
              passwordType === 'normal'
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            ]"
          >
            {{ t('passwordTool.generate') }}
          </button>
          <!-- Missing key: passwordTool.strongPassword ("强密码") -->
          <button
            @click="passwordType = 'strong'"
            :class="[
              'px-4 py-2 rounded-md transition-colors',
              passwordType === 'strong'
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            ]"
          >
            {{ t('passwordTool.includeSymbols') }}
          </button>
        </div>
      </div>

      <!-- 生成的密码 -->
      <div class="mb-8">
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('passwordTool.generatedPassword') }}</h2>
        <div class="relative">
          <input
            type="text"
            v-model="generatedPassword"
            readonly
            class="w-full px-4 py-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            @click="copyToClipboard(generatedPassword)"
            class="absolute right-2 top-1/2 transform -translate-y-1/2 px-3 py-1 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors"
          >
            {{ t('common.copy') }}
          </button>
        </div>
        <div class="mt-4 flex space-x-4">
          <button
            @click="generatePassword"
            class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors"
          >
            {{ t('passwordTool.generate') }}
          </button>
          <!-- Missing key: passwordTool.generateMultiple ("生成多个密码") -->
          <button
            @click="generateMultiplePasswords"
            class="px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 transition-colors"
          >
            {{ t('passwordTool.generate') }}
          </button>
        </div>
      </div>

      <!-- 多个密码 -->
      <div v-if="multiplePasswords.length > 0">
        <!-- Missing key: passwordTool.multiplePasswords ("多个密码") -->
        <h2 class="text-lg font-medium text-gray-700 mb-4">{{ t('passwordTool.generatedPassword') }}</h2>
        <div class="space-y-2">
          <div
            v-for="(password, index) in multiplePasswords"
            :key="index"
            class="flex items-center justify-between bg-gray-50 p-3 rounded-md"
          >
            <span class="font-medium">{{ password }}</span>
            <button
              @click="copyToClipboard(password)"
              class="px-2 py-1 bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors text-sm"
            >
              {{ t('common.copy') }}
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
const generate_password = (length: number): string => {
  const charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;':,.<>/?";
  let password = "";
  for (let i = 0; i < length; i++) {
    const randomIndex = Math.floor(Math.random() * charset.length);
    password += charset[randomIndex];
  }
  return password;
};

const generate_strong_password = (length: number): string => {
  const uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  const lowercase = "abcdefghijklmnopqrstuvwxyz";
  const digits = "0123456789";
  const symbols = "!@#$%^&*()_+-=[]{}|;':,.<>/?";

  // 确保密码长度至少为 4
  const len = Math.max(length, 4);

  let password = "";

  // 确保至少包含一个大写字母、一个小写字母、一个数字和一个特殊字符
  password += uppercase[Math.floor(Math.random() * uppercase.length)];
  password += lowercase[Math.floor(Math.random() * lowercase.length)];
  password += digits[Math.floor(Math.random() * digits.length)];
  password += symbols[Math.floor(Math.random() * symbols.length)];

  // 填充剩余长度
  const allChars = uppercase + lowercase + digits + symbols;
  for (let i = 4; i < len; i++) {
    password += allChars[Math.floor(Math.random() * allChars.length)];
  }

  // 打乱密码顺序
  password = password.split('').sort(() => Math.random() - 0.5).join('');

  return password;
};

// 响应式数据
const passwordLength = ref(16);
const passwordType = ref('strong');
const generatedPassword = ref('');
const multiplePasswords = ref<string[]>([]);

// 生成密码
const generatePassword = () => {
  if (passwordType.value === 'strong') {
    generatedPassword.value = generate_strong_password(passwordLength.value);
  } else {
    generatedPassword.value = generate_password(passwordLength.value);
  }
};

// 生成多个密码
const generateMultiplePasswords = () => {
  multiplePasswords.value = [];
  for (let i = 0; i < 5; i++) {
    if (passwordType.value === 'strong') {
      multiplePasswords.value.push(generate_strong_password(passwordLength.value));
    } else {
      multiplePasswords.value.push(generate_password(passwordLength.value));
    }
  }
};

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    alert(t('common.copied'));
  } catch (err) {
    console.error('复制失败:', err);
    // Missing key: passwordTool.copyFailed ("复制失败，请手动复制")
    alert(t('common.error.loadFailed'));
  }
};

// 初始化生成
generatePassword();
</script>
