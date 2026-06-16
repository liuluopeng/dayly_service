<template>
  <div class="h-screen bg-black flex flex-col">
    <div class="flex items-center gap-3 py-2 px-4 bg-[#111] shrink-0">
      <button
        class="bg-[#333] text-[#aaa] border-0 py-1 px-3 rounded cursor-pointer text-[13px] hover:bg-[#444] hover:text-white"
        @click="goBack"
      >
        ← {{ t('videoPlayer.back') }}
      </button>
      <span class="text-[#ccc] text-sm truncate flex-1">{{ fileName }}</span>
      <div class="flex gap-1">
        <button
          v-for="s in speeds"
          :key="s"
          class="border-0 py-1 px-2.5 rounded cursor-pointer text-xs font-semibold min-w-[36px]"
          :class="currentSpeed === s ? 'bg-[#64b5f6] text-black' : 'bg-[#333] text-[#aaa] hover:bg-[#444] hover:text-white'"
          @click="setSpeed(s)"
        >
          {{ s }}x
        </button>
      </div>
    </div>
    <div ref="playerContainer" class="player-wrap flex-1 flex items-center justify-center overflow-hidden"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useFiles } from '../composables/useFiles';
import DPlayer from 'dplayer';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { getAuthenticatedFileUrl } = useFiles();

const playerContainer = ref<HTMLDivElement | null>(null);
let dp: InstanceType<typeof DPlayer> | null = null;

const speeds = [0.5, 1, 1.5, 2, 2.5, 3];
const currentSpeed = ref(1);

const filePath = computed(() => decodeURIComponent(route.query.path as string || ''));
const videoUrl = computed(() => getAuthenticatedFileUrl(filePath.value));
const fileName = computed(() => filePath.value.split('/').pop() || '');

function goBack() {
  router.back();
}

function setSpeed(s: number) {
  currentSpeed.value = s;
  if (dp?.video) {
    dp.video.playbackRate = s;
  }
}

onMounted(() => {
  if (!playerContainer.value) return;
  dp = new DPlayer({
    container: playerContainer.value,
    autoplay: true,
    theme: '#64b5f6',
    lang: 'zh-cn',
    screenshot: true,
    hotkey: true,
    video: {
      url: videoUrl.value,
      type: 'auto',
    },
  });
});

onBeforeUnmount(() => {
  dp?.destroy();
});
</script>

<style scoped>
.player-wrap :deep(.dplayer) {
  max-width: 100%;
  max-height: 100%;
  width: 100%;
  height: 100%;
}
</style>
