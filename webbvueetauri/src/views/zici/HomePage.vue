<script setup>
import cnchar from 'cnchar-all';
import { onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
// 导入键盘遮罩组件
import KeyboardMask from '../../components/zici/keyboard-mask/KeyboardMask.vue';

// 导入wasm初始化函数和函数
import { my_console_log, get_new_chars } from '../../types/wasm-typed';

const { t } = useI18n();

// 标记wasm是否已初始化
const wasmInitialized = ref(false);

// 初始化wasm模块
onMounted(async () => {
  try {
    wasmInitialized.value = true;
    console.log('wasm模块初始化成功');

    // 页面加载时从路由参数获取学期选择
    const routeTerm = router.currentRoute.value.query.term;
    if (routeTerm) {
      const [grade, term] = String(routeTerm).split('-').map(Number);
      currentGrade.value = grade;
      currentTerm.value = term;
    }

    // WASM初始化完成后，自动加载和绘制汉字
    loadAndDrawCharacters();
  } catch (error) {
    console.error('wasm模块初始化失败:', error);
  }
});





// 学期选择
const currentGrade = ref(6);
const currentTerm = ref(2);

// Gallery相关状态
const currentIndex = ref(0);
const currentCharacter = ref('');
const currentPinyin = ref('');
const currentWords = ref([]);
const validCharacters = ref([]);

let charactersArray = [];

// 初始化路由
const router = useRouter();

// 移除了不再需要的键盘遮罩打开和关闭函数
// 现在直接在组件中传递当前字符

// 上一个汉字
const previousChar = () => {
  if (currentIndex.value > 0) {
    currentIndex.value--;
    updateCurrentCharacter();
  }
};

// 下一个汉字
const nextChar = () => {
  if (currentIndex.value < validCharacters.value.length - 1) {
    currentIndex.value++;
    updateCurrentCharacter();
  }
};

// 跳转到指定汉字
const goToCharacter = (index) => {
  currentIndex.value = index;
  updateCurrentCharacter();
};

// 更新当前显示的汉字
const updateCurrentCharacter = () => {
  const char = validCharacters.value[currentIndex.value];
  currentCharacter.value = char;
  currentPinyin.value = cnchar.spell(char, 'tone');

  // 获取组词
  const words = cnchar.words(char);
  if (words && words.length > 0) {
    // 先打乱词语数组，然后取前10个
    const shuffledWords = [...words];
    for (let i = shuffledWords.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffledWords[i], shuffledWords[j]] = [shuffledWords[j], shuffledWords[i]];
    }
    currentWords.value = shuffledWords.slice(0, 10);
  } else {
    currentWords.value = [];
  }

  // 重绘当前汉字的笔画
  drawCurrentCharacter();
};

// 绘制当前汉字的笔画
const drawCurrentCharacter = () => {
  const drawArea = document.getElementById('draw-area');
  if (!drawArea) return;

  // 清空绘制区域
  drawArea.innerHTML = '';

  // 创建新的绘制容器
  const charContainer = document.createElement('div');
  charContainer.style.textAlign = 'center';
  charContainer.style.padding = '1rem';
  drawArea.appendChild(charContainer);

  var option = {
    clear: false,
    el: charContainer,
    style: {
      radicalColor: '#44f',
      backgroundColor: '#eee',
      length: 150
    },
    type: cnchar.draw.TYPE.ANIMATION,
    animation: {
      strokeAnimationSpeed: 2.5,
      delayBetweenStrokes: 1,
      loopAnimate: true,
    }
  };

  try {
    cnchar.draw(currentCharacter.value, option);
  } catch (error) {
    console.error('绘制字符失败:', currentCharacter.value, error);
  }
};

// 学期选择变化时的处理函数
const handleTermChange = (event) => {
  // 从选中的值解析出年级和学期
  const [grade, term] = event.target.value.split('-').map(Number);
  currentGrade.value = grade;
  currentTerm.value = term;
  loadAndDrawCharacters();
};

// 监听年级或学期变化，更新路由参数
watch([currentGrade, currentTerm], ([newGrade, newTerm]) => {
  // 更新当前页面的路由参数
  router.replace({
    query: {
      term: `${newGrade}-${newTerm}`
    }
  });
});

