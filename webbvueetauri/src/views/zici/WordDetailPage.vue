<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import cnchar from 'cnchar-all';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const allInitials = ['zh', 'ch', 'sh', 'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'r', 'z', 'c', 's', 'y', 'w'];

const wordParam = computed(() => route.query.word || '');
const pinyinParam = computed(() => route.query.pinyin || '');

const wordData = ref(null);
const isLoading = ref(true);

const goBack = () => {
  router.back();
};

const findWordData = async () => {
  isLoading.value = true;
  try {
    const pinyin = pinyinParam.value;
    const word = wordParam.value;

    if (pinyin) {
      const apiUrl = import.meta.env.VITE_API_BASE_URL || '';
      const token = localStorage.getItem('token') || '';
      const response = await fetch(`${apiUrl}/api/pinyin/dict?ori=${encodeURIComponent(pinyin)}`, {
        headers: token ? { Authorization: `Bearer ${token}` } : {}
      });
      if (!response.ok) throw new Error(`加载数据失败: ${response.status}`);
      const wordsData = await response.json();
      const found = Array.isArray(wordsData) ? wordsData.find(w => w.word === word) : undefined;
      if (found) {
        wordData.value = {
          word: found.word,
          pinyin: found.pinyin_array ? found.pinyin_array.join(' ') : '',
          pinyin_flat: found.pinyin_flat || '',
          explanation: found.explanation || '',
          frequency: found.frequency || 0,
          word_length: found.word_length || found.word.length,
          first_char: found.first_char || ''
        };
      }
    }

    if (!wordData.value) {
      for (const key in pinyinApiResults.value) {
        const found = pinyinApiResults.value[key].find(w => w.word === word);
        if (found) {
          wordData.value = {
            word: found.word,
            pinyin: found.pinyin_array ? found.pinyin_array.join(' ') : '',
            pinyin_flat: found.pinyin_flat || '',
            explanation: found.explanation || '',
            frequency: found.frequency || 0,
            word_length: found.word_length || found.word.length,
            first_char: found.first_char || ''
          };
          break;
        }
      }
    }
  } catch (error) {
    console.error('查找词语数据失败:', error);
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  findWordData();
});

const speakWord = () => {
  stopSpeak();
  try {
    if (wordData.value && wordData.value.word) {
      cnchar.voice.speak(wordData.value.word);
    }
  } catch (error) {
    console.error('词语发音失败:', error);
  }
};

const stopSpeak = () => {
  try {
    if (cnchar.voice) {
      if (typeof cnchar.voice.stop === 'function') cnchar.voice.stop();
      if (typeof cnchar.voice.pause === 'function') cnchar.voice.pause();
      if (typeof cnchar.voice.cancel === 'function') cnchar.voice.cancel();
    }
  } catch (error) {
    console.error('停止语音失败:', error);
  }
};

const extractExplanationText = computed(() => {
  if (!wordData.value || !wordData.value.explanation) return '';

  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(wordData.value.explanation, 'text/html');
    const entry = doc.querySelector('entry');
    return entry ? entry.innerHTML : '';
  } catch (e) {
    return wordData.value.explanation;
  }
});

const getExplanationTextForSpeak = computed(() => {
  if (!wordData.value || !wordData.value.explanation) return '';

  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(wordData.value.explanation, 'text/html');

    let hwContent = '';
    const hwElement = doc.querySelector('hw');
    if (hwElement) hwContent = hwElement.textContent.trim();

    const defElements = doc.querySelectorAll('def');
    let definitions = [];

    defElements.forEach(defElement => {
      const defClone = defElement.cloneNode(true);
      const posTags = defClone.querySelectorAll('ps, pt, pu, pv');
      posTags.forEach(tag => tag.remove());
      const numTags = defClone.querySelectorAll('num');
      numTags.forEach(tag => tag.remove());
      let defText = defClone.textContent.trim();
      defText = defText.replace(/[①②③④⑤⑥⑦⑧⑨⑩]/g, '');
      defText = defText.replace(/[1-9][0-9]*[、.．]/g, '');
      defText = defText.replace(/^[、.．\s]+/, '');
      defText = defText.replace(/\s+/g, ' ').trim();
      if (defText) definitions.push(defText);
    });

    let resultText = '';
    if (hwContent) resultText += hwContent + '，';
    if (definitions.length > 0) {
      definitions = definitions.map(def => def.replace(/～/g, wordData.value.word));
      resultText += definitions.join(' ');
    }
    return resultText || '';
  } catch (error) {
    console.error('解析解释文本失败:', error);
    return '';
  }
});

const speakExplanation = () => {
  stopSpeak();
  try {
    const text = getExplanationTextForSpeak.value;
    if (text) {
      cnchar.voice.speak(text);
    }
  } catch (error) {
    console.error('解释发音失败:', error);
  }
};

const frequencyScore = computed(() => {
  if (!wordData.value) return 0;
  return (1 - wordData.value.frequency / 56000).toFixed(6);
});

const splitPinyin = (pinyin) => {
  for (const initial of allInitials) {
    if (pinyin.startsWith(initial)) {
      const rest = pinyin.substring(initial.length);
      return { initial, final: rest };
    }
  }
  return { initial: '', final: pinyin };
};

const coloredPinyinParts = computed(() => {
  if (!wordData.value || !wordData.value.pinyin) return [];
  const pinyins = wordData.value.pinyin.split(' ');
  return pinyins.map(p => splitPinyin(p));
});
</script>

