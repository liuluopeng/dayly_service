<script setup lang="ts">
import { ref } from 'vue';

const leftUrl = ref('');
const rightUrl = ref('');
const leftError = ref(false);
const rightError = ref(false);

function onLeftError() { leftError.value = true; }
function onRightError() { rightError.value = false; }
function resetLeft() { leftError.value = false; }
function resetRight() { rightError.value = false; }
</script>

<template>
  <div class="p-4 max-w-[1200px] mx-auto">
    <h2 class="mb-4 text-[1.4em]">GIF 对比</h2>

    <div class="grid grid-cols-2 gap-4 mb-4">
      <div>
        <input
          v-model="leftUrl"
          placeholder="左侧 GIF URL"
          class="w-full py-2 px-3 border border-[#ddd] rounded-md text-sm outline-none focus:border-[#1976d2]"
          @input="resetLeft"
        />
      </div>
      <div>
        <input
          v-model="rightUrl"
          placeholder="右侧 GIF URL"
          class="w-full py-2 px-3 border border-[#ddd] rounded-md text-sm outline-none focus:border-[#1976d2]"
          @input="resetRight"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="bg-[#1a1a2e] rounded-lg flex items-center justify-center min-h-[300px] overflow-hidden">
        <img
          v-if="leftUrl"
          :src="leftUrl"
          alt="左侧 GIF"
          class="max-w-full max-h-[70vh] object-contain"
          @error="onLeftError"
        />
        <span v-else-if="leftError" class="text-[#e74c3c]">加载失败</span>
        <span v-else class="text-[#666]">输入 URL 预览</span>
      </div>
      <div class="bg-[#1a1a2e] rounded-lg flex items-center justify-center min-h-[300px] overflow-hidden">
        <img
          v-if="rightUrl"
          :src="rightUrl"
          alt="右侧 GIF"
          class="max-w-full max-h-[70vh] object-contain"
          @error="onRightError"
        />
        <span v-else-if="rightError" class="text-[#e74c3c]">加载失败</span>
        <span v-else class="text-[#666]">输入 URL 预览</span>
      </div>
    </div>
  </div>
</template>
