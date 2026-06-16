<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { save_note_wasm } from "../types/wasm-typed";

const router = useRouter();
const { t, locale } = useI18n();

// 日历相关
const currentDate = ref(new Date());
const selectedDate = ref<Date | null>(null);

const currentYear = computed(() => currentDate.value.getFullYear());
const currentMonth = computed(() => currentDate.value.getMonth());

const weekDays = computed(() => {
  const days = t('home.weekDays');
  return Array.isArray(days) ? days : ['日', '一', '二', '三', '四', '五', '六'];
});

const calendarDays = computed(() => {
  const year = currentYear.value;
  const month = currentMonth.value;

  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);

  const daysInMonth = lastDay.getDate();
  const startingDayOfWeek = firstDay.getDay();

  const days: (number | null)[] = [];

  for (let i = 0; i < startingDayOfWeek; i++) {
    days.push(null);
  }

  for (let i = 1; i <= daysInMonth; i++) {
    days.push(i);
  }

  return days;
});

const isToday = (day: number | null): boolean => {
  if (!day) return false;
  const today = new Date();
  return (
    day === today.getDate() &&
    currentMonth.value === today.getMonth() &&
    currentYear.value === today.getFullYear()
  );
};

const isSelected = (day: number | null): boolean => {
  if (!day || !selectedDate.value) return false;
  return (
    day === selectedDate.value.getDate() &&
    currentMonth.value === selectedDate.value.getMonth() &&
    currentYear.value === selectedDate.value.getFullYear()
  );
};

const selectDate = (day: number | null) => {
  if (day) {
    selectedDate.value = new Date(currentYear.value, currentMonth.value, day);
  }
};

const prevMonth = () => {
  currentDate.value = new Date(currentYear.value, currentMonth.value - 1, 1);
};

const nextMonth = () => {
  currentDate.value = new Date(currentYear.value, currentMonth.value + 1, 1);
};

// 新建笔记
const noteContent = ref("");
const noteInputRef = ref<HTMLTextAreaElement | null>(null);
const isSaving = ref(false);
const saveStatus = ref("");

onMounted(() => {
  setTimeout(() => {
    noteInputRef.value?.focus();
  }, 100);
});

const createNote = async () => {
  if (!noteContent.value.trim()) return;

  isSaving.value = true;
  saveStatus.value = t('common.saving');

  try {
    await save_note_wasm(
      null,
      noteContent.value,
      undefined
    );

    saveStatus.value = t('common.save') + "!";
    noteContent.value = "";

    setTimeout(() => {
      saveStatus.value = "";
    }, 2000);
  } catch (err) {
    saveStatus.value = t('common.error.saveFailed');
    console.error(t('common.error.saveFailed'), err);

    setTimeout(() => {
      saveStatus.value = "";
    }, 2000);
  } finally {
    isSaving.value = false;
  }
};

// 剪贴板
const clipboardText = ref("");
const isReadingClipboard = ref(false);

const readClipboard = async () => {
  isReadingClipboard.value = true;
  try {
    const text = await navigator.clipboard.readText();
    clipboardText.value = text;
  } catch (err) {
    console.error(t('home.clipboard.readFailed'), err);
    clipboardText.value = t('home.clipboard.readFailed');
  } finally {
    isReadingClipboard.value = false;
  }
};

const copyToClipboard = async () => {
  if (!clipboardText.value) return;

  try {
    await navigator.clipboard.writeText(clipboardText.value);
    saveStatus.value = t('common.copied');
    setTimeout(() => {
      saveStatus.value = "";
    }, 2000);
  } catch (err) {
    console.error(t('common.copy') + " " + t('common.error.saveFailed'), err);
  }
};

onMounted(() => {
  readClipboard();
});

// 当前时间
const currentTime = ref(new Date());
const formattedTime = computed(() => {
  return currentTime.value.toLocaleTimeString(locale.value === 'zh' ? 'zh-CN' : 'en-US', {
    hour: "2-digit",
    minute: "2-digit"
  });
});
const formattedDate = computed(() => {
  return currentTime.value.toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', {
    month: "long",
    day: "numeric",
    weekday: "long"
  });
});

