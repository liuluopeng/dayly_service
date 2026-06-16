<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import cnchar from 'cnchar-all';

const { t } = useI18n();
const router = useRouter();

// 声母列表
const initialSounds = [
  { char: 'b', pinyin: 'b', example: '波' },
  { char: 'p', pinyin: 'p', example: '坡' },
  { char: 'm', pinyin: 'm', example: '摸' },
  { char: 'f', pinyin: 'f', example: '佛' },
  { char: 'd', pinyin: 'd', example: '的' },
  { char: 't', pinyin: 't', example: '特' },
  { char: 'n', pinyin: 'n', example: '呢' },
  { char: 'l', pinyin: 'l', example: '乐' },
  { char: 'g', pinyin: 'g', example: '哥' },
  { char: 'k', pinyin: 'k', example: '科' },
  { char: 'h', pinyin: 'h', example: '喝' },
  { char: 'j', pinyin: 'j', example: '鸡' },
  { char: 'q', pinyin: 'q', example: '七' },
  { char: 'x', pinyin: 'x', example: '希' },
  { char: 'zh', pinyin: 'zh', example: '知' },
  { char: 'ch', pinyin: 'ch', example: '吃' },
  { char: 'sh', pinyin: 'sh', example: '狮' },
  { char: 'r', pinyin: 'r', example: '日' },
  { char: 'z', pinyin: 'z', example: '资' },
  { char: 'c', pinyin: 'c', example: '疵' },
  { char: 's', pinyin: 's', example: '撕' },
  { char: 'y', pinyin: 'y', example: '医' },
  { char: 'w', pinyin: 'w', example: '乌' },
  { char: '*', pinyin: '*', example: '空' }
];

// 完整的韵母列表
const allFinalSounds = [
  { char: 'a', pinyin: 'a', example: '啊' },
  { char: 'o', pinyin: 'o', example: '窝' },
  { char: 'e', pinyin: 'e', example: '鹅' },
  { char: 'i', pinyin: 'i', example: '衣' },
  { char: 'u', pinyin: 'u', example: '乌' },
  { char: 'ü', pinyin: 'ü', example: '迂' },
  { char: 'ai', pinyin: 'ai', example: '爱' },
  { char: 'ei', pinyin: 'ei', example: '飞' },
  { char: 'ui', pinyin: 'ui', example: '水' },
  { char: 'ao', pinyin: 'ao', example: '奥' },
  { char: 'ou', pinyin: 'ou', example: '欧' },
  { char: 'iu', pinyin: 'iu', example: '牛' },
  { char: 'ie', pinyin: 'ie', example: '姐' },
  { char: 'ue', pinyin: 'ue', example: '月' },
  { char: 'er', pinyin: 'er', example: '儿' },
  { char: 'an', pinyin: 'an', example: '安' },
  { char: 'en', pinyin: 'en', example: '恩' },
  { char: 'in', pinyin: 'in', example: '音' },
  { char: 'un', pinyin: 'un', example: '云' },
  { char: 'ang', pinyin: 'ang', example: '昂' },
  { char: 'eng', pinyin: 'eng', example: '风' },
  { char: 'ing', pinyin: 'ing', example: '英' },
  { char: 'ong', pinyin: 'ong', example: '东' }
];

