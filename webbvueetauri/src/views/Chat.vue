<template>
  <div class="h-screen flex flex-col bg-[#1a1a2e] text-[#eee]">
    <div class="flex items-center py-2 px-4 border-b border-[#333] gap-3">
      <button class="bg-[#2a2a4a] text-[#64b5f6] border border-[#444] py-1 px-3 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#3a3a5a]" @click="goBack">&larr; {{ t('common.back') }}</button>
      <span class="flex-1 text-center text-[15px]">聊天</span>
    </div>

    <div ref="messagesContainer" class="flex-1 overflow-y-auto p-4 space-y-3">
      <div v-if="loading" class="text-center text-[#888]">{{ t('common.loading') }}</div>
      <div v-for="msg in messages" :key="msg.id"
        class="flex flex-col"
        :class="msg.sender_id === currentUserId ? 'items-end' : 'items-start'"
      >
        <div class="text-[11px] text-[#888] mb-1">
          {{ msg.username }} · {{ formatTime(msg.created_at) }}
        </div>
        <div class="max-w-[70%] rounded-lg px-3 py-2 text-[14px]"
          :class="msg.sender_id === currentUserId ? 'bg-[#3a5a8a]' : 'bg-[#2a2a4a]'"
        >
          {{ msg.content }}
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2 py-2 px-4 border-t border-[#333]">
      <input
        v-model="newMessage"
        class="flex-1 bg-[#2a2a4a] border border-[#444] rounded px-3 py-2 text-[14px] text-[#eee] outline-none focus:border-[#64b5f6]"
        placeholder="输入消息..."
        @keyup.enter="sendMessage"
      />
      <button
        class="bg-[#64b5f6] text-[#1a1a2e] border-0 rounded px-4 py-2 cursor-pointer text-[14px] font-bold hover:bg-[#90caf9] disabled:opacity-50 disabled:cursor-default"
        :disabled="!newMessage.trim() || sending"
        @click="sendMessage"
      >
        发送
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { send_message_wasm, get_messages_wasm, connect_chat_ws } from '../types/wasm-typed';
import type { ChatMessageWithUsername } from '../types/models';
import { useAuth } from '../composables/useAuth';

const { t } = useI18n();
const router = useRouter();
const { username } = useAuth();

const messages = ref<ChatMessageWithUsername[]>([]);
const newMessage = ref('');
const loading = ref(true);
const sending = ref(false);
const messagesContainer = ref<HTMLElement | null>(null);
const currentUserId = ref('');
let ws: WebSocket | null = null;
let lastMessageTime: string | null = null;

function goBack() {
  router.push('/menu');
}

function formatTime(ts: string): string {
  const d = new Date(ts);
  const pad = (n: number) => n.toString().padStart(2, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  });
}

function connectWebSocket() {
  ws = connect_chat_ws('/api/chat/ws', (text: string) => {
    try {
      const msg: ChatMessageWithUsername = JSON.parse(text);
      if (!messages.value.some(m => m.id === msg.id)) {
        messages.value.push(msg);
        lastMessageTime = msg.created_at;
        scrollToBottom();
      }
    } catch (e) {
      console.error('Failed to parse WS message:', e);
    }
  });
}

async function loadMessages() {
  try {
    const res = await get_messages_wasm(lastMessageTime);
    const data = res?.data;
    if (!data) return;

    if (lastMessageTime) {
      if (data.length > 0) {
        messages.value.push(...data);
        lastMessageTime = data[data.length - 1].created_at;
        scrollToBottom();
      }
    } else {
      messages.value = data.reverse();
      if (messages.value.length > 0) {
        lastMessageTime = messages.value[messages.value.length - 1].created_at;
        const first = messages.value.find(m => m.username === username.value);
        if (first) currentUserId.value = first.sender_id;
      }
      scrollToBottom();
    }
  } catch (e) {
    console.error('Failed to load messages:', e);
  }
}

async function sendMessage() {
  const content = newMessage.value.trim();
  if (!content || sending.value) return;

  sending.value = true;
  try {
    const res = await send_message_wasm(content);
    if (res?.data) {
      const msg: ChatMessageWithUsername = {
        id: res.data.id,
        sender_id: res.data.sender_id,
        username: username.value || '我',
        content: res.data.content,
        created_at: res.data.created_at,
      };
      messages.value.push(msg);
      currentUserId.value = res.data.sender_id;
      lastMessageTime = msg.created_at;
      newMessage.value = '';
      scrollToBottom();
    }
  } catch (e) {
    console.error('Failed to send message:', e);
  } finally {
    sending.value = false;
  }
}

onMounted(async () => {
  await loadMessages();
  loading.value = false;
  connectWebSocket();
});

onUnmounted(() => {
  if (ws) {
    ws.close();
    ws = null;
  }
});
</script>
