<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { login_wasm, init_api } from '../types/wasm-typed';
import { useAuth } from '../composables/useAuth';
import { getApiUrl, getPortFromUrl } from '../utils/apiUrl';

const { t } = useI18n();
const router = useRouter();
const { isLoggedIn, login } = useAuth();
const username = ref(localStorage.getItem('savedUsername') || '');
const password = ref('');
const usernameInput = ref<HTMLInputElement | null>(null);
const isLoading = ref(false);
const errorMessage = ref('');

onMounted(() => {
  if (isLoggedIn.value) {
    router.push('/menu');
  } else {
    setTimeout(() => {
      usernameInput.value?.focus();
    }, 100);
  }
});

async function handleLogin() {
  if (!username.value || !password.value) {
    errorMessage.value = t('login.error.emptyFields');
    return;
  }

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const apiUrl = getApiUrl();
    const port = getPortFromUrl(apiUrl);

    init_api(null, apiUrl, port);

    const result = await login_wasm(username.value, password.value, null, port);

    if (result) {
      const { token, token_type } = result;
      login(token, token_type, username.value);

      init_api(token, apiUrl, port);

      router.push('/menu');
    }
  } catch (error) {
    console.error('登录失败:', error);
    errorMessage.value = error instanceof Error ? error.message : t('login.error.loginFailed');
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-500 to-purple-600 p-4">
    <div class="w-full max-w-md">
      <div class="bg-white/10 backdrop-blur-lg rounded-3xl p-8 shadow-2xl">
        <div class="text-center mb-8">
          <h1 class="text-3xl font-bold text-white mb-2">{{ t('login.title') }}</h1>
          <p class="text-white/70">{{ t('login.subtitle') }}</p>
        </div>

        <form @submit.prevent="handleLogin" class="space-y-6">
          <div>
            <label class="block text-white/80 mb-2 text-sm font-medium">{{ t('login.username') }}</label>
            <input ref="usernameInput" v-model="username" type="text" :placeholder="t('login.usernamePlaceholder')"
              class="w-full px-4 py-3 bg-white/10 border border-white/20 rounded-xl text-white placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-white/50 focus:border-transparent transition-all"
              :disabled="isLoading" />
          </div>

          <div>
            <label class="block text-white/80 mb-2 text-sm font-medium">{{ t('login.password') }}</label>
            <input v-model="password" type="password" :placeholder="t('login.passwordPlaceholder')"
              class="w-full px-4 py-3 bg-white/10 border border-white/20 rounded-xl text-white placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-white/50 focus:border-transparent transition-all"
              :disabled="isLoading" />
          </div>

          <div v-if="errorMessage" class="p-4 bg-red-500/20 border border-red-500/30 rounded-xl text-red-200 text-sm">
            {{ errorMessage }}
          </div>

          <button type="submit" :disabled="isLoading"
            class="w-full py-3 bg-gradient-to-r from-blue-500 to-purple-500 text-white font-semibold rounded-xl shadow-lg hover:shadow-xl transform hover:scale-[1.02] transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none">
            <div v-if="isLoading" class="flex items-center justify-center gap-2">
              <div class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              <span>{{ t('login.submitting') }}</span>
            </div>
            <span v-else>{{ t('login.submit') }}</span>
          </button>
        </form>

        <div class="mt-6 text-center text-white/60 text-sm">
          <p>{{ t('login.forgotPassword') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
