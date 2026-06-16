<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useApiReinit } from "../composables/useApiReinit";
import { useAuth } from "../composables/useAuth";
import LanguageSwitcher from "../components/LanguageSwitcher.vue";

const { t } = useI18n();
const router = useRouter();
const { reinitApi, setApiUrl, getApiUrl } = useApiReinit();
const { logout, username } = useAuth();
const currentApiUrl = ref("");
const showCustom = ref(false);
const customUrl = ref("");
const isCustom = ref(false);

onMounted(() => {
  currentApiUrl.value = getApiUrl();
  customUrl.value = currentApiUrl.value;
  isCustom.value = !!localStorage.getItem('apiUrl');
});

function handleLogout() {
  logout();
  router.push('/login');
}

function applyCustomUrl() {
  if (!customUrl.value.trim()) return;
  setApiUrl(customUrl.value.trim());
  currentApiUrl.value = customUrl.value.trim();
  isCustom.value = true;
  reinitApi();
}

function resetToAuto() {
  localStorage.removeItem('apiUrl');
  location.reload();
}
</script>

<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold text-center text-blue-600 mb-8">{{ t('settings.title') }}</h1>

    <div class="bg-white p-6 rounded-lg shadow-md max-w-md mx-auto">
      <div class="mb-6 pb-4 border-b border-gray-200">
        <div class="flex items-center gap-3">
          <div class="w-12 h-12 rounded-full bg-blue-500 flex items-center justify-center text-white text-xl font-bold">
            {{ username ? username.charAt(0).toUpperCase() : '?' }}
          </div>
          <div>
            <p class="text-lg font-semibold text-gray-800">{{ username || t('notLoggedIn') }}</p>
            <p class="text-sm text-gray-500">{{ t('currentUser') }}</p>
          </div>
        </div>
      </div>

      <div class="mb-6 pb-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold mb-2">{{ t('settings.language') }}</h2>
        <LanguageSwitcher />
      </div>

      <h2 class="text-xl font-semibold mb-2">{{ t('settings.serverAddress') }}</h2>
      <p class="text-gray-600 mb-2 bg-gray-50 p-3 rounded font-mono break-all">
        {{ currentApiUrl }}
      </p>
      <p v-if="isCustom" class="text-orange-500 text-sm mb-4">
        {{ t('settings.manualOverride') }}
        <button @click="resetToAuto" class="ml-2 text-blue-500 underline">{{ t('settings.restoreAuto') }}</button>
      </p>
      <p v-else class="text-green-600 text-sm mb-4">{{ t('settings.autoDetected') }}</p>

      <div class="border-t border-gray-200 pt-4">
        <button @click="showCustom = !showCustom"
          class="text-sm text-gray-500 hover:text-gray-700">
          {{ showCustom ? t('settings.collapse') : t('settings.customAddress') }}
        </button>
        <div v-if="showCustom" class="mt-3 space-y-2">
          <input v-model="customUrl" placeholder="http://192.168.1.100:23000"
            class="w-full px-3 py-2 border rounded font-mono text-sm" />
          <button @click="applyCustomUrl"
            class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm">
            {{ t('settings.apply') }}
          </button>
        </div>
      </div>

      <div class="mt-8 pt-6 border-t border-gray-200">
        <button @click="handleLogout"
          class="w-full px-4 py-3 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors font-medium">
          {{ t('settings.logout') }}
        </button>
      </div>
    </div>
  </div>
</template>
