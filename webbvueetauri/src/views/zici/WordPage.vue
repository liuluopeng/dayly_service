<script setup>
import cnchar from 'cnchar-all';
import { onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

// 导入wasm初始化函数和函数
import { my_console_log, get_new_words } from '../../types/wasm-typed';

const { t } = useI18n();

// 标记wasm是否已初始化
const wasmInitialized = ref(false);

// 初始化wasm模块
onMounted(async () => {
  try {
    wasmInitialized.value = true;
    console.log('wasm模块初始化成功');
    // WASM初始化完成后，自动加载和绘制词语
    loadAndDrawWords();
  } catch (error) {
    console.error('wasm模块初始化失败:', error);
  }
});

// Gallery相关状态
const currentIndex = ref(0);
const currentWord = ref('');
const currentPinyin = ref('');
const validWords = ref([]);

let wordsArray = [];

// 初始化路由
const router = useRouter();

// 上一个词语
const previousWord = () => {
  if (currentIndex.value > 0) {
    currentIndex.value--;
    updateCurrentWord();
  }
};

// 下一个词语
const nextWord = () => {
  if (currentIndex.value < validWords.value.length - 1) {
    currentIndex.value++;
    updateCurrentWord();
  }
};

// 跳转到指定词语
const goToWord = (index) => {
  currentIndex.value = index;
  updateCurrentWord();
};

// 更新当前显示的词语
const updateCurrentWord = () => {
  const word = validWords.value[currentIndex.value];
  currentWord.value = word;
  currentPinyin.value = cnchar.spell(word, 'tone');
};

// 加载和绘制词语的函数
const loadAndDrawWords = () => {
  if (!wasmInitialized.value) {
    console.warn('wasm模块未初始化，跳过词语加载');
    return;
  }

  // 从wasm获取词语
  try {
    wordsArray = get_new_words();
    console.log('从wasm获取的词语数组:', wordsArray);
  } catch (error) {
    console.error('调用wasm函数失败:', error);
  }

  // 验证所有词语都是中文且长度为2-4个字
  const isValidWord = (word) => {
    return /^[\u4e00-\u9fa5]{2,4}$/.test(word);
  };

  // 过滤出有效的中文词语
  validWords.value = Array.from(wordsArray).filter(isValidWord);
  console.log('有效中文词语:', validWords.value);

  // 初始化显示第一个词语
  if (validWords.value.length > 0) {
    currentIndex.value = 0;
    updateCurrentWord();
  } else {
    console.error('从wasm获取词语失败');
    currentWord.value = t('wordPage.loadFailed');
    currentPinyin.value = '';
  }
};

// 发音指定的文本（词语）
const speakText = (text) => {
  if (text) {
    cnchar.voice.speak(text);
  }
};
</script>

<template>
  <div class="p-4 w-full box-border max-[480px]:p-[0.8rem]">
    <h1 class="text-[#333] text-center max-md:text-[1.8rem] max-md:mb-4 max-[480px]:text-[1.5rem]">{{ t('wordPage.title') }}</h1>

    <!-- 词语列表 -->
    <div class="my-6 text-center max-md:my-[1.2rem]">
      <h3 class="text-[1.3rem] text-[#333] mb-[0.8rem] max-md:text-xl max-md:mb-[0.6rem]">{{ t('wordPage.allWords') }}</h3>
      <div class="flex flex-wrap justify-center gap-2 max-w-[1200px] mx-auto p-[0.8rem] bg-[#f5f5f5] rounded-[0.8rem] max-md:gap-[0.4rem] max-md:p-[0.6rem]">
        <div v-for="(word, index) in validWords" :key="index"
          class="px-4 py-2 text-[1.2rem] bg-white border border-solid border-[#ddd] rounded-[0.4rem] cursor-pointer transition-all duration-300 min-w-[80px] text-center hover:border-[#44f] hover:bg-[#f0f0ff] max-md:px-[0.8rem] max-md:py-[0.4rem] max-md:text-base max-md:min-w-[60px] max-[480px]:min-w-[50px] max-[480px]:text-[0.9rem]"
          :class="{ '!border-[#44f] !bg-[#44f] !text-white !font-bold !shadow-[0_0_10px_rgba(68,68,255,0.5)]': index === currentIndex }"
          @click="goToWord(index)">
          {{ word }}
        </div>
      </div>
    </div>

    <!-- Gallery视图 -->
    <div class="flex justify-center items-center gap-8 my-8 flex-wrap max-md:gap-4 max-md:my-6">
      <!-- 导航按钮 -->
      <button class="px-8 py-4 text-[1.5rem] border-none rounded-lg bg-[#44f] text-white cursor-pointer transition-colors duration-300 hover:bg-[#33d] disabled:bg-[#ccc] disabled:cursor-not-allowed max-md:px-6 max-md:py-[0.8rem] max-md:text-[1.2rem] max-[480px]:px-[1.2rem] max-[480px]:py-[0.6rem] max-[480px]:text-base" @click="previousWord" :disabled="currentIndex === 0">
        {{ t('wordPage.prev') }}
      </button>

      <!-- 当前词语展示区域 -->
      <div class="flex flex-col items-center gap-8 p-8 bg-white rounded-2xl shadow-[0_0.5rem_1rem_rgba(0,0,0,0.1)] max-w-[800px] w-full max-md:p-6 max-md:gap-6 max-[480px]:p-4 max-[480px]:gap-4">
        <div class="text-center">
          <h1 class="text-[6rem] m-0 text-[#333] text-center max-md:text-[4rem] max-[480px]:text-[3rem]">{{ currentWord }}</h1>
          <h2 class="text-[2rem] text-[#666] mt-2 mx-0 mb-0 cursor-pointer max-md:text-[1.5rem] max-[480px]:text-[1.3rem]" @click="speakText(currentPinyin)">{{ currentPinyin }}</h2>
        </div>
      </div>

      <!-- 导航按钮 -->
      <button class="px-8 py-4 text-[1.5rem] border-none rounded-lg bg-[#44f] text-white cursor-pointer transition-colors duration-300 hover:bg-[#33d] disabled:bg-[#ccc] disabled:cursor-not-allowed max-md:px-6 max-md:py-[0.8rem] max-md:text-[1.2rem] max-[480px]:px-[1.2rem] max-[480px]:py-[0.6rem] max-[480px]:text-base" @click="nextWord" :disabled="currentIndex === validWords.length - 1">
        {{ t('wordPage.next') }}
      </button>
    </div>

    <!-- 进度指示器 -->
    <div class="text-center text-[1.2rem] text-[#666] mt-4 max-md:text-base max-md:mt-[0.8rem]">
      <span>{{ currentIndex + 1 }} / {{ validWords.length }}</span>
    </div>
  </div>
</template>
