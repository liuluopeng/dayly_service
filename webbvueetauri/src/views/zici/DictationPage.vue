<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import cnchar from 'cnchar-all';
import HanziWriter from 'hanzi-writer';
// 导入键盘遮罩组件
import KeyboardMask from '../../components/zici/keyboard-mask/KeyboardMask.vue';

// 导入wasm初始化函数和函数
import { get_new_chars } from '../../types/wasm-typed';

const { t } = useI18n();

// 标记wasm是否已初始化
const wasmInitialized = ref(false);

// 测验是否正确
const quiz_is_right = ref(false);
// 当前汉字
const currentChar = ref('');
// 拼音
const currentPinyin = ref('');
// 词语列表
const topWords = ref<string[]>([]);
// 历史选择的汉字
const history_chars = ref<string[]>([]);
// 当前在历史记录中的位置
const current_char_index = ref(-1);
// HanziWriter引用
const writerContainer = ref<HTMLElement | null>(null);
let writer: any = null;



// 通知相关变量
const notification = ref({ visible: false, message: '', type: 'info' as 'success' | 'error' | 'info' });

// 键盘遮罩状态
const showKeyboardMask = ref(false);
const clickX = ref(0);
const clickY = ref(0);

// 初始化路由
const route = useRoute();

// 学期信息
const currentGrade = ref(6);
const currentTerm = ref(2);

// 从路由参数获取学期信息
const getTermFromRoute = () => {
  const termParam = route.query.term as string;
  if (termParam && /^[1-6]-[1-2]$/.test(termParam)) {
    const [grade, term] = termParam.split('-').map(Number);
    currentGrade.value = grade;
    currentTerm.value = term;
    return true;
  }
  return false;
};

// 打开键盘遮罩
const openKeyboardMask = (event: MouseEvent) => {
  if (!currentChar.value) return;
  toggleAnswer();
  // 记录点击位置
  clickX.value = event.clientX;
  clickY.value = event.clientY;
  showKeyboardMask.value = true;
};

// 关闭键盘遮罩
const closeKeyboardMask = () => {
  showKeyboardMask.value = false;
};

// 从wasm加载当前学期的汉字
const getCurrentTermChars = () => {
  if (!wasmInitialized.value) {
    console.warn('wasm模块未初始化，跳过汉字加载');
    return [];
  }

  try {
    getTermFromRoute(); // 从路由获取学期信息
    // 调用wasm的get_new_chars函数获取对应学期的汉字
    const chars = get_new_chars(currentGrade.value, currentTerm.value);
    // 确保返回的是数组
    if (Array.isArray(chars) && chars.length > 0) {
      return chars;
    } else {
      console.warn('未获取到汉字数据，可能是学期参数有误');
      return [];
    }
  } catch (error) {
    console.error('加载汉字失败:', error);
    return [];
  }
};

// 初始化加载第一个汉字
const loadFirstChar = () => {
  getTermFromRoute();
  selectRandomChar();
};

// 处理词语中的汉字，将与当前测验汉字相同的字替换为田字格
const processWord = (word: string): { type: 'char' | 'tianzige'; value: string }[] => {
  return word.split('').map(char => {
    if (char === currentChar.value) {
      return { type: 'tianzige', value: char };
    } else {
      return { type: 'char', value: char };
    }
  });
};

// 随机选择一个汉字
const selectRandomChar = () => {
  // 如果当前显示答案，先翻回正面
  if (quiz_is_right.value) {
    quiz_is_right.value = false;
    // 等待翻转动画完成后再更新内容（CSS transition时间为0.6s）
    setTimeout(() => {
      loadNextChar();
    }, 600);
    return;
  }

  loadNextChar();
};

