<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted, reactive } from 'vue';
import { KEY_PERMUTATION_ALPHABET, CAN_PRINT_KEY } from '../../../config/key';
import cnchar from 'cnchar-all';

// 导入键盘组件
import SingleKey from '../key/SingleKey.vue';
import KeyWrap from '../key/KeyWrap.vue';

// 定义组件props
const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  char: {
    type: String,
    default: ''
  },
  clickX: {
    type: Number,
    default: 0
  },
  clickY: {
    type: Number,
    default: 0
  }
});



// 键盘按键状态 - 使用本地状态而不是全局store
const keysPressed = ref<Record<string, boolean>>({});

// 字符到键码的映射
const charToKeyCode = ref<Record<string, string>>({});

// 存储打下的字母序列
const typedSequence = ref('');

// 定时器数组，用于管理所有定时器
const timers = ref<number[]>([]);

// 键盘位置
const keyboardPosition = reactive({
  top: 0,
  left: 0
});

// 监听点击位置和显示状态的变化，更新键盘位置
watch(() => [props.show, props.clickX, props.clickY], ([isShowing, x, y]) => {
  if (isShowing && x && y) {
    // 计算键盘位置，使其在点击位置附近显示
    // 键盘默认居中显示，如果点击位置在屏幕边缘，会自动调整
    keyboardPosition.top = (y as number) + 20; // 点击位置下方20px
    keyboardPosition.left = x as number;     // 点击位置的水平位置
  } else if (!isShowing) {
    // 当遮罩隐藏时，清除字符序列
    typedSequence.value = '';
  }
});

// 清除所有定时器
const clearAllTimers = () => {
  timers.value.forEach(timer => clearTimeout(timer));
  timers.value = [];
};

// 初始化字符到键码的映射
onMounted(() => {
  // 从CAN_PRINT_KEY反向映射
  for (const [keyCode, char] of Object.entries(CAN_PRINT_KEY)) {
    charToKeyCode.value[char.toLowerCase()] = keyCode;
  }

  // 添加更多拼音字符映射（包括声调字符）
  const additionalPinyinMapping = {
    'a': 'KeyA',
    'o': 'KeyO',
    'e': 'KeyE',
    'i': 'KeyI',
    'u': 'KeyU',
    'v': 'KeyV',
    'ü': 'KeyV', // 通常用v代替ü
    'b': 'KeyB',
    'p': 'KeyP',
    'm': 'KeyM',
    'f': 'KeyF',
    'd': 'KeyD',
    't': 'KeyT',
    'n': 'KeyN',
    'l': 'KeyL',
    'g': 'KeyG',
    'k': 'KeyK',
    'h': 'KeyH',
    'j': 'KeyJ',
    'q': 'KeyQ',
    'x': 'KeyX',
    'z': 'KeyZ',
    'c': 'KeyC',
    's': 'KeyS',
    'r': 'KeyR',
    'y': 'KeyY',
    'w': 'KeyW'
  };

  // 合并映射
  Object.assign(charToKeyCode.value, additionalPinyinMapping);
});

// 组件卸载时清除所有定时器
onUnmounted(() => {
  clearAllTimers();
});

// 当字符变化时，如果遮罩是显示状态，播放拼音动画
watch(() => [props.char, props.show], ([newChar, isShowing]) => {
  if (newChar && isShowing) {
    // 延迟执行按键动画
    setTimeout(() => {
      playPinyinAnimation(newChar as string);
    }, 500);
  }
});



