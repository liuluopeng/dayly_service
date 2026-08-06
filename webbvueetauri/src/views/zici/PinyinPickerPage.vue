<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import cnchar from 'cnchar-all';

const { t } = useI18n();
const router = useRouter();

// 声母列表
const initialSounds = ref([]);
const allFinalSounds = ref([]);
const pinyinDictionary = ref({});

async function loadPinyinData() {
  try {
    const { get_base_url_wasm } = await import('../../types/wasm-typed');
    const res = await fetch(`${get_base_url_wasm()}/api/zici/pinyin`);
    const json = await res.json();
    const data = json.data || {};
    initialSounds.value = data.initials || [];
    allFinalSounds.value = data.finals || [];
    pinyinDictionary.value = data.combos || {};
  } catch (e) {
    console.error('拼音数据加载失败:', e);
  }
}
loadPinyinData();

// 选择状态
const selectedInitialIndex = ref(0);
const selectedFinalIndex = ref(0);
const initialScrollRef = ref(null);
const finalScrollRef = ref(null);

const ITEM_HEIGHT = 60;

// 根据当前声母获取合法的韵母列表
const filteredFinalSounds = computed(() => {
  const currentInitial = initialSounds.value[selectedInitialIndex.value].pinyin;
  const finalsMap = pinyinDictionary.value[currentInitial];
  if (!finalsMap) return [];

  return Object.keys(finalsMap).map(final => {
    const finalItem = allFinalSounds.value.find(f => f.pinyin === final);
    return finalItem || { char: final, pinyin: final, example: '' };
  });
});

// 组合拼音
const combinedPinyin = computed(() => {
  const initial = initialSounds[selectedInitialIndex.value].pinyin;
  const validFinals = filteredFinalSounds.value;
  if (validFinals.length === 0) return initial;

  const final = validFinals[Math.min(selectedFinalIndex.value, validFinals.length - 1)];
  const finalsMap = pinyinDictionary.value[initial];

  if (finalsMap && finalsMap[final.pinyin]) {
    return finalsMap[final.pinyin];
  }

  return initial;
});

const selectedInitial = computed(() => initialSounds[selectedInitialIndex.value].pinyin);
const selectedFinal = computed(() => {
  const validFinals = filteredFinalSounds.value;
  if (validFinals.length === 0) return '';
  return validFinals[Math.min(selectedFinalIndex.value, validFinals.length - 1)].pinyin;
});

const allFinalsForGrid = ['a', 'o', 'e', 'i', 'u', 'ü', 'ai', 'ei', 'ui', 'ao', 'ou', 'iu', 'ie', 'üe', 'er', 'an', 'en', 'in', 'un', 'ün', 'ang', 'eng', 'ing', 'ong'];

const gridTable = computed(() => {
  const rows = [];
  for (const initial of initialSounds.value) {
    const row = [];
    for (const final of allFinalsForGrid) {
      const finalsMap = pinyinDictionary.value[initial.pinyin];
      let pinyin = null;
      if (finalsMap && finalsMap[final]) {
        pinyin = finalsMap[final];
      }
      row.push({
        initial: initial.pinyin,
        final: final,
        pinyin: pinyin,
        exists: pinyin !== null
      });
    }
    rows.push({ initial: initial, cells: row });
  }
  return rows;
});

const handleGridCellClick = (cell) => {
  if (!cell.exists) return;

  // 单元格点击同时更新声母/韵母：临时禁用 watcher 重置，避免高亮跳到第一个韵母
  skipFinalReset = true;

  const initialIndex = initialSounds.value.findIndex(s => s.pinyin === cell.initial);
  if (initialIndex >= 0) {
    selectedInitialIndex.value = initialIndex;
  }

  const finalsForInitial = Object.keys(pinyinDictionary.value[cell.initial] || {});
  const finalIndex = finalsForInitial.indexOf(cell.final);
  if (finalIndex >= 0) {
    selectedFinalIndex.value = finalIndex;
  }

  skipFinalReset = false;
  navigateToWords();
};

// 监听声母变化，重置韵母选择（单元格点击场景除外）
let skipFinalReset = false;
watch(selectedInitialIndex, () => {
  if (skipFinalReset) return;
  selectedFinalIndex.value = 0;
});