// 加载下一个汉字的实际逻辑
const loadNextChar = () => {
  // 获取当前学期的汉字
  const chars = getCurrentTermChars();
  if (chars.length === 0) return;

  // 随机选择一个汉字
  const randomIndex = Math.floor(Math.random() * chars.length);
  const randomChar = chars[randomIndex];

  // 如果当前不是最后一个历史记录，则截断后续记录
  if (current_char_index.value < history_chars.value.length - 1) {
    history_chars.value = history_chars.value.slice(0, current_char_index.value + 1);
  }

  // 添加到历史记录
  history_chars.value.push(randomChar);
  // 更新当前位置
  current_char_index.value = history_chars.value.length - 1;

  // 设置当前汉字
  currentChar.value = randomChar;

  // 同时继续执行原来的词语朗读功能
  const words = cnchar.words(currentChar.value);

  if (words && words.length > 0) {
    // 先打乱词语数组，然后取前3个
    const shuffledWords = [...words];
    // 使用Fisher-Yates洗牌算法打乱数组
    for (let i = shuffledWords.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffledWords[i], shuffledWords[j]] = [shuffledWords[j], shuffledWords[i]];
    }
    topWords.value = shuffledWords.slice(0, 3);

    currentPinyin.value = String(cnchar.spell(currentChar.value));
    quiz_is_right.value = false;
  };
};

// 下一个随机汉字
const nextRandomChar = () => {
  selectRandomChar();
};

// 上一个汉字
const prevChar = () => {
  if (current_char_index.value <= 0) return;

  // 如果当前显示答案，先翻回正面
  if (quiz_is_right.value) {
    quiz_is_right.value = false;
    // 等待翻转动画完成后再更新内容（CSS transition时间为0.6s）
    setTimeout(() => {
      loadPrevChar();
    }, 600);
    return;
  }

  loadPrevChar();
};

// 加载上一个汉字的实际逻辑
const loadPrevChar = () => {
  current_char_index.value--;
  const prevChar = history_chars.value[current_char_index.value];
  currentChar.value = prevChar;
  currentPinyin.value = String(cnchar.spell(currentChar.value));

  // 获取上一个汉字的词语
  const words = cnchar.words(currentChar.value);
  if (words && words.length > 0) {
    const shuffledWords = [...words];
    for (let i = shuffledWords.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffledWords[i], shuffledWords[j]] = [shuffledWords[j], shuffledWords[i]];
    }
    topWords.value = shuffledWords.slice(0, 3);
  } else {
    topWords.value = [];
  }

  quiz_is_right.value = false;
};

// 发音
const speakChar = () => {
  console.log('currentChar.value', currentChar.value);
  if (currentChar.value) {
    cnchar.voice.speak(currentChar.value);
  }
};

// 发音指定的文本（单个字符或词语）
const speakText = (text: string) => {
  if (text) {
    cnchar.voice.speak(text);
  }
};

// 显示答案
const toggleAnswer = () => {
  quiz_is_right.value = !quiz_is_right.value;
};



// 显示通知
const showNotification = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  notification.value = { visible: true, message, type };
  // 1秒后自动隐藏通知
  setTimeout(() => {
    notification.value.visible = false;
  }, 1000);
};

