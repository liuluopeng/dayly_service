import { ref, computed, onMounted, watch } from 'vue';

export function useApiPort() {
  const apiPort = ref<string | null>(null);

  const hasPort = computed(() => {
    return !!apiPort.value;
  });

  // 从 localStorage 读取初始值
  onMounted(() => {
    const savedPort = localStorage.getItem('API_PORT');
    if (savedPort) {
      apiPort.value = savedPort;
    }
  });

  // 监听变化并保存到 localStorage
  watch(apiPort, (newPort) => {
    if (newPort) {
      localStorage.setItem('API_PORT', newPort);
    } else {
      localStorage.removeItem('API_PORT');
    }
  });

  const setPort = (port: string) => {
    apiPort.value = port;
  };

  const clearPort = () => {
    apiPort.value = null;
  };

  return {
    apiPort,
    hasPort,
    setPort,
    clearPort
  };
}
