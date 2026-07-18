import { createRouter, createWebHashHistory } from 'vue-router';
import Home from '../views/Home.vue';
import Menu from '../views/Menu.vue';
import GgttSearch from '../views/GgttSearch.vue';
import XiandaihanyuSearch from '../views/XiandaihanyuSearch.vue';
import CollinsSearch from '../views/CollinsSearch.vue';
import LdoceSearch from '../views/LdoceSearch.vue';
import SongView from '../views/SongView.vue';
import MusicPlayer from '../views/MusicPlayer.vue';
import MusicPlayerWasm from '../views/MusicPlayerWasm.vue';
import MusicPlayerLyrics from '../views/MusicPlayerLyrics.vue';
import Login from '../views/Login.vue';
import FileManager from '../views/FileManager.vue';
import VideoPlayer from '../views/VideoPlayer.vue';
import EpubReader from '../views/EpubReader.vue';
import PdfViewer from '../views/PdfViewer.vue';
import ShortNote from '../views/ShortNote.vue';
import Settings from '../views/Settings.vue';
import OpenAiChat from '../views/OpenAiChat.vue';
import NoteSearch from '../views/NoteSearch.vue';
import NoteDetail from '../views/NoteDetail.vue';
import NoteList from '../views/NoteList.vue';
import NoteCreate from '../views/NoteCreate.vue';
import SearchHistory from '../views/SearchHistory.vue';
import ImageGallery from '../views/ImageGallery.vue';
import VideoList from '../views/VideoList.vue';
import UserDirectoryAdmin from '../views/UserDirectoryAdmin.vue';
import Chat from '../views/Chat.vue';
import Sharing from '../views/Sharing.vue';
import ClipboardHistory from '../views/ClipboardHistory.vue';

