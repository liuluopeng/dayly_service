<script setup>
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import KeyboardAnimation from '../../components/zici/keyboard-animation/KeyboardAnimation.vue';
import cnchar from 'cnchar-all';

const { t } = useI18n();

const initialSounds = [
  { char: 'b', pinyin: 'b', example: '波', mp3Path: '/b.mp3' },
  { char: 'p', pinyin: 'p', example: '坡', mp3Path: '/p.mp3' },
  { char: 'm', pinyin: 'm', example: '摸', mp3Path: '/m.mp3' },
  { char: 'f', pinyin: 'f', example: '佛', mp3Path: '/f.mp3' },
  { char: 'd', pinyin: 'd', example: '的', mp3Path: '/d.mp3' },
  { char: 't', pinyin: 't', example: '特', mp3Path: '/t.mp3' },
  { char: 'n', pinyin: 'n', example: '呢', mp3Path: '/n.mp3' },
  { char: 'l', pinyin: 'l', example: '乐', mp3Path: '/l.mp3' },
  { char: 'g', pinyin: 'g', example: '哥', mp3Path: '/g.mp3' },
  { char: 'k', pinyin: 'k', example: '科', mp3Path: '/k.mp3' },
  { char: 'h', pinyin: 'h', example: '喝', mp3Path: '/h.mp3' },
  { char: 'j', pinyin: 'j', example: '鸡', mp3Path: '/j.mp3' },
  { char: 'q', pinyin: 'q', example: '七', mp3Path: '/q.mp3' },
  { char: 'x', pinyin: 'x', example: '希', mp3Path: '/x.mp3' },
  { char: 'zh', pinyin: 'zh', example: '知', mp3Path: '/zh.mp3' },
  { char: 'ch', pinyin: 'ch', example: '吃', mp3Path: '/ch.mp3' },
  { char: 'sh', pinyin: 'sh', example: '狮', mp3Path: '/sh.mp3' },
  { char: 'r', pinyin: 'r', example: '日', mp3Path: '/r.mp3' },
  { char: 'z', pinyin: 'z', example: '资', mp3Path: '/z.mp3' },
  { char: 'c', pinyin: 'c', example: '疵', mp3Path: '/c.mp3' },
  { char: 's', pinyin: 's', example: '撕', mp3Path: '/s.mp3' },
  { char: 'y', pinyin: 'y', example: '医', mp3Path: '/y.mp3' },
  { char: 'w', pinyin: 'w', example: '乌', mp3Path: '/w.mp3' }
];

const finalSounds = [
  { char: 'a', pinyin: 'a', example: '啊', mp3Path: '/a.mp3' },
  { char: 'o', pinyin: 'o', example: '窝', mp3Path: '/o.mp3' },
  { char: 'e', pinyin: 'e', example: '鹅', mp3Path: '/e.mp3' },
  { char: 'i', pinyin: 'i', example: '衣', mp3Path: '/i.mp3' },
  { char: 'u', pinyin: 'u', example: '乌', mp3Path: '/u.mp3' },
  { char: 'ü', pinyin: 'ü', example: '迂', mp3Path: '/yu1.mp3' },
  { char: 'ai', pinyin: 'ai', example: '爱', mp3Path: '/ai.mp3' },
  { char: 'ei', pinyin: 'ei', example: '飞', mp3Path: '/ei.mp3' },
  { char: 'ui', pinyin: 'ui', example: '水', mp3Path: '/ui.mp3' },
  { char: 'ao', pinyin: 'ao', example: '奥', mp3Path: '/ao.mp3' },
  { char: 'ou', pinyin: 'ou', example: '欧', mp3Path: '/ou.mp3' },
  { char: 'iu', pinyin: 'iu', example: '牛', mp3Path: '/iu.mp3' },
  { char: 'ie', pinyin: 'ie', example: '姐', mp3Path: '/ie.mp3' },
  { char: 'ue', pinyin: 'ue', example: '月', mp3Path: '/yue1.mp3' },
  { char: 'er', pinyin: 'er', example: '儿', mp3Path: '/er.mp3' },
  { char: 'an', pinyin: 'an', example: '安', mp3Path: '/an.mp3' },
  { char: 'en', pinyin: 'en', example: '恩', mp3Path: '/en.mp3' },
  { char: 'in', pinyin: 'in', example: '音', mp3Path: '/in.mp3' },
  { char: 'un', pinyin: 'un', example: '云', mp3Path: '/un.mp3' },
  { char: 'ang', pinyin: 'ang', example: '昂', mp3Path: '/ang.mp3' },
  { char: 'eng', pinyin: 'eng', example: '风', mp3Path: '/eng.mp3' },
  { char: 'ing', pinyin: 'ing', example: '英', mp3Path: '/ing.mp3' },
  { char: 'ong', pinyin: 'ong', example: '东', mp3Path: '/ong.mp3' }
];

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
