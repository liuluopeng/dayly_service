<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import MarkdownIt from "markdown-it";
import {
  create_session,
  list_sessions,
  add_message,
  get_session_messages,
} from "../types/wasm-typed";
import type { OpenAiSession, OpenAiMessage } from "../types/models";

const router = useRouter();
const { t, locale } = useI18n();
const md = new MarkdownIt();

// 会话列表
const sessions = ref<OpenAiSession[]>([]);
const loading = ref(false);
const error = ref("");

// 当前选中的会话
const currentSession = ref<OpenAiSession | null>(null);
const messages = ref<OpenAiMessage[]>([]);
const messagesLoading = ref(false);

// 输入框
const inputMessage = ref("");
const inputRef = ref<HTMLTextAreaElement | null>(null);
const sending = ref(false);
const streaming = ref(false);
const streamingContent = ref("");
const streamingThink = ref("");

// 模型选择
const models = [
  { value: "deepseek-chat", label: "DeepSeek Chat" },
  { value: "deepseek-reasoner", label: "DeepSeek Reasoner (带思考过程)" }
];
const selectedModel = ref("deepseek-chat");

// 临时会话状态（未保存到数据库）
const tempSessionTitle = ref("");
const isTempSession = ref(false);

// 消息容器引用（用于自动滚动）
const messagesContainer = ref<HTMLDivElement | null>(null);