import Base64Tool from '../views/Base64Tool.vue';
import TimestampTool from '../views/TimestampTool.vue';
import UUIDTool from '../views/UUIDTool.vue';
import PasswordTool from '../views/PasswordTool.vue';
import ChineseRemover from '../views/ChineseRemover.vue';
import ImageConverter from '../views/ImageConverter.vue';
import QrCodeGenerator from '../views/QrCodeGenerator.vue';
import QrCodeScanner from '../views/QrCodeScanner.vue';
import ZiciChars from '../views/zici/HomePage.vue';
import ZiciWords from '../views/zici/WordPage.vue';
import ZiciDictation from '../views/zici/DictationPage.vue';
import ZiciKeyboard from '../views/zici/KeyboardLayoutPage.vue';
import ZiciPinyinPicker from '../views/zici/PinyinPickerPage.vue';
import ZiciPinyinWords from '../views/zici/PinyinWordsPage.vue';
import ZiciWordDetail from '../views/zici/WordDetailPage.vue';
import ZiciWordFrequency from '../views/zici/WordFrequencyPage.vue';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/login',
    name: 'Login',
    component: Login
  },
  {
    path: '/menu',
    name: 'Menu',
    component: Menu
  },
  {
    path: '/ggtt',
    name: 'GgttSearch',
    component: GgttSearch
  },
  {
    path: '/xiandaihanyu',
    name: 'XiandaihanyuSearch',
    component: XiandaihanyuSearch
  },
  {
    path: '/collins',
    name: 'CollinsSearch',
    component: CollinsSearch
  },
  {
    path: '/ldoce',
    name: 'LdoceSearch',
    component: LdoceSearch
  },
  {
    path: '/songs',
    name: 'SongView',
    component: SongView
  },
  {
    path: '/player',
    name: 'MusicPlayer',
    component: MusicPlayer
  },
  {
    path: '/player-wasm',
    name: 'MusicPlayerWasm',
    component: MusicPlayerWasm
  },
  {
    path: '/player-lyrics',
    name: 'MusicPlayerLyrics',
    component: MusicPlayerLyrics
  },
  {
    path: '/files',
    name: 'FileManager',
    component: FileManager
  },
  {
    path: '/video',
    name: 'VideoPlayer',
    component: VideoPlayer
  },
  {
    path: '/epub',
    name: 'EpubReader',
    component: EpubReader
  },
  {
    path: '/pdf',
    name: 'PdfViewer',
    component: PdfViewer
  },
  {
    path: '/short-notes',
    name: 'ShortNote',
    component: ShortNote
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  },
  {
    path: '/openai-chat',
    name: 'OpenAiChat',
    component: OpenAiChat
  },
  {
    path: '/note-search',
    name: 'NoteSearch',
    component: NoteSearch
  },
  {
    path: '/notes',
    name: 'NoteList',
    component: NoteList
  },
  {
    path: '/note-create',
    name: 'NoteCreate',
    component: NoteCreate
  },
  {
    path: '/note/:id',
    name: 'NoteDetail',
    component: NoteDetail
  },
  {
    path: '/search-history',
    name: 'SearchHistory',
    component: SearchHistory
  },

  {
    path: '/tools/base64',
    name: 'Base64Tool',
    component: Base64Tool
  },
  {
    path: '/tools/timestamp',
    name: 'TimestampTool',
    component: TimestampTool
  },
  {
    path: '/tools/uuid',
    name: 'UUIDTool',
    component: UUIDTool
  },
  {
    path: '/tools/password',
    name: 'PasswordTool',
    component: PasswordTool
  },
  {
    path: '/tools/chinese-remover',
    name: 'ChineseRemover',
    component: ChineseRemover
  },
  {
    path: '/tools/image-converter',
    name: 'ImageConverter',
    component: ImageConverter
  },
  {
    path: '/tools/qrcode',
    name: 'QrCodeGenerator',
    component: QrCodeGenerator
  },
  {
    path: '/tools/qrscan',
    name: 'QrCodeScanner',
    component: QrCodeScanner
  },
  {
    path: '/zici/chars',
    name: 'ZiciChars',
    component: ZiciChars
  },
  {
    path: '/zici/words',
    name: 'ZiciWords',
    component: ZiciWords
  },
  {
    path: '/zici/dictation',
    name: 'ZiciDictation',
    component: ZiciDictation
  },
  {
    path: '/zici/keyboard',
    name: 'ZiciKeyboard',
    component: ZiciKeyboard
  },
  {
    path: '/zici/pinyin-picker',
    name: 'ZiciPinyinPicker',
    component: ZiciPinyinPicker
  },
  {
    path: '/zici/pinyin-words',
    name: 'ZiciPinyinWords',
    component: ZiciPinyinWords
  },
  {
    path: '/zici/word-detail',
    name: 'ZiciWordDetail',
    component: ZiciWordDetail
  },
  {
    path: '/zici/word-frequency',
    name: 'ZiciWordFrequency',
    component: ZiciWordFrequency
  },
  {
    path: '/images',
    name: 'ImageGallery',
    component: ImageGallery
  },
  {
    path: '/videos',
    name: 'VideoList',
    component: VideoList
  },
  {
    path: '/admin/user-directories',
    name: 'UserDirectoryAdmin',
    component: UserDirectoryAdmin
  },
  {
    path: '/clipboard-history',
    name: 'ClipboardHistory',
    component: ClipboardHistory
  },
  {
    path: '/chat',
    name: 'Chat',
    component: Chat
  },
  {
    path: '/sharing',
    name: 'Sharing',
    component: Sharing
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/login'
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

// 路由守卫
router.beforeEach((to, _from) => {
  const token = localStorage.getItem('token');

  const requiresAuth = ['/menu', '/ggtt', '/xiandaihanyu', '/collins', '/ldoce', '/songs', '/player', '/player-wasm', '/player-lyrics', '/files', '/video', '/epub', '/pdf', '/short-notes', '/openai-chat', '/note-search', '/notes', '/note-create', '/note', '/tools', '/zici', '/search-history', '/images', '/videos', '/admin', '/settings', '/chat', '/sharing', '/clipboard-history'];

  if (to.path === '/login' && token) {
    return '/menu';
  }

  const requiresAuthMatch = requiresAuth.some(path => to.path.startsWith(path));

  if (requiresAuthMatch && !token) {
    return '/login';
  }

  if (to.path === '/' && !token) {
    return '/login';
  }
});

export default router;