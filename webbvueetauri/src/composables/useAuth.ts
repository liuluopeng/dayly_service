import { ref, computed, watch } from 'vue';

// 模块级单例 — 所有 useAuth() 调用共享同一份状态
const token = ref<string | null>(null);
const tokenType = ref<string | null>(null);
const username = ref<string | null>(null);

const isLoggedIn = computed(() => !!token.value);

// 同步初始化（不用 onMounted，避免时序问题）
const savedToken = localStorage.getItem('token');
if (savedToken) {
  token.value = savedToken;
  tokenType.value = localStorage.getItem('token_type');
}
const savedUsername = localStorage.getItem('savedUsername');
if (savedUsername) {
  username.value = savedUsername;
}

// 监听变化并持久化
watch(token, (v) => {
  v ? localStorage.setItem('token', v) : localStorage.removeItem('token');
});
watch(tokenType, (v) => {
  v ? localStorage.setItem('token_type', v) : localStorage.removeItem('token_type');
});

function login(newToken: string, newTokenType: string, newUsername?: string) {
  token.value = newToken;
  tokenType.value = newTokenType;
  if (newUsername) {
    username.value = newUsername;
    localStorage.setItem('savedUsername', newUsername);
  }
}

function logout() {
  token.value = null;
  tokenType.value = null;
  username.value = null;
}

export function useAuth() {
  return { token, tokenType, username, isLoggedIn, login, logout };
}