// 播放拼音按键动画 - 循环播放
const playPinyinAnimation = (char: string) => {
  // 先清除所有现有的定时器
  clearAllTimers();

  // 清除字符序列
  typedSequence.value = '';

  // 获取拼音
  const pinyinResult = cnchar.spell(char);
  const pinyin = Array.isArray(pinyinResult) ? pinyinResult[0] : pinyinResult;
  console.log('拼音:', pinyin);

  // 分解拼音为字符序列
  let pinyinSequence = pinyin.toLowerCase().split('');
  console.log('拼音序列:', pinyinSequence);

  // 过滤掉声调符号和其他不需要的字符
  const validPinyinChars = 'abcdefghijklmnopqrstuvwxyzü';
  pinyinSequence = pinyinSequence.filter((c: string) => validPinyinChars.includes(c));
  console.log('过滤后的拼音序列:', pinyinSequence);

  // 如果没有有效的拼音字符，直接返回
  if (pinyinSequence.length === 0) return;

  // 循环播放按键
  const playNextKey = (index: number) => {
    const char = pinyinSequence[index % pinyinSequence.length];

    pressAndReleaseKey(char);

    // 设置下一个按键的定时器
    // 当轮播结束后，先清空字符序列，然后再开始新一轮循环
    if ((index + 1) % pinyinSequence.length === 0) {
      // 轮播结束时，先清空字符序列
      const clearTimer = window.setTimeout(() => {
        typedSequence.value = '';
        // 然后再开始新一轮循环
        const nextRoundTimer = window.setTimeout(() => {
          playNextKey(index + 1);
        }, 500); // 新一轮开始前的延迟
        timers.value.push(nextRoundTimer);
      }, 500); // 轮播结束后显示空状态的时间
      timers.value.push(clearTimer);
    } else {
      // 普通按键延迟
      const timer = window.setTimeout(() => {
        playNextKey(index + 1);
      }, 500);
      timers.value.push(timer);
    }
  };

  // 开始播放
  playNextKey(0);
};

// 按下并释放键
const pressAndReleaseKey = (char: string) => {
  const keyCode = charToKeyCode.value[char];
  if (!keyCode) return;

  // 按下键 - 只使用本地状态
  keysPressed.value[keyCode] = true;

  // 将字符添加到序列中
  typedSequence.value += char;

  // 100ms后释放键
  const timer = window.setTimeout(() => {
    keysPressed.value[keyCode] = false;
  }, 100);

  // 保存定时器到数组
  timers.value.push(timer);
};
</script>

<template>
  <div class="keyboard-mask static w-full bg-transparent z-[1]">
    <key-wrap title="" className="y-key-wrap__standard y-key-wrap__alphabet keyboard-wrap-mask relative w-full m-0 p-[0.6rem_0.9rem] bg-[#f0f2eb] rounded-[0.6rem] shadow-[0.2rem_0.2rem_0.4rem_rgba(0,0,0,0.1)] flex flex-col md:p-[0.8rem_1.2rem]! md:items-center md:rounded-[0.8rem]! md:max-w-[90vw]! md:left-1/2! md:-translate-x-1/2">
      <!-- 显示打下的字符序列 -->
      <div class="typed-sequence-display text-left text-[3.6rem] mb-[0.8rem] pt-[0.4rem] pb-[0.4rem] pl-[1.4rem] pr-0 text-[#333] min-h-[4.8rem] w-full relative z-[1001] font-bold flex items-center justify-start">{{ typedSequence }}</div>

      <template v-for="(value, key) in KEY_PERMUTATION_ALPHABET" :key="key">
        <div class="y-keyboard__wrap y-keyboard__wrap--alphabet w-full flex flex-col items-center" :class="['y-keyboard__' + key]">
          <div class="y-keyboard__line y-keyboard__line--alphabet flex justify-start mb-[0.4rem] flex-nowrap w-full items-center" v-for="(v, index) in value" :key="index + 'line'">
            <single-key v-for="item in v" :key="item.code" :code="item.code" :value="item.value" :unit="item.unit"
              :keys-pressed="keysPressed"></single-key>
          </div>
        </div>
      </template>
    </key-wrap>
  </div>
</template>

<style scoped>
.keyboard-wrap-mask .y-keyboard__line--alphabet .y-single-key {
  margin: 0.2rem 0.15rem;
}

.keyboard-wrap-mask .y-keyboard__line--alphabet:nth-child(2) {
  margin-left: 1.8rem;
}

.keyboard-wrap-mask .y-keyboard__line--alphabet:nth-child(3) {
  margin-left: 4.1rem;
}
</style>