<template>
  <div class="w-full min-h-full bg-[#f5f5f5] overflow-hidden flex flex-col box-border">
    <div class="flex-1 flex flex-col bg-[#f8f9fa] rounded-2xl m-[0.8rem] shadow-[0_0.3rem_0.6rem_rgba(0,0,0,0.1)] overflow-hidden">
      <div class="flex items-center justify-between py-[0.8rem] px-[1rem] bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] text-white">
        <button class="bg-white/20 border-none text-white py-[0.5rem] px-[1rem] rounded-[0.4rem] cursor-pointer text-[0.9rem] transition-all duration-200 hover:bg-white/30 hover:scale-105" @click="goBack">← {{ t('common.back') }}</button>
        <span class="text-[1.1rem] font-bold">{{ t('wordDetailPage.title') }}</span>
        <div class="w-20"></div>
      </div>

      <div v-if="isLoading" class="flex-1 flex flex-col items-center justify-center gap-4 text-[#666]">
        <div class="w-10 h-10 border-[3px] border-[#4caf50] border-t-transparent rounded-full animate-spin"></div>
        <span>{{ t('wordDetailPage.loading') }}</span>
      </div>

      <div v-else-if="!wordData" class="flex-1 flex flex-col items-center justify-center gap-4 text-[#666]">
        <p>{{ t('wordDetailPage.notFound') }}</p>
        <button class="bg-white/20 border-none text-white py-[0.5rem] px-[1rem] rounded-[0.4rem] cursor-pointer text-[0.9rem] transition-all duration-200 hover:bg-white/30 hover:scale-105" @click="goBack">{{ t('common.back') }}</button>
      </div>

      <div v-else class="flex-1 overflow-y-auto p-[1.5rem]">
        <div class="bg-white rounded-2xl p-8 mb-4 shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)]">
          <div class="text-center">
            <div class="flex items-start justify-center gap-4">
              <div class="flex flex-col items-center gap-[0.3rem]" v-for="(char, idx) in wordData.word.split('')" :key="idx">
                <div class="text-[3rem] font-bold text-[#333]">{{ char }}</div>
                <div class="text-[3rem] text-[#666]" v-if="coloredPinyinParts[idx]">
                  <span class="text-[#4caf50]">{{ coloredPinyinParts[idx].initial }}</span><span class="text-[#2196f3]">{{
                    coloredPinyinParts[idx].final }}</span>
                </div>
              </div>
              <button class="bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] text-white border-none cursor-pointer transition-all duration-300 hover:scale-110 hover:shadow-[0_0.3rem_0.6rem_rgba(76,175,80,0.4)] active:scale-95 py-[0.6rem] px-[0.8rem] rounded-full text-[1.2rem] shadow-[0_0.2rem_0.4rem_rgba(76,175,80,0.3)]" @click="speakWord">
                🔊
              </button>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-2xl p-4 flex justify-around mb-4 shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)]">
          <div class="flex flex-col items-center gap-[0.3rem]">
            <span class="text-[0.8rem] text-[#999]">{{ t('wordDetailPage.charCount') }}</span>
            <span class="text-[1.2rem] font-bold text-[#333]">{{ wordData.word_length }}</span>
          </div>
          <div class="flex flex-col items-center gap-[0.3rem]">
            <span class="text-[0.8rem] text-[#999]">{{ t('wordDetailPage.firstChar') }}</span>
            <span class="text-[1.2rem] font-bold text-[#333]">{{ wordData.first_char }}</span>
          </div>
          <div class="flex flex-col items-center gap-[0.3rem]">
            <span class="text-[0.8rem] text-[#999]">{{ t('wordDetailPage.frequency') }}</span>
            <span class="text-[1.2rem] font-bold text-[#333]">{{ frequencyScore }}</span>
          </div>
        </div>

        <div class="bg-white rounded-2xl p-[1.5rem] shadow-[0_0.2rem_0.4rem_rgba(0,0,0,0.1)]">
          <div class="flex items-center justify-between mb-4 pb-2 border-b-2 border-[#4caf50]">
            <div class="text-[1rem] font-bold text-[#4caf50]">{{ t('wordDetailPage.explanation') }}</div>
            <button class="bg-[linear-gradient(135deg,#4caf50_0%,#45a049_100%)] text-white border-none cursor-pointer transition-all duration-300 hover:scale-110 hover:shadow-[0_0.3rem_0.6rem_rgba(76,175,80,0.4)] active:scale-95 py-[0.4rem] px-[0.8rem] rounded-[0.5rem] text-[0.85rem] shadow-[0_0.15rem_0.3rem_rgba(76,175,80,0.2)]" @click="speakExplanation" v-if="getExplanationTextForSpeak">
              🔊 {{ t('wordDetailPage.readExplanation') }}
            </button>
          </div>
          <div class="explanation-content text-[0.95rem] leading-[1.8] text-[#333]" v-html="extractExplanationText"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.explanation-content :deep(entry) {
  display: block;
}

.explanation-content :deep(hw) {
  display: none;
}

.explanation-content :deep(py) {
  display: none;
}

.explanation-content :deep(def) {
  display: block;
  margin-bottom: 0.8rem;
  padding-left: 1rem;
  border-left: 3px solid #4caf50;
}

.explanation-content :deep(ps) {
  color: #4caf50;
  font-weight: bold;
  margin-right: 0.5rem;
}

.explanation-content :deep(num) {
  font-weight: bold;
  margin-right: 0.3rem;
}

.explanation-content :deep(ci) {
  display: block;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #eee;
}

.explanation-content :deep(.title) {
  font-size: 0.85rem;
  color: #999;
  margin-bottom: 0.5rem;
}

.explanation-content :deep(.cont) {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.explanation-content :deep(.cont a) {
  color: #4caf50;
  text-decoration: none;
  padding: 0.3rem 0.6rem;
  background: #e8f5e9;
  border-radius: 0.3rem;
  font-size: 0.85rem;
}
</style>