// 初始化HanziWriter
const initHanziWriter = () => {
  if (!writerContainer.value || !currentChar.value) return;

  // 如果已有writer实例，先销毁
  if (writer) {
    try {
      // 检查destroy方法是否存在
      if (typeof writer.destroy === 'function') {
        writer.destroy();
      } else {
        // 如果没有destroy方法，尝试其他可能的清理方式
        console.log('destroy method not found, using alternative cleanup');
        // 移除事件监听器
        writer.off && writer.off('quiz-complete');
        // 清空容器内容
        if (writerContainer.value) {
          writerContainer.value.innerHTML = '';
        }
      }
    } catch (error) {
      console.error('Error destroying writer instance:', error);
    } finally {
      // 确保writer被重置为null
      writer = null;
    }
  }

  // 动态计算HanziWriter的尺寸，基于容器的实际大小
  const containerRect = writerContainer.value.getBoundingClientRect();
  const size = Math.min(containerRect.width, containerRect.height);

  // 创建新的writer实例
  writer = HanziWriter.create(writerContainer.value, currentChar.value, {
    width: size,
    height: size,
    showCharacter: false,
    padding: 0,
    strokeAnimationSpeed: 2,
    delayBetweenStrokes: 50,
    showOutline: false,
    highlightOnComplete: false,
  });

  // 开始测验
  writer.quiz({
    onMistake: (strokeData: any) => {
      console.log('Oh no! you made a mistake on stroke ' + strokeData.strokeNum);
      console.log("You've made " + strokeData.mistakesOnStroke + " mistakes on this stroke so far");
      console.log("You've made " + strokeData.totalMistakes + " total mistakes on this quiz");
      console.log("There are " + strokeData.strokesRemaining + " strokes remaining in this character");
      showNotification(t('dictation.mistakeNotification', { stroke: strokeData.strokeNum }), 'error');
    },
    onCorrectStroke: (strokeData: any) => {
      console.log('Yes!!! You got stroke ' + strokeData.strokeNum + ' correct!');
      console.log('You made ' + strokeData.mistakesOnStroke + ' mistakes on this stroke');
      console.log("You've made " + strokeData.totalMistakes + ' total mistakes on this quiz');
      console.log('There are ' + strokeData.strokesRemaining + ' strokes remaining in this character');
    },
    onComplete: (summaryData: any) => {
      console.log('You did it! You finished drawing ' + summaryData.character);
      console.log('You made ' + summaryData.totalMistakes + ' total mistakes on this quiz');
      showNotification(t('dictation.completeNotification', { char: summaryData.character }), 'success');
      // 测验完成，翻转卡片显示答案
      quiz_is_right.value = true;
    }
  });
};

// 监听当前汉字变化，更新HanziWriter
watch(currentChar, (newChar) => {
  if (newChar && writerContainer.value) {
    quiz_is_right.value = false;
    // 延迟一下确保DOM已更新
    setTimeout(() => {
      initHanziWriter();
    }, 0);
  }
});

// 监听路由参数变化，当学期参数变化时重新加载汉字
watch(() => route.query.term, (newTerm) => {
  if (newTerm) {
    // 清空历史记录
    history_chars.value = [];
    current_char_index.value = -1;
    loadFirstChar();
  }
});

// 处理窗口大小变化
const handleResize = () => {
  if (writer && currentChar.value) {
    // 销毁当前实例
    if (typeof writer.destroy === 'function') {
      writer.destroy();
    }
    writer = null;
    // 重新初始化
    setTimeout(() => {
      initHanziWriter();
    }, 100);
  }
};

// 组件挂载时加载汉字
onMounted(async () => {
  try {
    wasmInitialized.value = true;
    console.log('wasm模块初始化成功');
  } catch (error) {
    console.error('wasm模块初始化失败:', error);
  }
  loadFirstChar();

  // 添加窗口大小变化监听
  window.addEventListener('resize', handleResize);
});

// 组件卸载时清理资源
onUnmounted(() => {
  // 移除窗口大小变化监听
  window.removeEventListener('resize', handleResize);

  // 销毁writer实例
  if (writer) {
    try {
      if (typeof writer.destroy === 'function') {
        writer.destroy();
      } else {
        writer.off && writer.off('quiz-complete');
        if (writerContainer.value) {
          writerContainer.value.innerHTML = '';
        }
      }
    } catch (error) {
      console.error('Error destroying writer instance on unmount:', error);
    } finally {
      writer = null;
    }
  }
});
</script>