// 加载和绘制汉字的函数
const loadAndDrawCharacters = () => {
  if (!wasmInitialized.value) {
    console.warn('wasm模块未初始化，跳过汉字加载');
    return;
  }

  // 从wasm获取汉字
  try {
    charactersArray = get_new_chars(currentGrade.value, currentTerm.value);
    console.log('从wasm获取的汉字数组:', charactersArray);
  } catch (error) {
    console.error('调用wasm函数失败:', error);
  }

  // 验证所有字符都是中文
  const isValidChinese = (char) => {
    return /^[一-龥]$/.test(char);
  };

  // 过滤出有效的中文字符
  validCharacters.value = Array.from(charactersArray).filter(isValidChinese);
  console.log('有效中文字符:', validCharacters.value);

  // 初始化显示第一个汉字
  if (validCharacters.value.length > 0) {
    currentIndex.value = 0;
    updateCurrentCharacter();
  } else {
    console.error('从wasm获取汉字失败');
    currentCharacter.value = t('ziciHome.loadFailed');
    currentPinyin.value = '';
    currentWords.value = [];
    drawCurrentCharacter();
  }
};

// 发音指定的文本（单个字符或词语）
const speakText = (text) => {
  if (text) {
    cnchar.voice.speak(text);
  }
};
</script>

