<template>
  <div class="p-5 max-w-[800px] mx-auto">
    <h1 class="text-center text-[#333] mb-[30px]">{{ t('chineseRemover.title') }}</h1>

    <div class="mb-[30px]">
      <label for="text-input" class="block mb-[10px] font-medium text-[#666]">{{ t('chineseRemover.input') }}：</label>
      <textarea
        id="text-input"
        v-model="inputText"
        :placeholder="t('chineseRemover.inputPlaceholder')"
        rows="5"
        class="w-full p-[15px] border border-[#ddd] rounded-lg text-base resize-y font-inherit focus:outline-none focus:border-[#42b983] focus:shadow-[0_0_0_2px_rgba(66,185,131,0.2)]"
      ></textarea>
    </div>

    <div class="bg-[#f9f9f9] p-5 rounded-lg border border-[#eee]">
      <h3 class="mt-0 mb-[15px] text-[#555]">{{ t('chineseRemover.output') }}：</h3>
      <div class="bg-white p-[15px] rounded border border-[#ddd] min-h-[100px] whitespace-pre-wrap font-mono text-sm leading-normal">{{ processedText }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 输入文本
const inputText = ref('');

// 计算属性：去除汉字和中文标点后的文本
const processedText = computed(() => {
  // 使用正则表达式去除所有汉字和中文标点符号
  // 汉字的Unicode范围：一-龥
  // 中文标点的Unicode范围：　-〿 -⁯＀-￯
  return inputText.value.replace(/[一-龥　-〿 -⁯＀-￯]/g, '');
});
</script>