<template>
  <div class="min-h-full flex flex-col items-center justify-start p-8 pb-0 bg-[#f8f9fa] font-['KaiTi_SC','KaiTi','STKaiti','SimKai',cursive,sans-serif] sm:p-4 md:w-full md:min-h-full md:overflow-hidden md:box-border max-[480px]:p-3.2">
    <!-- 通知组件 -->
    <div v-if="notification.visible"
      class="fixed top-8 right-8 p-6 pr-8 rounded-[0.8rem] text-[1.1rem] font-semibold text-white z-[10000] opacity-95 pointer-events-none shadow-[0_4px_12px_rgba(0,0,0,0.3)] transition-all duration-300 ease-in-out sm:top-4 sm:right-4 sm:p-4 sm:pr-6 sm:text-base"
      :class="{
        'bg-[#2ecc71]': notification.type === 'success',
        'bg-[#e74c3c]': notification.type === 'error',
        'bg-[#3498db]': notification.type === 'info'
      }">
      {{ notification.message }}
    </div>

    <div class="flex items-center justify-between max-w-[1200px] w-full gap-8 sm:flex-row sm:justify-between sm:gap-4 sm:mb-4">
      <!-- 上一个按钮 -->
      <button
        class="p-6 pr-12 pl-12 border-none rounded-2xl text-xl cursor-pointer transition-all duration-300 ease-in-out font-semibold min-w-[16rem] flex items-center justify-center gap-3.2 bg-[#95a5a6] text-white hover:bg-[#7f8c8d] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(149,165,166,0.4)] active:translate-y-0 sm:min-w-32 sm:p-4 sm:pr-6 sm:pl-6 sm:text-base max-[480px]:min-w-[16rem] max-[480px]:p-4.8 max-[480px]:pr-12 max-[480px]:pl-12 max-[480px]:text-base"
        @click="prevChar" :title="t('dictation.prev')">
        ⬅️ {{ t('dictation.prev') }}
      </button>

      <!-- 下一个按钮 -->
      <button
        class="p-6 pr-12 pl-12 border-none rounded-2xl text-xl cursor-pointer transition-all duration-300 ease-in-out font-semibold min-w-[16rem] flex items-center justify-center gap-3.2 bg-[#3498db] text-white hover:bg-[#2980b9] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(52,152,219,0.4)] active:translate-y-0 sm:min-w-32 sm:p-4 sm:pr-6 sm:pl-6 sm:text-base max-[480px]:min-w-[16rem] max-[480px]:p-4.8 max-[480px]:pr-12 max-[480px]:pl-12 max-[480px]:text-base"
        @click="nextRandomChar" :title="t('dictation.next')">
        ➡️ {{ t('dictation.next') }}
      </button>
    </div>


    <!-- 主要功能按钮区域 -->
    <div class="flex gap-4 justify-center items-center sm:flex-row sm:flex-wrap sm:gap-3.2 max-[480px]:flex-col max-[480px]:items-center">
      <button
        class="p-6 pr-12 pl-12 border-none rounded-2xl text-xl cursor-pointer transition-all duration-300 ease-in-out font-semibold min-w-[16rem] flex items-center justify-center gap-3.2 bg-[#e74c3c] text-white hover:bg-[#c0392b] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(231,76,60,0.4)] active:translate-y-0 sm:min-w-32 sm:p-4 sm:pr-6 sm:pl-6 sm:text-base max-[480px]:min-w-[16rem] max-[480px]:p-4.8 max-[480px]:pr-12 max-[480px]:pl-12 max-[480px]:text-base"
        @click="speakChar" :title="t('dictation.listenPronunciation')">
        🔊 {{ t('dictation.listenPronunciation') }}
      </button>
      <button
        class="p-6 pr-12 pl-12 border-none rounded-2xl text-xl cursor-pointer transition-all duration-300 ease-in-out font-semibold min-w-[16rem] flex items-center justify-center gap-3.2 bg-[#3498db] text-white hover:bg-[#2980b9] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(52,152,219,0.4)] active:translate-y-0 sm:min-w-32 sm:p-4 sm:pr-6 sm:pl-6 sm:text-base max-[480px]:min-w-[16rem] max-[480px]:p-4.8 max-[480px]:pr-12 max-[480px]:pl-12 max-[480px]:text-base"
        @click="toggleAnswer" :title="t('dictation.viewAnswer')">
        👁️ {{ t('dictation.viewAnswer') }}
      </button>
      <button
        class="p-6 pr-12 pl-12 border-none rounded-2xl text-xl cursor-pointer transition-all duration-300 ease-in-out font-semibold min-w-[16rem] flex items-center justify-center gap-3.2 bg-[#27ae60] text-white hover:bg-[#229954] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(39,174,96,0.4)] active:translate-y-0 sm:min-w-32 sm:p-4 sm:pr-6 sm:pl-6 sm:text-base max-[480px]:min-w-[16rem] max-[480px]:p-4.8 max-[480px]:pr-12 max-[480px]:pl-12 max-[480px]:text-base"
        @click="openKeyboardMask" :title="t('dictation.viewKeys')">
        ⌨️ {{ t('dictation.viewKeys') }}
      </button>
    </div>

    <div class="flex flex-row gap-6 mt-4 sm:flex-col sm:gap-4 sm:mt-4">
      <div class="text-[1.8rem] text-[#34495e] text-center min-h-12 font-medium p-4 pr-8 pl-8 rounded-[0.8rem] bg-[#ecf0f1] flex items-center gap-1.6 sm:text-xl sm:p-3.2 sm:pr-6 sm:pl-6 max-[480px]:text-[1.1rem]"
        v-for="(word, index) in topWords" :key="index" @click="speakText(word)">
        <span v-for="(item, charIndex) in processWord(word)" :key="charIndex"
          class="inline-flex flex-col items-center relative justify-start min-h-[calc(1.8rem_+_1rem_+_2px)] [&>span]:inline-flex [&>span]:items-center [&>span]:gap-0.5">
          <span class="text-[0.8rem] text-[#666] mb-0.5 leading-4 whitespace-nowrap text-center h-4 w-full flex items-center justify-center shrink-0 font-['Helvetica_Neue',Arial,'PingFang_SC','Hiragino_Sans_GB','Microsoft_YaHei',sans-serif]">{{ cnchar.spell(item.value, 'tone') }}</span>
          <span v-if="item.type === 'char'"
            class="inline-block align-middle leading-[1.8rem] h-[1.8rem] w-[1.8rem] text-center shrink-0">
            {{ item.value }}
          </span>
          <div v-else
            class="tianzige w-[1.8rem] h-[1.8rem] border-[0.2rem] border-dashed border-[rgba(52,73,94,0.5)] relative inline-block align-middle mx-0.5 overflow-hidden">
            <span></span>
            <span></span>
          </div>
        </span>
      </div>
    </div>

    <!-- 中间内容区域 -->
    <div class="flex flex-col items-center gap-6 flex-1 max-w-[600px] mt-4 sm:flex-col sm:gap-4">
      <!-- 汉字输入区域容器 -->
      <div class="flex justify-center items-center sm:flex-col sm:gap-4">
        <!-- 手写输入区域 -->
        <div
          class="char-display group w-[25rem] h-[25rem] bg-white shadow-[0_8px_24px_rgba(0,0,0,0.15)] mb-8 border-[0.3rem] border-dashed border-[rgba(52,73,94,0.5)] transition-all duration-300 ease-in-out [perspective:1000px] cursor-pointer relative overflow-hidden sm:w-[18rem] sm:h-[18rem] max-[480px]:w-64 max-[480px]:h-64"
          :class="{ flipped: quiz_is_right }" :title="t('dictation.viewAnswer')">
          <div
            class="char-inner w-full h-full relative [transform-style:preserve-3d] transition-transform duration-600 ease-in-out flex items-center justify-center">
            <span></span>
            <span></span>
            <div class="char-front absolute w-full h-full [backface-visibility:hidden] flex items-center justify-center">
              <div ref="writerContainer"
                class="w-full h-full flex items-center justify-center p-0 box-border"></div>
            </div>
            <div
              class="char-back absolute w-full h-full [backface-visibility:hidden] flex items-center justify-center [transform:rotateY(180deg)]">
              <div class="text-[12.5rem] text-[#2c3e50] font-bold font-['KaiTi_SC','KaiTi','STKaiti','SimKai',cursive,sans-serif] sm:text-[9rem] max-[480px]:text-8xl">
                {{ currentChar }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

  </div>

  <!-- 键盘遮罩组件 -->
  <KeyboardMask :show="showKeyboardMask" :char="currentChar" :click-x="clickX" :click-y="clickY"
    @close="closeKeyboardMask" />
</template>

<style scoped>
/* Card grid lines - pseudo-elements cannot be expressed as Tailwind utilities */

/* Horizontal dashed line through center */
.char-inner::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  width: 100%;
  height: 0.2rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.5) 0, rgba(52, 73, 94, 0.5) 0.2rem, transparent 0.2rem, transparent 0.5rem);
  transform: translateY(-50%);
  z-index: 2;
  pointer-events: none;
}

