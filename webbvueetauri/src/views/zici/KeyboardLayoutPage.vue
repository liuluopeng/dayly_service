<script setup>
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import KeyboardAnimation from '../../components/zici/keyboard-animation/KeyboardAnimation.vue';
import cnchar from 'cnchar-all';

const { t } = useI18n();

const initialSounds = ref([]);
const finalSounds = ref([]);

async function loadPinyinData() {
  try {
    const base = (await import('../../types/wasm-typed')).get_base_url_wasm();
    const res = await fetch(`${base}/api/zici/pinyin`);
    const json = await res.json();
    const data = json.data || {};
    initialSounds.value = (data.initials || []).map((i) => ({ char: i.char, pinyin: i.pinyin, example: i.example, mp3Path: `/${i.pinyin}.mp3` }));
    finalSounds.value = (data.finals || []).map((i) => ({ char: i.char, pinyin: i.pinyin, example: i.example, mp3Path: `/${i.pinyin}.mp3` }));
  } catch (e) {
    console.error('拼音数据加载失败:', e);
  }
}
loadPinyinData();

const convertToKeyboardInput = (pinyin) => {
  return pinyin.replace(/ü/g, 'v');
};

const speak = (text, mp3Path) => {
  if (mp3Path) {
    const audio = new Audio(mp3Path);
    audio.play().catch(error => {
      console.log('MP3播放失败，使用cnchar发音:', error);
      if (text) {
        cnchar.voice.speak(text);
      }
    });
  } else if (text) {
    cnchar.voice.speak(text);
  }
};
</script>

<template>
  <div class="p-8 max-w-[1600px] mx-auto w-full box-border">
    <h1 class="text-center text-[#333] mb-8">{{ t('keyboardLayoutPage.pageTitle') }}</h1>

    <div class="flex gap-8 w-full box-border">
      <div class="flex-1 min-w-[300px] rounded-2xl p-6 shadow-[0_0.5rem_1rem_rgba(0,0,0,0.1)] bg-[#ccd2f5]">
        <h2 class="text-[#333] mb-4 text-2xl text-center">{{ t('keyboardLayoutPage.initialKeys') }}</h2>
        <div class="flex flex-col gap-6">
          <div v-for="item in initialSounds" :key="item.pinyin" class="flex flex-col items-center gap-2 bg-[#fafafa] rounded-[0.8rem] p-4 border border-b-2 border-[#e0e0e0]">
            <div class="text-[2.5rem] font-bold text-[#333] text-center py-[0.8rem] px-6 bg-[#f0f0f0] rounded-lg min-w-[120px] cursor-pointer transition-all duration-300 flex flex-col items-center gap-[0.3rem] touch-manipulation [-webkit-tap-highlight-color:transparent] hover:bg-[#e0e0ff] hover:scale-105" @click="speak(item.example, item.mp3Path)">
              {{ item.pinyin }}
              <span class="text-[1.8rem] text-[#666] font-normal">{{ item.example }}</span>
            </div>
            <KeyboardAnimation :input="convertToKeyboardInput(item.pinyin)" />
          </div>
        </div>
      </div>

      <div class="flex-1 min-w-[300px] rounded-2xl p-6 shadow-[0_0.5rem_1rem_rgba(0,0,0,0.1)] bg-[#f6fff0]">
        <h2 class="text-[#333] mb-4 text-2xl text-center">{{ t('keyboardLayoutPage.finalKeys') }}</h2>
        <div class="flex flex-col gap-6">
          <div v-for="item in finalSounds" :key="item.pinyin" class="flex flex-col items-center gap-2 bg-[#fafafa] rounded-[0.8rem] p-4 border border-b-2 border-[#e0e0e0]">
            <div class="text-[2.5rem] font-bold text-[#333] text-center py-[0.8rem] px-6 bg-[#f0f0f0] rounded-lg min-w-[120px] cursor-pointer transition-all duration-300 flex flex-col items-center gap-[0.3rem] touch-manipulation [-webkit-tap-highlight-color:transparent] hover:bg-[#e0e0ff] hover:scale-105" @click="speak(item.example, item.mp3Path)">
              {{ item.pinyin }}
              <span class="text-[1.8rem] text-[#666] font-normal">{{ item.example }}</span>
            </div>
            <KeyboardAnimation :input="convertToKeyboardInput(item.pinyin)" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