// 页面加载时初始化
onMounted(() => {
  // 初始化完成
});

// 更新选中索引
const updateIndex = (type, index) => {
  if (type === 'initial') {
    selectedInitialIndex.value = index;
  } else {
    selectedFinalIndex.value = index;
  }
};

const navigateToWords = () => {
  router.push({ name: 'ZiciPinyinWords', query: { pinyin: combinedPinyin.value } });
};

// 获取拼音的所有音调形式（基于cnchar源码实现）
const getAllTonePinyin = (pinyin) => {
  // 定义声调标记（与cnchar源码一致）
  const tones = {
    a: ['a', 'ā', 'á', 'ǎ', 'à'],
    e: ['e', 'ē', 'é', 'ě', 'è'],
    i: ['i', 'ī', 'í', 'ǐ', 'ì'],
    o: ['o', 'ō', 'ó', 'ǒ', 'ò'],
    u: ['u', 'ū', 'ú', 'ǔ', 'ù'],
    ü: ['ü', 'ǖ', 'ǘ', 'ǚ', 'ǜ']
  };

  // 处理特殊情况
  if (!pinyin || typeof pinyin !== 'string') {
    return [];
  }

  // 标准化拼音（转为小写）
  pinyin = pinyin.toLowerCase();

  // 检查拼音中包含的元音（优先级：a > e > i > o > u > ü）
  const vowelOrder = ['a', 'e', 'i', 'o', 'u', 'ü'];
  let targetVowel = '';
  let vowelIndex = -1;

  // 找到拼音中的第一个元音（按优先级顺序）
  for (const vowel of vowelOrder) {
    const index = pinyin.indexOf(vowel);
    if (index !== -1) {
      targetVowel = vowel;
      vowelIndex = index;
      break;
    }
  }

  // 如果没有元音，返回原拼音
  if (!targetVowel) {
    return [pinyin];
  }

  // 生成所有音调的拼音
  const result = [];
  const vowelTones = tones[targetVowel];

  // 生成轻声到四声的所有形式
  for (let i = 0; i < 5; i++) {
    const toneChar = vowelTones[i];
    const newPinyin = pinyin.substring(0, vowelIndex) + toneChar + pinyin.substring(vowelIndex + 1);
    result.push(newPinyin);
  }

  return result;
};
</script>

