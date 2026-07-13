<script setup lang="ts">
import { onMounted, onBeforeUnmount, reactive } from 'vue';

import { CAN_PRINT_KEY } from '../../../config/key';
import { useConfigStore } from '../../../store/config';
import { storeToRefs } from 'pinia';

const configStore = useConfigStore();
const { onlyShowMain } = storeToRefs(configStore);

defineProps({
  title: {
    type: String,
    default: ''
  },
  className: {
    type: String,
    default: ''
  }
});
const state = reactive({
  currentCode: '',
  keysPressed: {} as any
});
const handleKeyDown = (e: KeyboardEvent) => {
  const code = e.code;
  // 只拦截虚拟键盘实际处理的按键，不阻塞系统快捷键（CMD+/F12/缩放等）
  const isVirtualKey = code.startsWith('Key') || code.startsWith('Digit') || code === 'Enter' || code === 'Backspace' || code === 'Space';
  if (isVirtualKey) {
    e.preventDefault();
  }
  const capsLockOn = e.getModifierState('CapsLock');
  if (capsLockOn && CAN_PRINT_KEY[code]) {
    configStore.setPrintContent(CAN_PRINT_KEY[code].toUpperCase() || '');
  } else {
    configStore.setPrintContent(CAN_PRINT_KEY[code] || '');
  }

  if (code === 'Enter') {
    configStore.setPrintContent('<br/>');
  }
  if (code === 'Backspace') {
    configStore.minusPrintContent();
  }

  if (!state.keysPressed[code]) {
    console.log('----------', 'handleKeyDown', e, '----------cyy log');
    configStore.setCurrentCode(e.code);
    state.keysPressed[code] = true;
    state.currentCode = code;
  }
};

const handleKeyUp = (e: KeyboardEvent) => {
  console.log('----------', 'handleKeyUp', e, '----------cyy log');
  state.keysPressed[e.code] = false;
  state.currentCode = '';
};

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
  document.addEventListener('keyup', handleKeyUp);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeyDown);
  document.removeEventListener('keyup', handleKeyUp);
});
</script>
<template>
  <div class="y-key-wrap flex flex-col items-center" :class="className">
    <slot :keys-pressed="state.keysPressed"></slot>
  </div>
  <Transition name="menu">
    <div class="y-key-wrap__title text-center text-[#666666] text-[2rem] font-bold mt-[2rem]" v-show="!onlyShowMain">{{ title }}</div>
  </Transition>
</template>