<template>
  <div class="p-[0.8rem] sm:p-4 w-full box-border">
    <h1 class="text-[1.5rem] [480px]:text-[1.8rem] sm:text-base text-[#333] text-center mb-0 [480px]:mb-4 sm:mb-0">{{ t('ziciHome.title', { grade: currentGrade, term: currentTerm === 1 ? t('ziciHome.termUp') : t('ziciHome.termDown') }) }}</h1>

    <!-- 学期选择框 -->
    <div class="flex justify-center my-6 [480px]:my-6 sm:my-8 p-4 [480px]:p-4 sm:p-6 bg-white rounded-[0.8rem] shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)]">
      <select
        :value="`${currentGrade}-${currentTerm}`"
        @change="handleTermChange"
        class="py-[1rem] px-[1.5rem] [480px]:py-[1rem] [480px]:px-[1.5rem] sm:py-[1.2rem] sm:px-8 text-[1.3rem] [480px]:text-[1.5rem] sm:text-[1.8rem] border border-[#ddd] rounded-[0.6rem] bg-white cursor-pointer transition-all duration-300 min-w-[14rem] [480px]:min-w-[16rem] sm:min-w-[20rem] hover:border-[#44f] focus:outline-none focus:border-[#44f] focus:shadow-[0_0_0_2px_rgba(68,68,255,0.2)]"
      >
        <option value="1-1">{{ t('ziciHome.gradeTerm', { grade: 1, term: t('ziciHome.termUp') }) }}</option>
        <option value="1-2">{{ t('ziciHome.gradeTerm', { grade: 1, term: t('ziciHome.termDown') }) }}</option>
        <option value="2-1">{{ t('ziciHome.gradeTerm', { grade: 2, term: t('ziciHome.termUp') }) }}</option>
        <option value="2-2">{{ t('ziciHome.gradeTerm', { grade: 2, term: t('ziciHome.termDown') }) }}</option>
        <option value="3-1">{{ t('ziciHome.gradeTerm', { grade: 3, term: t('ziciHome.termUp') }) }}</option>
        <option value="3-2">{{ t('ziciHome.gradeTerm', { grade: 3, term: t('ziciHome.termDown') }) }}</option>
        <option value="4-1">{{ t('ziciHome.gradeTerm', { grade: 4, term: t('ziciHome.termUp') }) }}</option>
        <option value="4-2">{{ t('ziciHome.gradeTerm', { grade: 4, term: t('ziciHome.termDown') }) }}</option>
        <option value="5-1">{{ t('ziciHome.gradeTerm', { grade: 5, term: t('ziciHome.termUp') }) }}</option>
        <option value="5-2">{{ t('ziciHome.gradeTerm', { grade: 5, term: t('ziciHome.termDown') }) }}</option>
        <option value="6-1">{{ t('ziciHome.gradeTerm', { grade: 6, term: t('ziciHome.termUp') }) }}</option>
        <option value="6-2">{{ t('ziciHome.gradeTerm', { grade: 6, term: t('ziciHome.termDown') }) }}</option>
      </select>
    </div>

    <!-- 汉字列表 -->
    <div class="my-6 text-center">
      <h3 class="text-[1.3rem] text-[#333] mb-[0.8rem]">{{ t('ziciHome.allChars') }}</h3>
      <div class="flex flex-wrap justify-center gap-[0.2rem] sm:gap-[0.3rem] max-w-[1200px] mx-auto p-[0.6rem] sm:p-[0.8rem] bg-[#f5f5f5] rounded-[0.8rem]">
        <div
          v-for="(char, index) in validCharacters"
          :key="index"
          class="py-[0.4rem] px-[0.6rem] sm:py-2 sm:px-[0.8rem] text-[1rem] sm:text-[1.2rem] bg-white border border-[#ddd] rounded-[0.4rem] cursor-pointer transition-all duration-300 min-w-[30px] sm:min-w-[35px] text-center hover:border-[#44f] hover:bg-[#f0f0ff]"
          :class="{ 'border-[#44f] bg-[#44f] text-white font-bold shadow-[0_0_10px_rgba(68,68,255,0.5)]': index === currentIndex }"
          @click="goToCharacter(index)"
        >
          {{ char }}
        </div>
      </div>
    </div>

    <!-- Gallery视图 -->
    <div class="flex justify-center items-center gap-4 [480px]:gap-4 sm:gap-8 my-6 [480px]:my-6 sm:my-8 flex-wrap">
      <!-- 导航按钮 -->
      <button
        class="py-[0.8rem] px-[1.5rem] sm:py-4 sm:px-8 text-[1.2rem] sm:text-[1.5rem] border-none rounded-[0.5rem] bg-[#44f] text-white cursor-pointer transition-colors duration-300 hover:bg-[#33d] disabled:bg-[#ccc] disabled:cursor-not-allowed"
        @click="previousChar"
        :disabled="currentIndex === 0"
      >
        {{ t('ziciHome.prev') }}
      </button>

      <!-- 当前汉字展示区域 -->
      <div class="flex flex-col items-center gap-[1rem] [480px]:gap-4 sm:gap-8 p-[1rem] [480px]:p-[1.5rem] sm:p-8 bg-white rounded-2xl shadow-[0_0.5rem_1rem_rgba(0,0,0,0.1)] max-w-[800px] w-full">
        <div class="text-center">
          <h1 class="text-[3.5rem] [480px]:text-[4rem] sm:text-[6rem] m-0 text-[#333]">{{ currentCharacter }}</h1>
          <h2
            class="text-[1.3rem] [480px]:text-[1.5rem] sm:text-[2rem] text-[#666] mt-2 mb-0 ml-0 mr-0 cursor-pointer"
            @click="speakText(currentPinyin)"
          >{{ currentPinyin }}</h2>
        </div>

        <!-- 笔画和键盘并排展示 -->
        <div class="flex gap-[1.5rem] sm:gap-8 w-full flex-wrap justify-center">
          <!-- 笔画展示 -->
          <div class="flex-1 min-w-full sm:min-w-[300px] text-center">
            <h3 class="text-[1.3rem] sm:text-[1.5rem] text-[#333] mt-0 mb-4 ml-0 mr-0">{{ t('ziciHome.strokeDemo') }}</h3>
            <div
              id="draw-area"
              class="mt-[1.5rem] sm:mt-8 p-[1.5rem] sm:p-8 bg-[#f5f5f5] rounded-[0.8rem] flex justify-center items-center min-h-[200px] [480px]:min-h-[250px] sm:min-h-[300px]"
            ></div>
          </div>

          <!-- 键盘展示 -->
          <div class="flex-1 min-w-full sm:min-w-[300px] text-center mt-0">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-[1.3rem] sm:text-[1.5rem] text-[#333] mt-0 mb-4 ml-0 mr-0">{{ t('ziciHome.pinyinKeyboard') }}</h3>
            </div>
            <KeyboardMask :show="true" :char="currentCharacter" :click-x="0" :click-y="0" />
          </div>
        </div>

        <!-- 组词展示 -->
        <div class="w-full text-center">
          <h3 class="text-[1.3rem] sm:text-[1.5rem] text-[#333] mt-0 mb-4 ml-0 mr-0">{{ t('ziciHome.wordFormation') }}</h3>
          <div class="flex flex-wrap justify-center gap-[0.8rem] sm:gap-4 mt-4">
            <div
              v-for="(word, index) in currentWords"
              :key="index"
              class="py-[0.4rem] px-[0.8rem] sm:py-2 sm:px-4 bg-[#f0f0f0] rounded-[0.5rem] text-[1rem] sm:text-[1.2rem] text-[#333] cursor-pointer"
              @click="speakText(word)"
            >
              {{ word }}
            </div>
            <div v-if="currentWords.length === 0" class="text-[#999] italic">
              {{ t('ziciHome.noWords') }}
            </div>
          </div>
        </div>
      </div>

      <!-- 导航按钮 -->
      <button
        class="py-[0.8rem] px-[1.5rem] sm:py-4 sm:px-8 text-[1.2rem] sm:text-[1.5rem] border-none rounded-[0.5rem] bg-[#44f] text-white cursor-pointer transition-colors duration-300 hover:bg-[#33d] disabled:bg-[#ccc] disabled:cursor-not-allowed"
        @click="nextChar"
        :disabled="currentIndex === validCharacters.length - 1"
      >
        {{ t('ziciHome.next') }}
      </button>
    </div>

    <!-- 进度指示器 -->
    <div class="text-center text-[1.2rem] text-[#666] mt-4">
      <span>{{ currentIndex + 1 }} / {{ validCharacters.length }}</span>
    </div>
  </div>
</template>
