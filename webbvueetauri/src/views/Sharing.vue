<template>
  <div class="h-screen flex flex-col bg-[#1a1a2e] text-[#eee]">
    <div class="flex items-center py-2 px-4 border-b border-[#333] gap-3">
      <button class="bg-[#2a2a4a] text-[#64b5f6] border border-[#444] py-1 px-3 rounded cursor-pointer text-[13px] whitespace-nowrap hover:bg-[#3a3a5a]" @click="goBack">&larr; {{ t('common.back') }}</button>
      <span class="flex-1 text-center text-[15px]">局域网共享</span>
      <span class="w-2 h-2 rounded-full" :class="connected ? 'bg-green-500' : 'bg-gray-500'"></span>
    </div>

    <!-- 在线设备 -->
    <div v-if="allPeers.length > 0" class="flex gap-2 px-4 py-1.5 overflow-x-auto border-b border-[#333]">
      <span v-for="p in allPeers" :key="p.id"
        class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 whitespace-nowrap text-[11px]"
        :class="p.id === '__self' ? 'bg-[#1a2a4a] text-[#64b5f6]' : 'bg-[#2a2a4a] text-[#aaa]'"
      >
        <span class="w-1.5 h-1.5 rounded-full" :class="p.id === '__self' ? 'bg-blue-500' : 'bg-green-500'"></span>{{ p.name }}
      </span>
    </div>

    <div class="flex border-b border-[#333]">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="flex-1 py-2 text-[13px] border-b-2 transition-colors"
        :class="activeTab === tab.id ? 'border-[#64b5f6] text-[#64b5f6]' : 'border-transparent text-[#888]'"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- 剪贴板 Tab -->
    <div v-if="activeTab === 'clipboard'" class="flex-1 overflow-y-auto flex flex-col">
      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="clipboardHistory.length === 0" class="flex flex-col items-center justify-center h-full text-[#888]">
          <span class="text-[36px] mb-3">📋</span>
          <span>{{ connected ? '复制文本即可同步' : '正在连接...' }}</span>
        </div>
        <div v-for="(item, i) in clipboardHistory" :key="i"
          class="mb-3 p-3 bg-[#2a2a4a] rounded-lg cursor-pointer hover:bg-[#3a3a5a]"
          @click="copyToClipboard(item.content)"
        >
          <div class="text-[14px] text-[#eee] whitespace-pre-wrap break-all line-clamp-3">{{ item.content }}</div>
          <div class="text-[11px] text-[#888] mt-1">{{ item.from }} · {{ formatTime(item.time) }}</div>
        </div>
      </div>
      <!-- 手动输入框 -->
      <div class="p-2 border-t border-[#333] flex gap-2">
        <input
          v-model="clipboardInput"
          class="flex-1 bg-[#2a2a4a] text-[#eee] border border-[#444] rounded py-2 px-3 text-[13px] outline-none focus:border-[#64b5f6]"
          placeholder="输入文本发送..."
          @keyup.enter="sendManualClipboard"
        />
        <button
          class="bg-[#64b5f6] text-[#1a1a2e] border-0 rounded px-4 cursor-pointer text-[13px] hover:bg-[#90caf9]"
          @click="sendManualClipboard"
        >发送</button>
      </div>
    </div>

    <!-- 文件传输 Tab -->
    <div v-if="activeTab === 'files'" class="flex-1 overflow-y-auto flex flex-col">
      <div class="p-4">
        <button
          class="w-full bg-[#64b5f6] text-[#1a1a2e] border-0 rounded py-3 cursor-pointer text-[14px] font-bold hover:bg-[#90caf9]"
          @click="pickAndSendFile"
        >
          选择文件发送
        </button>
      </div>

      <div v-if="transfers.length === 0" class="flex-1 flex items-center justify-center text-[#888]">
        暂无文件传输
      </div>

      <div v-for="(t, i) in transfers" :key="i" class="mx-4 mb-3 p-3 bg-[#2a2a4a] rounded-lg">
        <div class="flex items-center gap-2">
          <span :class="t.direction === 'send' ? 'text-[#64b5f6]' : 'text-green-400'">
            {{ t.direction === 'send' ? '↑' : '↓' }}
          </span>
          <span class="flex-1 text-[14px] truncate">{{ t.name }}</span>
          <span class="text-[12px] text-[#888]">{{ formatSize(t.size) }}</span>
        </div>
        <div class="mt-2 h-1 bg-[#444] rounded overflow-hidden">
          <div class="h-full transition-all rounded" :class="t.status === 'completed' ? 'bg-green-500' : 'bg-[#64b5f6]'" :style="{width: (t.progress * 100) + '%'}"></div>
        </div>
        <div class="flex items-center justify-between mt-1">
          <span class="text-[11px] text-[#888]">
            {{ t.status === 'completed' ? '完成' : Math.round(t.progress * 100) + '%' }}
          </span>
          <button
            v-if="t.status === 'completed' && t.direction === 'receive' && t.chunks.length > 0"
            class="text-[11px] text-[#64b5f6] bg-transparent border border-[#64b5f6] rounded px-2 py-0.5 cursor-pointer hover:bg-[#64b5f6] hover:text-[#1a1a2e]"
            @click="downloadFile(t)"
          >下载</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useAuth } from '../composables/useAuth';