// 拼音字典：声母 -> 韵母 -> 组合拼音
const pinyinDictionary = {
  'b': {
    'a': 'ba',
    'o': 'bo',
    'ai': 'bai',
    'ei': 'bei',
    'ao': 'bao',
    'an': 'ban',
    'en': 'ben',
    'ang': 'bang',
    'eng': 'beng',
    'i': 'bi',
    'ie': 'bie',
    'iao': 'biao',
    'ian': 'bian',
    'in': 'bin',
    'ing': 'bing',
    'u': 'bu'
  },
  'p': {
    'a': 'pa',
    'o': 'po',
    'ai': 'pai',
    'ei': 'pei',
    'ao': 'pao',
    'ou': 'pou',
    'an': 'pan',
    'en': 'pen',
    'ang': 'pang',
    'eng': 'peng',
    'i': 'pi',
    'ie': 'pie',
    'iao': 'piao',
    'ian': 'pian',
    'in': 'pin',
    'ing': 'ping',
    'u': 'pu'
  },
  'm': {
    'a': 'ma',
    'o': 'mo',
    'ai': 'mai',
    'ei': 'mei',
    'ao': 'mao',
    'ou': 'mou',
    'an': 'man',
    'en': 'men',
    'ang': 'mang',
    'eng': 'meng',
    'i': 'mi',
    'ie': 'mie',
    'iao': 'miao',
    'iou': 'miu',
    'ian': 'mian',
    'in': 'min',
    'ing': 'ming',
    'u': 'mu'
  },
  'f': {
    'a': 'fa',
    'o': 'fo',
    'e': 'me',
    'ei': 'fei',
    'ou': 'fou',
    'an': 'fan',
    'en': 'fen',
    'ang': 'fang',
    'eng': 'feng',
    'u': 'fu'
  },
  'd': {
    'a': 'da',
    'e': 'de',
    'ai': 'dai',
    'ei': 'dei',
    'ao': 'dao',
    'ou': 'dou',
    'an': 'dan',
    'en': 'den',
    'ang': 'dang',
    'eng': 'deng',
    'ong': 'dong',
    'i': 'di',
    'ia': 'dia',
    'ie': 'die',
    'iao': 'diao',
    'iou': 'diu',
    'ian': 'dian',
    'ing': 'ding',
    'u': 'du',
    'uo': 'duo',
    'uei': 'dui',
    'uan': 'duan',
    'un': 'dun'
  },
  't': {
    'a': 'ta',
    'e': 'te',
    'ai': 'tai',
    'ei': 'tei',
    'ao': 'tao',
    'ou': 'tou',
    'an': 'tan',
    'ang': 'tang',
    'eng': 'teng',
    'ong': 'tong',
    'i': 'ti',
    'ie': 'tie',
    'iao': 'tiao',
    'ian': 'tian',
    'ing': 'ting',
    'u': 'tu',
    'uo': 'tuo',
    'uei': 'tui',
    'uan': 'tuan',
    'un': 'tun'
  },
  'n': {
    'a': 'na',
    'e': 'ne',
    'ai': 'nai',
    'ei': 'nei',
    'ao': 'nao',
    'ou': 'nou',
    'an': 'nan',
    'en': 'nen',
    'ang': 'nang',
    'eng': 'neng',
    'ong': 'nong',
    'i': 'ni',
    'ie': 'nie',
    'iao': 'niao',
    'iou': 'niu',
    'ian': 'nian',
    'in': 'nin',
    'iang': 'niang',
    'ing': 'ning',
    'u': 'nu',
    'uo': 'nuo',
    'uan': 'nuan',
    'ü': 'nü',
    'üe': 'nüe'
  },
  'l': {
    'a': 'la',
    'e': 'le',
    'ai': 'lai',
    'ei': 'lei',
    'ao': 'lao',
    'ou': 'lou',
    'an': 'lan',
    'ang': 'lang',
    'eng': 'leng',
    'ong': 'long',
    'i': 'li',
    'ia': 'lia',
    'ie': 'lie',
    'iao': 'liao',
    'iou': 'liu',
    'ian': 'lian',
    'in': 'lin',
    'iang': 'liang',
    'ing': 'ling',
    'u': 'lu',
    'uo': 'luo',
    'uan': 'luan',
    'un': 'lun',
    'ü': 'lü',
    'üe': 'lüe'
  },
  'g': {
    'a': 'ga',
    'e': 'ge',
    'ai': 'gai',
    'ei': 'gei',
    'ao': 'gao',
    'ou': 'gou',
    'an': 'gan',
    'en': 'gen',
    'ang': 'gang',
    'eng': 'geng',
    'ong': 'gong',
    'u': 'gu',
    'ua': 'gua',
    'uo': 'guo',
    'uai': 'guai',
    'uei': 'gui',
    'uan': 'guan',
    'un': 'gun',
    'uang': 'guang'
  },
  'k': {
    'a': 'ka',
    'e': 'ke',
    'ai': 'kai',
    'ao': 'kao',
    'ou': 'kou',
    'an': 'kan',
    'en': 'ken',
    'ang': 'kang',
    'eng': 'keng',
    'ong': 'kong',
    'u': 'ku',
    'ua': 'kua',
    'uo': 'kuo',
    'uai': 'kuai',
    'uei': 'kui',
    'uan': 'kuan',
    'un': 'kun',
    'uang': 'kuang'
  },
  'h': {
    'a': 'ha',
    'e': 'he',
    'ai': 'hai',
    'ei': 'hei',
    'ao': 'hao',
    'ou': 'hou',
    'an': 'han',
    'en': 'hen',
    'ang': 'hang',
    'eng': 'heng',
    'ong': 'hong',
    'u': 'hu',
    'ua': 'hua',
    'uo': 'huo',
    'uai': 'huai',
    'uei': 'hui',
    'uan': 'huan',
    'un': 'hun',
    'uang': 'huang'
  },
  'j': {
    'i': 'ji',
    'ia': 'jia',
    'ie': 'jie',
    'iao': 'jiao',
    'iou': 'jiu',
    'ian': 'jian',
    'in': 'jin',
    'iang': 'jiang',
    'ing': 'jing',
    'iong': 'jiong',
    'ü': 'ju',
    'üe': 'jue',
    'üan': 'juan',
    'ün': 'jun'
  },
  'q': {
    'i': 'qi',
    'ia': 'qia',
    'ie': 'qie',
    'iao': 'qiao',
    'iou': 'qiu',
    'ian': 'qian',
    'in': 'qin',
    'iang': 'qiang',
    'ing': 'qing',
    'iong': 'qiong',
    'ü': 'qu',
    'üe': 'que',
    'üan': 'quan',
    'ün': 'qun'
  },
  'x': {
    'i': 'xi',
    'ia': 'xia',
    'ie': 'xie',
    'iao': 'xiao',
    'iou': 'xiu',
    'ian': 'xian',
    'in': 'xin',
    'iang': 'xiang',
    'ing': 'xing',
    'iong': 'xiong',
    'ü': 'xu',
    'üe': 'xue',
    'üan': 'xuan',
    'ün': 'xun'
  },
  'zh': {
    'a': 'zha',
    'e': 'zhe',
    'ai': 'zhai',
    'ei': 'zhei',
    'ao': 'zhao',
    'ou': 'zhou',
    'an': 'zhan',
    'en': 'zhen',
    'ang': 'zhang',
    'eng': 'zheng',
    'ong': 'zhong',
    'u': 'zhu',
    'ua': 'zhua',
    'uo': 'zhuo',
    'uai': 'zhuai',
    'uei': 'zhui',
    'uan': 'zhuan',
    'un': 'zhun',
    'uang': 'zhuang'
  },
  'ch': {
    'a': 'cha',
    'e': 'che',
    'ai': 'chai',
    'ao': 'chao',
    'ou': 'chou',
    'an': 'chan',
    'en': 'chen',
    'ang': 'chang',
    'eng': 'cheng',
    'ong': 'chong',
    'u': 'chu',
    'uo': 'chuo',
    'uai': 'chuai',
    'uei': 'chui',
    'uan': 'chuan',
    'un': 'chun',
    'uang': 'chuang'
  },
  'sh': {
    'a': 'sha',
    'e': 'she',
    'ai': 'shai',
    'ei': 'shei',
    'ao': 'shao',
    'ou': 'shou',
    'an': 'shan',
    'en': 'shen',
    'ang': 'shang',
    'eng': 'sheng',
    'u': 'shu',
    'ua': 'shua',
    'uo': 'shuo',
    'uai': 'shuai',
    'uei': 'shui',
    'uan': 'shuan',
    'un': 'shun',
    'uang': 'shuang'
  },
  'r': {
    'e': 're',
    'ao': 'rao',
    'ou': 'rou',
    'an': 'ran',
    'en': 'ren',
    'ang': 'rang',
    'eng': 'reng',
    'ong': 'rong',
    'u': 'ru',
    'uo': 'ruo',
    'uei': 'rui',
    'uan': 'ruan',
    'un': 'run'
  },
  'z': {
    'a': 'za',
    'e': 'ze',
    'ai': 'zai',
    'ei': 'zei',
    'ao': 'zao',
    'ou': 'zou',
    'an': 'zan',
    'en': 'zen',
    'ang': 'zang',
    'eng': 'zeng',
    'ong': 'zong',
    'u': 'zu',
    'uo': 'zuo',
    'uei': 'zui',
    'uan': 'zuan',
    'un': 'zun'
  },
  'c': {
    'a': 'ca',
    'e': 'ce',
    'ai': 'cai',
    'ao': 'cao',
    'ou': 'cou',
    'an': 'can',
    'en': 'cen',
    'ang': 'cang',
    'eng': 'ceng',
    'ong': 'cong',
    'u': 'cu',
    'uo': 'cuo',
    'uei': 'cui',
    'uan': 'cuan',
    'un': 'cun'
  },
  's': {
    'a': 'sa',
    'e': 'se',
    'ai': 'sai',
    'ao': 'sao',
    'ou': 'sou',
    'an': 'san',
    'en': 'sen',
    'ang': 'sang',
    'eng': 'seng',
    'ong': 'song',
    'u': 'su',
    'uo': 'suo',
    'uei': 'sui',
    'uan': 'suan',
    'un': 'sun'
  },

  'y': {
    'a': 'ya',
    'i': 'yi',
    'e': 'ye',
    'ao': 'yao',
    'ou': 'you',
    'an': 'yan',
    'in': 'yin',
    'ang': 'yang',
    'ing': 'ying',
    'ong': 'yong',
    'u': 'yu',
    'uan': 'yuan',
    'ue': 'yue',
    'un': 'yun',
  },

  'w': {
    'u': 'wu',
    'a': 'wa',
    'o': 'wo',
    'ai': 'wai',
    'ei': 'wei',
    'an': 'wan',
    'en': 'wen',
    'ang': 'wang',
    'eng': 'weng',

  },

  '*': {
    'a': 'a',
    'ao': 'ao',
    'ai': 'ai',
    'an': 'an',
    'ang': 'ang',
    'e': 'e',
    'er': 'er',
    'en': 'en',
    'eng': 'eng',
    'o': 'o',
    'ou': 'ou',
  },


};