// 加载会话列表
async function loadSessions() {
  loading.value = true;
  error.value = "";
  try {
    const result = await list_sessions();
    console.log("会话列表响应:", result);
    sessions.value = result || [];
  } catch (err) {
    console.error("加载会话失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.loadFailed');
  } finally {
    loading.value = false;
  }
}



// 创建新会话（临时，不立即保存到数据库）
async function createNewSession() {
  // 创建临时会话状态
  const title = t('openAiChat.newChatTitle', { time: new Date().toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US') });
  tempSessionTitle.value = title;
  isTempSession.value = true;
  currentSession.value = null;
  messages.value = [];
  // 清空输入框
  inputMessage.value = "";
  // 自动对焦到输入框
  await nextTick();
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100);
  // 滚动到底部
  await nextTick();
  scrollToBottom();
}

// 选择会话
async function selectSession(session: OpenAiSession) {
  currentSession.value = session;
  isTempSession.value = false;
  await loadMessages(session.id);
  // 自动对焦到输入框
  await nextTick();
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100);
}

// 加载消息列表
async function loadMessages(sessionId: string) {
  messagesLoading.value = true;
  try {
    const result = await get_session_messages(sessionId);
    console.log("消息列表响应:", result);
    messages.value = result || [];
  } catch (err) {
    console.error("加载消息失败:", err);
  } finally {
    messagesLoading.value = false;
    await nextTick();
    scrollToBottom();
  }
}

// 发送消息
async function sendMessage() {
  if (!inputMessage.value.trim()) return;

  const content = inputMessage.value.trim();
  inputMessage.value = "";
  sending.value = true;

  try {
    let sessionId = currentSession.value?.id;

    // 如果是临时会话，先创建会话
    if (isTempSession.value || !sessionId) {
      const result = await create_session(tempSessionTitle.value);
      sessionId = result.id;
      currentSession.value = result;
      isTempSession.value = false;
      await loadSessions();
    }

    if (!sessionId) {
      throw new Error("会话ID不存在");
    }

    // 保存用户消息到数据库
    await add_message(sessionId, "user", content, undefined, undefined);

    // 重新加载消息列表
    await loadMessages(sessionId);

    // 调用流式API
    await streamChatCompletion({
      model: selectedModel.value,
      messages: messages.value.map(m => ({
        role: m.role,
        content: m.content
      })),
      session_id: sessionId
    }, sessionId);

  } catch (err) {
    console.error("发送消息失败:", err);
    error.value = err instanceof Error ? err.message : t('common.error.saveFailed');
  } finally {
    sending.value = false;
    // 发送完成后重新对焦到输入框
    await nextTick();
    setTimeout(() => {
      inputRef.value?.focus();
    }, 100);
  }
}

// 流式传输聊天完成
async function streamChatCompletion(request: any, sessionId: string) {
  streaming.value = true;
  streamingContent.value = "";
  streamingThink.value = "";

  try {
    // 获取API基础URL
    const apiUrl = localStorage.getItem('apiUrl') ||
      (import.meta.env.DEV ? `http://${window.location.hostname}:23001` : `http://${window.location.hostname}:23000`);

    const response = await fetch(`${apiUrl}/api/openai/chat/completions/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const reader = response.body?.getReader();
    if (!reader) {
      throw new Error('No response body');
    }

    const decoder = new TextDecoder();
    let fullContent = '';
    let fullThink = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value, { stream: true });
      const lines = chunk.split('\n');

      for (const line of lines) {
        if (line.startsWith('data: ') && !line.includes('[DONE]')) {
          const data = line.slice(6);
          try {
            const json = JSON.parse(data);
            if (json.choices && json.choices[0]?.delta) {
              const delta = json.choices[0].delta;

              // 处理content
              if (delta.content) {
                const content = delta.content;
                streamingContent.value += content;
                fullContent += content;
              }

              // 处理reasoning_content
              if (delta.reasoning_content) {
                const think = delta.reasoning_content;
                streamingThink.value += think;
                fullThink += think;
              }

              // 处理cite（通常在最后一个chunk中）
              if (delta.cite) {
              }

              await nextTick();
              scrollToBottom();
            }
          } catch (e) {
            console.error('解析流式数据失败:', e);
          }
        }
      }
    }

    // 等待一小段时间，确保后端有足够时间保存AI回复到数据库
    await new Promise(resolve => setTimeout(resolve, 500));

    // 流结束后，重新加载消息列表
    await loadMessages(sessionId);

    // 加载完成后再清空
    streaming.value = false;
    streamingContent.value = "";
    streamingThink.value = "";
  } catch (err) {
    console.error('流式传输失败:', err);
    streaming.value = false;
    streamingContent.value = "";
    streamingThink.value = "";
    throw err;
  }
}

// 滚动到底部
function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

// 格式化时间
function formatTime(timeStr: string): string {
  const date = new Date(timeStr);
  return date.toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US');
}

// 返回菜单
function goBack() {
  router.push("/menu");
}

// 页面加载时获取会话列表
onMounted(() => {
  loadSessions();
  // 自动对焦到输入框
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100);
});
</script>

<template>
  <div class="flex h-screen bg-gray-100 dark:bg-gray-900">
    <!-- 左侧会话列表 -->
    <div class="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
      <!-- 标题栏 -->
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-white">{{ t('openAiChat.sessionList') }}</h2>
          <button @click="createNewSession"
            class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors text-sm">
            {{ t('openAiChat.newSession') }}
          </button>
        </div>
      </div>

      <!-- 会话列表 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="loading" class="text-center text-gray-500 dark:text-gray-400 py-4">
          {{ t('openAiChat.loading') }}
        </div>
        <div v-else-if="error" class="text-center text-red-500 py-4">
          {{ error }}
        </div>
        <div v-else-if="sessions.length === 0" class="text-center text-gray-500 dark:text-gray-400 py-4">
          {{ t('openAiChat.noSessions') }}
        </div>
        <div v-else v-for="session in sessions" :key="session.id" @click="selectSession(session)" :class="[
          'p-3 rounded-lg cursor-pointer mb-2 transition-colors',
          currentSession?.id === session.id
            ? 'bg-blue-100 dark:bg-blue-900 border-l-4 border-blue-500'
            : 'bg-gray-50 dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600'
        ]">
          <div class="font-medium text-gray-800 dark:text-white truncate">
            {{ session.title }}
          </div>
          <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {{ formatTime(session.updated_at) }}
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧聊天区域 -->
    <div class="flex-1 flex flex-col">
      <!-- 聊天头部 -->
      <div class="p-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <h1 class="text-xl font-semibold text-gray-800 dark:text-white">
            {{ currentSession?.title || tempSessionTitle || t('openAiChat.selectOrCreate') }}
          </h1>
          <div class="flex items-center gap-4">
            <!-- 模型选择 -->
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('openAiChat.model') }}</span>
              <select v-model="selectedModel"
                class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                <option v-for="model in models" :key="model.value" :value="model.value">
                  {{ model.label }}
                </option>
              </select>
            </div>
            <button @click="goBack"
              class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 transition-colors">
              {{ t('openAiChat.backToMenu') }}
            </button>
          </div>
        </div>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="flex-1 overflow-y-auto p-4 space-y-4">
        <div v-if="messages.length === 0 && !streaming" class="text-center text-gray-500 dark:text-gray-400 py-8">
          {{ t('openAiChat.startConversation') }}
        </div>

        <div v-for="message in messages" :key="message.id" :class="[
          'flex',
          message.role === 'user' ? 'justify-end' : 'justify-start'
        ]">
          <div :class="[
            'max-w-3xl rounded-lg p-4',
            message.role === 'user'
              ? 'bg-blue-500 text-white'
              : 'bg-white dark:bg-gray-700 text-gray-800 dark:text-white border border-gray-200 dark:border-gray-600'
          ]">
            <!-- 思考过程（仅助手消息显示） -->
            <div v-if="message.role === 'assistant' && message.think"
              class="mb-3 p-2 bg-gray-100 dark:bg-gray-600 rounded text-sm text-gray-600 dark:text-gray-300">
              <div class="font-medium text-xs text-gray-500 dark:text-gray-400 mb-1">{{ t('openAiChat.thinkingProcess') }}</div>
              <div class="whitespace-pre-wrap">{{ message.think }}</div>
            </div>

            <!-- 消息内容 -->
            <div class="prose dark:prose-invert max-w-none" v-html="md.render(message.content)"></div>

            <!-- 引用来源（仅助手消息显示） -->
            <div v-if="message.role === 'assistant' && message.cite && message.cite.length > 0"
              class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
              <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">{{ t('openAiChat.referenceSources') }}</div>
              <div class="space-y-2">
                <div v-for="cite in message.cite" :key="cite.cite_index" class="text-sm">
                  <a :href="cite.url" target="_blank"
                    class="text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300 flex items-center gap-2">
                    <img v-if="cite.site_icon" :src="cite.site_icon" class="w-4 h-4" :alt="cite.site_name" />
                    <span>[{{ cite.cite_index }}] {{ cite.title }}</span>
                  </a>
                  <div class="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
                    {{ cite.snippet }}
                  </div>
                </div>
              </div>
            </div>

            <div class="text-xs opacity-70 mt-2">
              {{ formatTime(message.created_at) }}
            </div>
          </div>
        </div>

        <!-- 流式传输中的消息 -->
        <div v-if="streaming" class="flex justify-start">
          <div
            class="max-w-3xl rounded-lg p-4 bg-white dark:bg-gray-700 text-gray-800 dark:text-white border border-gray-200 dark:border-gray-600">
            <!-- 思考过程 -->
            <div v-if="streamingThink"
              class="mb-3 p-2 bg-gray-100 dark:bg-gray-600 rounded text-sm text-gray-600 dark:text-gray-300">
              <div class="font-medium text-xs text-gray-500 dark:text-gray-400 mb-1">{{ t('openAiChat.thinkingProcess') }}</div>
              <div class="whitespace-pre-wrap">{{ streamingThink }}</div>
            </div>

            <!-- 消息内容 -->
            <div class="prose dark:prose-invert max-w-none" v-html="md.render(streamingContent)"></div>

            <div class="flex items-center gap-2 mt-2">
              <div class="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
              <span class="text-xs text-gray-500 dark:text-gray-400">{{ t('openAiChat.aiThinking') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="p-4 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
        <div class="flex gap-2">
          <textarea ref="inputRef" v-model="inputMessage" @keydown.enter.prevent="sendMessage" :placeholder="t('openAiChat.inputPlaceholder')"
            class="flex-1 p-3 border border-gray-300 dark:border-gray-600 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            rows="3" :disabled="sending"></textarea>
          <button @click="sendMessage" :disabled="sending || !inputMessage.trim()"
            class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
            {{ sending ? t('openAiChat.sending') : t('openAiChat.send') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