import { getApiUrl } from '../utils/apiUrl';

const { t } = useI18n();
const router = useRouter();
const { username } = useAuth();

const tabs = [
  { id: 'clipboard', label: '剪贴板' },
  { id: 'files', label: '文件传输' },
];
const activeTab = ref('clipboard');
const connected = ref(false);
const myName = ref('');
const peers = ref<{ id: string; name: string }[]>([]);
const allPeers = computed(() => {
  const list = [...peers.value];
  if (myName.value) {
    list.unshift({ id: '__self', name: myName.value + '(我)' });
  }
  return list;
});
const clipboardHistory = ref<{ content: string; from: string; time: string }[]>([]);
const transfers = ref<{
  file_id: string; name: string; size: number;
  direction: string; progress: number; status: string; chunks: string[];
}[]>([]);

let ws: WebSocket | null = null;
let pc: RTCPeerConnection | null = null;
let dc: RTCDataChannel | null = null;
let lastClipboard = '';
let clipboardTimer: ReturnType<typeof setInterval> | null = null;
const clipboardInput = ref('');

function goBack() {
  router.push('/menu');
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts);
    const pad = (n: number) => n.toString().padStart(2, '0');
    return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  } catch { return ''; }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function sendWs(msg: object) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg));
  }
}

function sendData(data: object) {
  if (dc && dc.readyState === 'open') {
    dc.send(JSON.stringify(data));
  }
}

async function setupWebRTC() {
  // 信令 WebSocket
  const apiUrl = getApiUrl();
  const wsUrl = apiUrl.replace(/^http/, 'ws') + '/api/webrtc/signaling?token=' + encodeURIComponent(localStorage.getItem('token') || '');
  ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    // 每个标签页分配唯一编号
    const tabKey = '__sharing_tab_id';
    let tabId = sessionStorage.getItem(tabKey);
    if (!tabId) {
      tabId = String(Date.now() % 10000);
      sessionStorage.setItem(tabKey, tabId);
    }
    myName.value = (username.value || 'Web') + '-Tab' + tabId;
    sendWs({ type: 'register', name: myName.value });
  };

  ws.onmessage = async (event) => {
    const msg = JSON.parse(event.data);

    if (msg.type === 'registered') {
      // 创建 PeerConnection
      pc = new RTCPeerConnection();

      pc.onicecandidate = (e) => {
        if (e.candidate) {
          sendWs({
            type: 'candidate',
            candidate: e.candidate.candidate,
            sdp_mid: e.candidate.sdpMid,
            sdp_mline_index: e.candidate.sdpMLineIndex,
          });
        }
      };

      // 创建 DataChannel
      dc = pc.createDataChannel('sharing');

      dc.onopen = () => {
        connected.value = true;
      };

      dc.onmessage = (e) => {
        handleDataMessage(JSON.parse(e.data));
      };

      dc.onclose = () => {
        connected.value = false;
      };

      // 创建 Offer
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      sendWs({ type: 'offer', sdp: offer.sdp });
    }

    if (msg.type === 'answer') {
      await pc?.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: msg.sdp }));
    }

    if (msg.type === 'candidate') {
      await pc?.addIceCandidate(new RTCIceCandidate({
        candidate: msg.candidate,
        sdpMid: msg.sdp_mid,
        sdpMLineIndex: msg.sdp_mline_index,
      }));
    }

    // 在线列表更新（通过 WebSocket 信令）
    if (msg.type === 'peer_list') {
      peers.value = (msg.peers || []).filter((p: any) => p.id !== '__self');
    }
  };

  ws.onclose = () => {
    connected.value = false;
    setTimeout(setupWebRTC, 3000);
  };
}