/* Vertical dashed line through center */
.char-inner::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 0;
  width: 0.2rem;
  height: 100%;
  background: repeating-linear-gradient(180deg, rgba(52, 73, 94, 0.5) 0, rgba(52, 73, 94, 0.5) 0.2rem, transparent 0.2rem, transparent 0.5rem);
  transform: translateX(-50%);
  z-index: 2;
  pointer-events: none;
}

/* 45-degree diagonal line */
.char-inner span:first-of-type {
  position: absolute;
  left: -50%;
  top: 50%;
  width: 200%;
  height: 0.1rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.25) 0, rgba(52, 73, 94, 0.25) 0.1rem, transparent 0.1rem, transparent 0.3rem);
  transform: translateY(-50%) rotate(45deg);
  z-index: 1;
  pointer-events: none;
}

/* -45-degree diagonal line */
.char-inner span:last-of-type {
  position: absolute;
  left: -50%;
  top: 50%;
  width: 200%;
  height: 0.1rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.25) 0, rgba(52, 73, 94, 0.25) 0.1rem, transparent 0.1rem, transparent 0.3rem);
  transform: translateY(-50%) rotate(-45deg);
  z-index: 1;
  pointer-events: none;
}

/* Flip card on answer reveal */
.char-display.flipped .char-inner {
  transform: rotateY(180deg);
}