<template>
  <div class="box-border w-full min-h-full p-[0.8rem] max-w-full flex flex-col overflow-hidden bg-[#f5f5f5]">
    <div class="flex gap-4 min-h-0 overflow-hidden flex-[0.4]">
      <div class="flex flex-col bg-[#f8f9fa] rounded-2xl shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1),inset_0_0_0_1px_rgba(255,255,255,0.8)] overflow-hidden relative flex-1">
        <div class="p-[0.8rem] text-center text-base font-bold text-white bg-gradient-to-br from-[#4caf50] to-[#45a049] shrink-0 select-none">{{ t('pinyinPicker.initials') }}</div>
        <div class="flex-1 overflow-y-auto p-2 grid grid-cols-[repeat(auto-fill,minmax(60px,1fr))] gap-[0.3rem]">
          <div v-for="(item, index) in initialSounds" :key="item.pinyin"
            class="flex flex-col items-center justify-center p-2 cursor-pointer transition-all duration-200 rounded-lg bg-white shadow-[0_1px_3px_rgba(0,0,0,0.1)] select-none hover:bg-[#e8f5e9] hover:scale-[1.05]"
            :class="{ '!bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] !text-white !shadow-[0_0.3rem_0.6rem_rgba(76,175,80,0.4)] !scale-[1.05]': index === selectedInitialIndex }"
            @click="updateIndex('initial', index)">
            <div class="text-[0.9rem] font-bold text-[#333]" :class="{ '!text-white': index === selectedInitialIndex }">{{ item.pinyin }}</div>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-center text-[2rem] font-bold text-[#999] w-[50px] shrink-0 self-center">+</div>

      <div class="flex flex-col bg-[#f8f9fa] rounded-2xl shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1),inset_0_0_0_1px_rgba(255,255,255,0.8)] overflow-hidden relative flex-1">
        <div class="p-[0.8rem] text-center text-base font-bold text-white bg-gradient-to-br from-[#4caf50] to-[#45a049] shrink-0 select-none">{{ t('pinyinPicker.finals') }}</div>
        <div class="flex-1 overflow-y-auto p-2 grid grid-cols-[repeat(auto-fill,minmax(60px,1fr))] gap-[0.3rem]">
          <div v-for="(item, index) in filteredFinalSounds" :key="item.pinyin"
            class="flex flex-col items-center justify-center p-2 cursor-pointer transition-all duration-200 rounded-lg bg-white shadow-[0_1px_3px_rgba(0,0,0,0.1)] select-none hover:bg-[#e8f5e9] hover:scale-[1.05]"
            :class="{ '!bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] !text-white !shadow-[0_0.3rem_0.6rem_rgba(76,175,80,0.4)] !scale-[1.05]': index === selectedFinalIndex }"
            @click="updateIndex('final', index)">
            <div class="text-[0.9rem] font-bold text-[#333]" :class="{ '!text-white': index === selectedFinalIndex }">{{ item.pinyin }}</div>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-center text-[2rem] font-bold text-[#999] w-[50px] shrink-0 self-center">=</div>

      <div class="flex flex-col items-center justify-center bg-gradient-to-br from-[#ff6b6b] to-[#ee5a5a] rounded-full p-4 cursor-pointer transition-all duration-300 shadow-[0_0.3rem_0.6rem_rgba(255,107,107,0.3)] w-[100px] h-[100px] shrink-0 self-center hover:scale-[1.05] hover:shadow-[0_0.5rem_1rem_rgba(255,107,107,0.4)]" @click="navigateToWords">
        <div class="text-[1.5rem] font-bold text-white">{{ combinedPinyin }}</div>
        <div class="text-[0.7rem] text-white/80 mt-[0.3rem]">{{ t('pinyinPicker.clickToJump') }}</div>
      </div>
    </div>

    <div class="flex gap-4 min-h-0 overflow-hidden flex-[0.6]">
      <div class="flex flex-col bg-[#f8f9fa] rounded-2xl shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1),inset_0_0_0_1px_rgba(255,255,255,0.8)] overflow-hidden relative flex-1 w-full">
        <div class="p-[0.8rem] text-center text-base font-bold text-white bg-gradient-to-br from-[#4caf50] to-[#45a049] shrink-0 select-none">{{ t('pinyinPicker.pinyinTable') }} ({{ combinedPinyin }})</div>
        <div class="flex-1 overflow-auto p-2">
          <table class="border-collapse w-full text-[0.75rem]">
            <thead>
              <tr>
                <th class="bg-[#f0f0f0] min-w-[50px] border border-[#ddd] p-[0.3rem] text-center whitespace-nowrap"></th>
                <th v-for="final in allFinalsForGrid" :key="final" class="bg-[#4caf50] text-white font-bold sticky top-0 z-10 border border-[#ddd] p-[0.3rem] text-center whitespace-nowrap">
                  {{ final }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in gridTable" :key="row.initial.pinyin">
                <td class="bg-[#f0f0f0] font-bold sticky left-0 z-5 flex flex-col items-center gap-[2px] border border-[#ddd] p-[0.3rem] text-center whitespace-nowrap">
                  <span class="text-[0.8rem]">{{ row.initial.pinyin }}</span>
                  <span class="text-[0.6rem] text-[#666]">{{ row.initial.example }}</span>
                </td>
                <td v-for="cell in row.cells" :key="cell.final"
                  class="bg-[#fafafa] cursor-default min-w-[36px] h-[32px] border border-[#ddd] p-[0.3rem] text-center whitespace-nowrap relative"
                  :class="{
                    'bg-white cursor-pointer text-[#333] font-medium hover:bg-[#e8f5e9] hover:scale-[1.1] hover:shadow-[0_2px_4px_rgba(0,0,0,0.2)] hover:z-20': cell.exists,
                    '!bg-[#4caf50] !text-white !font-bold !shadow-[0_0_0_2px_#fff,0_0_0_4px_#4caf50] z-[15]': cell.initial === selectedInitial && cell.final === selectedFinal
                  }"
                  @click="handleGridCellClick(cell)">
                  <span v-if="cell.exists">{{ cell.pinyin }}</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
