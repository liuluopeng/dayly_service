<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { setLocale } from '../i18n'

const { locale } = useI18n()

function switchTo(lang: 'zh' | 'en') {
  setLocale(lang)
}
</script>

<template>
  <div class="inline-flex rounded-lg overflow-hidden border border-gray-300">
    <button @click="switchTo('zh')"
      class="px-4 py-1.5 text-sm font-medium transition-colors"
      :class="locale === 'zh' ? 'bg-blue-500 text-white' : 'bg-white text-gray-700 hover:bg-gray-100'">
      中文
    </button>
    <button @click="switchTo('en')"
      class="px-4 py-1.5 text-sm font-medium transition-colors border-l border-gray-300"
      :class="locale === 'en' ? 'bg-blue-500 text-white' : 'bg-white text-gray-700 hover:bg-gray-100'">
      English
    </button>
  </div>
</template>