function handleDataMessage(msg: any) {
  if (msg.type === 'clipboard') {
    clipboardHistory.value.unshift({
      content: msg.content,
      from: msg.from,
      time: new Date().toISOString(),
    });
  } else if (msg.type === 'file_offer') {
    transfers.value.push({
      file_id: msg.file_id,
      name: msg.name,
      size: msg.size,
      direction: 'receive',
      progress: 0,
      status: 'receiving',
      chunks: [],
    });
  } else if (msg.type === 'file_chunk') {
    const idx = transfers.value.findIndex(t => t.file_id === msg.file_id);
    if (idx >= 0) {
      transfers.value[idx].chunks.push(msg.data);
    }
  } else if (msg.type === 'file_end') {
    const idx = transfers.value.findIndex(t => t.file_id === msg.file_id);
    if (idx >= 0) {
      transfers.value[idx].status = 'completed';
      transfers.value[idx].progress = 1;
    }
  }
}

async function checkClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    if (text && text !== lastClipboard) {
      lastClipboard = text;
      const from = myName.value || 'Web';
      sendData({ type: 'clipboard', content: text, from });
      clipboardHistory.value.unshift({ content: text, from, time: new Date().toISOString() });
    }
  } catch {}
}

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text);
  lastClipboard = text;
}

function downloadFile(t: { name: string; chunks: string[] }) {
  const bytes = t.chunks.flatMap(b64 => {
    const binary = atob(b64);
    return Array.from(binary, c => c.charCodeAt(0));
  });
  const blob = new Blob([new Uint8Array(bytes)]);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = t.name;
  a.click();
  URL.revokeObjectURL(url);
}

function sendManualClipboard() {
  const text = clipboardInput.value.trim();
  if (!text) return;
  const from = myName.value || 'Web';
  sendData({ type: 'clipboard', content: text, from });
  clipboardHistory.value.unshift({ content: text, from, time: new Date().toISOString() });
  clipboardInput.value = '';
}

async function pickAndSendFile() {
  const input = document.createElement('input');
  input.type = 'file';
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;

    const fileId = Date.now().toString();
    const buffer = await file.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    transfers.value.push({
      file_id: fileId,
      name: file.name,
      size: file.size,
      direction: 'send',
      progress: 0,
      status: 'sending',
      chunks: [],
    });

    sendData({ type: 'file_offer', file_id: fileId, name: file.name, size: file.size });

    const chunkSize = 65536;
    const totalChunks = Math.ceil(bytes.length / chunkSize);

    for (let i = 0; i < totalChunks; i++) {
      const start = i * chunkSize;
      const end = Math.min(start + chunkSize, bytes.length);
      const chunk = bytes.slice(start, end);

      // base64 encode
      const binary = Array.from(chunk).map(b => String.fromCharCode(b)).join('');
      const b64 = btoa(binary);

      sendData({ type: 'file_chunk', file_id: fileId, index: i, data: b64 });

      const idx = transfers.value.findIndex(t => t.file_id === fileId);
      if (idx >= 0) transfers.value[idx].progress = (i + 1) / totalChunks;
    }

    sendData({ type: 'file_end', file_id: fileId });

    const idx = transfers.value.findIndex(t => t.file_id === fileId);
    if (idx >= 0) {
      transfers.value[idx].status = 'completed';
      transfers.value[idx].progress = 1;
    }
  };
  input.click();
}

onMounted(() => {
  setupWebRTC();
  clipboardTimer = setInterval(checkClipboard, 1000);
});

onUnmounted(() => {
  if (clipboardTimer) clearInterval(clipboardTimer);
  dc?.close();
  pc?.close();
  ws?.close();
});
</script>
