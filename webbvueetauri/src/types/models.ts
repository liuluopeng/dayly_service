// Shared API response and model types
// Source of truth: Rust structs in my_type/src/dto/ and my_type/src/model/
// Field type mapping: Uuid→string, Option<T>→T|null, Vec<T>→T[], i64/i32→number, DateTime→string

// === 通用 ===

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

// === 歌曲 ===

export interface SongWithUrl {
  id: string;
  title: string;
  path: string;
  album?: string;
  artist?: string;
  cover_path?: string;
  cover_url?: string;
}

export interface LyricsResponse {
  song_id: string;
  title: string;
  artist?: string;
  lyrics: string;
}

export interface LyricsLine {
  time: number;
  text: string;
}

export interface AllLyricsResponse {
  song_id: string;
  title: string;
  artist?: string;
  lrc?: string;
  ttml?: string;
  eslrc?: string;
  qrc?: string;
  yrc?: string;
  lys?: string;
  auto_ttml?: string;
}

// === 视频 ===

export interface VideoItem {
  id: string;
  name: string;
  path: string;
  serve_url?: string;
  folder_path: string;
  size: number;
  duration_ms?: number;
  format?: string;
  width?: number;
  height?: number;
}

// === 图片 ===

export interface ImageItem {
  id: string;
  name: string;
  path: string;
  serve_url?: string;
  folder_path: string;
  size: number;
  width?: number;
  height?: number;
  format?: string;
}

// === 文件 ===

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  last_modified?: string | null;
}

export interface DirListing {
  path: string;
  entries: FileEntry[];
  total: number;
}

// === 用户/目录 ===

export interface UserItem {
  id: string;
  username: string;
  is_admin: boolean;
}

export interface UserDirectory {
  id: string;
  path: string;
  label: string | null;
  allow_list: string[];
  created_at: string | null;
}

export interface MediaPathItem {
  id: string;
  path: string;
  label: string | null;
  media_type: string;
  allow_list: string[];
}

// === 短笔记 ===

export interface ShortNote {
  id: string;
  content?: string;
  view_name?: string;
  created_at: string;
}

// === 词典/搜索历史 ===

export interface WordHistory {
  id: string;
  word: string;
  time: string;
  created_at: string;
}

export interface Word {
  id: string;
  word: string;
  hasSearchedTimes: number;
}

// === 五笔 ===

export interface GgttResult {
  char?: string;
  code_86?: string;
  has_diagram?: boolean;
  svg1?: string;
  svg2?: string;
  svg3?: string;
  svg4?: string;
}

// === AI 对话 ===

export interface CiteItem {
  url: string;
  title: string;
  snippet: string;
  cite_index: number;
  published_at: string | null;
  site_name: string;
  site_icon: string;
  query_indexes: number[];
}

export interface OpenAiSession {
  id: string;
  title: string;
  user_id?: string;
  created_at: string;
  updated_at: string;
}

export interface OpenAiMessage {
  id: string;
  session_id: string;
  role: string;
  content: string;
  think?: string;
  cite?: CiteItem[];
  created_at: string;
}

// === 聊天 ===

export interface ChatMessage {
  id: string;
  sender_id: string;
  content: string;
  created_at: string;
}

export interface ChatMessageWithUsername {
  id: string;
  sender_id: string;
  username: string;
  content: string;
  created_at: string;
}

export interface RecentContact {
  user_id: string;
  username: string;
  last_message: string;
  last_message_at: string;
}

export interface ContactItem {
  id: string;
  username: string;
}

// === 剪贴板 ===

export interface ClipboardEntry {
  id: number;
  type: string;
  text_content?: string | null;
  content_hash: string;
  created_at: string;
  image_url?: string | null;
  image_path?: string | null;
}