/* Tianzige (practice grid) pseudo-element lines */
.tianzige::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  width: 100%;
  height: 0.2rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.5) 0, rgba(52, 73, 94, 0.5) 0.2rem, transparent 0.2rem, transparent 0.5rem);
  transform: translateY(-50%);
  z-index: 2;
}

.tianzige::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 0;
  width: 0.2rem;
  height: 100%;
  background: repeating-linear-gradient(180deg, rgba(52, 73, 94, 0.5) 0, rgba(52, 73, 94, 0.5) 0.2rem, transparent 0.2rem, transparent 0.5rem);
  transform: translateX(-50%);
  z-index: 2;
}

.tianzige span:first-of-type {
  position: absolute;
  left: -50%;
  top: 50%;
  width: 200%;
  height: 0.1rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.25) 0, rgba(52, 73, 94, 0.25) 0.1rem, transparent 0.1rem, transparent 0.3rem);
  transform: translateY(-50%) rotate(45deg);
  z-index: 1;
}

.tianzige span:last-of-type {
  position: absolute;
  left: -50%;
  top: 50%;
  width: 200%;
  height: 0.1rem;
  background: repeating-linear-gradient(90deg, rgba(52, 73, 94, 0.25) 0, rgba(52, 73, 94, 0.25) 0.1rem, transparent 0.1rem, transparent 0.3rem);
  transform: translateY(-50%) rotate(-45deg);
  z-index: 1;
}
</style>