// 选择状态
const selectedInitialIndex = ref(0);
const selectedFinalIndex = ref(0);
const initialScrollRef = ref(null);
const finalScrollRef = ref(null);

const ITEM_HEIGHT = 60;

// 根据当前声母获取合法的韵母列表
const filteredFinalSounds = computed(() => {
  const currentInitial = initialSounds[selectedInitialIndex.value].pinyin;
  const finalsMap = pinyinDictionary[currentInitial];
  if (!finalsMap) return [];

  return Object.keys(finalsMap).map(final => {
    const finalItem = allFinalSounds.find(f => f.pinyin === final);
    return finalItem || { char: final, pinyin: final, example: '' };
  });
});

// 组合拼音
const combinedPinyin = computed(() => {
  const initial = initialSounds[selectedInitialIndex.value].pinyin;
  const validFinals = filteredFinalSounds.value;
  if (validFinals.length === 0) return initial;

  const final = validFinals[Math.min(selectedFinalIndex.value, validFinals.length - 1)];
  const finalsMap = pinyinDictionary[initial];

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
  for (const initial of initialSounds) {
    const row = [];
    for (const final of allFinalsForGrid) {
      const finalsMap = pinyinDictionary[initial.pinyin];
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

  const initialIndex = initialSounds.findIndex(s => s.pinyin === cell.initial);
  if (initialIndex >= 0) {
    selectedInitialIndex.value = initialIndex;
  }

  const finalsForInitial = Object.keys(pinyinDictionary[cell.initial] || {});
  const finalIndex = finalsForInitial.indexOf(cell.final);
  if (finalIndex >= 0) {
    selectedFinalIndex.value = finalIndex;
  }

  navigateToWords();
};

// 监听声母变化，重置韵母选择
watch(selectedInitialIndex, () => {
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