setInterval(() => {
  currentTime.value = new Date();
}, 1000);
</script>

<template>
  <div class="min-h-[100vh] bg-[linear-gradient(135deg,#1a1a2e_0%,#16213e_100%)] p-4 md:p-8">
    <!-- 大磁贴区域 -->
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-[repeat(auto-fit,minmax(120px,1fr))] auto-rows-[120px] gap-2 sm:gap-3 max-w-[1200px] mx-auto">

      <!-- 时间磁贴 - 大 -->
      <div class="col-span-1 sm:col-span-2 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#0078d4_0%,#106ebe_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex-1 flex flex-col justify-center">
          <div class="text-[2rem] md:text-[3rem] font-light leading-none">{{ formattedTime }}</div>
          <div class="text-base mt-2 opacity-90">{{ formattedDate }}</div>
        </div>
      </div>

      <!-- 日历磁贴 - 中 -->
      <div class="col-span-1 sm:col-span-2 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#f7630c_0%,#d65a0a_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex justify-between items-center mb-3">
          <button @click="prevMonth" class="bg-white/20 border-none text-white w-7 h-7 rounded-sm cursor-pointer text-sm transition-colors duration-200 hover:bg-white/30">&lt;</button>
          <span class="text-base font-semibold">{{ currentMonth + 1 }}{{ t('home.month') }}</span>
          <button @click="nextMonth" class="bg-white/20 border-none text-white w-7 h-7 rounded-sm cursor-pointer text-sm transition-colors duration-200 hover:bg-white/30">&gt;</button>
        </div>
        <div class="flex-1 flex flex-col">
          <div class="grid grid-cols-7 gap-0.5 mb-1">
            <span v-for="day in weekDays" :key="day" class="text-center text-[0.625rem] opacity-80 p-0.5">{{ day }}</span>
          </div>
          <div class="grid grid-cols-7 gap-0.5 flex-1">
            <span v-for="(day, index) in calendarDays" :key="index" :class="[
              'flex items-center justify-center text-xs p-1 rounded-sm cursor-pointer transition-colors duration-200',
              { 'bg-white/90 text-[#f7630c] font-semibold': isToday(day) },
              { 'bg-white/40': isSelected(day) },
              { 'cursor-default': !day }
            ]" @click="selectDate(day)"
              :style="day ? { background: isToday(day) ? 'rgba(255,255,255,0.9)' : isSelected(day) ? 'rgba(255,255,255,0.4)' : undefined } : {}"
              @mouseenter="day && (($event.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.2)')"
              @mouseleave="day && (($event.currentTarget as HTMLElement).style.background = isToday(day) ? 'rgba(255,255,255,0.9)' : isSelected(day) ? 'rgba(255,255,255,0.4)' : '')"
            >
              {{ day }}
            </span>
          </div>
        </div>
      </div>

      <!-- 快速笔记磁贴 - 宽 -->
      <div class="col-span-1 sm:col-span-2 md:col-span-4 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#107c10_0%,#0e6e0e_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex justify-between items-center mb-3">
          <span class="text-base font-semibold">{{ '📝 ' + t('home.quickNote.title') }}</span>
          <span v-if="saveStatus" class="text-xs opacity-90">{{ saveStatus }}</span>
        </div>
        <textarea ref="noteInputRef" v-model="noteContent" :placeholder="t('home.quickNote.placeholder')"
          class="bg-white/15 border border-white/20 rounded-sm p-2 text-white text-sm resize-none flex-1 mb-2 placeholder:text-white/60 focus:outline-none focus:bg-white/20 focus:border-white/40"
          rows="3" @keydown.ctrl.enter="createNote"></textarea>
        <button @click="createNote" :disabled="isSaving || !noteContent.trim()"
          class="bg-white/90 border-none text-[#333] py-2 px-4 rounded-sm text-sm font-medium cursor-pointer transition-all duration-200 self-end hover:not-disabled:bg-white disabled:opacity-50 disabled:cursor-not-allowed">
          {{ isSaving ? t('common.saving') : t('common.save') }}
        </button>
      </div>

      <!-- 剪贴板磁贴 - 中 -->
      <div class="col-span-1 sm:col-span-2 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#881798_0%,#6e1280_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex justify-between items-center mb-3">
          <span class="text-base font-semibold">{{ '📋 ' + t('home.clipboard.title') }}</span>
          <button @click="readClipboard" :disabled="isReadingClipboard"
            class="bg-white/20 border-none text-white w-6 h-6 rounded-sm cursor-pointer text-xs transition-colors duration-200 hover:bg-white/30 disabled:opacity-50 disabled:cursor-not-allowed">
            ↻
          </button>
        </div>
        <textarea v-model="clipboardText" :placeholder="t('home.clipboard.placeholder')"
          class="bg-white/15 border border-white/20 rounded-sm p-1.5 text-white text-xs resize-none flex-1 mb-2 placeholder:text-white/60 focus:outline-none focus:bg-white/20 focus:border-white/40"
          rows="2" readonly></textarea>
        <button @click="copyToClipboard" :disabled="!clipboardText"
          class="bg-white/90 border-none text-[#333] py-1.5 px-3 rounded-sm text-xs font-medium cursor-pointer transition-all duration-200 self-end hover:not-disabled:bg-white disabled:opacity-50 disabled:cursor-not-allowed">
          {{ t('common.copy') }}
        </button>
      </div>

      <!-- 统计磁贴 - 小 -->
      <div class="col-span-1 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#d13438_0%,#b92d31_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="text-[2.5rem] font-light leading-none">12</div>
          <div class="text-sm mt-1 opacity-90">{{ t('home.notes') }}</div>
        </div>
      </div>

      <!-- 待办磁贴 - 小 -->
      <div class="col-span-1 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#038387_0%,#027578_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="text-[2.5rem] font-light leading-none">5</div>
          <div class="text-sm mt-1 opacity-90">{{ t('home.todo') }}</div>
        </div>
      </div>

      <!-- 设置磁贴 - 小 -->
      <div class="col-span-1 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#605e5c_0%,#4a4846_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5" @click="router.push('/settings')">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="text-[2.5rem] mb-2">⚙️</div>
          <div class="text-sm mt-1 opacity-90">{{ t('home.settings') }}</div>
        </div>
      </div>

      <!-- 笔记列表磁贴 - 宽 -->
      <div class="col-span-1 sm:col-span-2 md:col-span-4 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#4f52b2_0%,#3f4190_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5" @click="router.push('/notes')">
        <div class="flex justify-between items-center mb-3">
          <span class="text-base font-semibold">{{ '📚 ' + t('home.viewAllNotes') }}</span>
        </div>
        <div class="text-sm opacity-90 mt-auto">{{ t('home.viewAllNotesDesc') }}</div>
      </div>

      <!-- 工具磁贴 - 中 -->
      <div class="col-span-1 sm:col-span-2 row-span-2 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#e3008c_0%,#c4007a_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5" @click="router.push('/tools')">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="text-[2.5rem] mb-2">🧰</div>
          <div class="text-sm mt-1 opacity-90">{{ t('home.toolbox') }}</div>
        </div>
      </div>

      <!-- 空白占位磁贴 - 小 -->
      <div class="col-span-1 rounded p-4 text-white cursor-pointer transition-all duration-200 ease-in-out flex flex-col overflow-hidden relative bg-[linear-gradient(135deg,#323130_0%,#252423_100%)] hover:-translate-y-1 hover:shadow-[0_8px_25px_rgba(0,0,0,0.3)] active:-translate-y-0.5">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="text-[2.5rem] mb-2">+</div>
        </div>
      </div>

    </div>
  </div>
</template>
