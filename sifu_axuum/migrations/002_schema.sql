-- 全量基础表结构（依据当前生产库 pg_dump --schema-only 导出整理）
-- 幂等风格：全部 IF NOT EXISTS / DO 块，可安全重复执行

-- ─── 序列 ────────────────────────────────────────────────

CREATE SEQUENCE IF NOT EXISTS clipboard_entries_id_seq
    START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;

CREATE SEQUENCE IF NOT EXISTS haiyu_dict_id_seq
    AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;

CREATE SEQUENCE IF NOT EXISTS pinyin_quick_words_id_seq
    AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;

CREATE SEQUENCE IF NOT EXISTS single_char_pinyin_id_seq
    AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;

-- ─── 用户与权限 ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    username character varying(255) NOT NULL,
    password character varying(255) NOT NULL,
    directory character varying,
    is_admin boolean DEFAULT false,
    hash character varying,
    language character varying DEFAULT 'zh',
    flutter_theme character varying DEFAULT 'dark'
);

CREATE TABLE IF NOT EXISTS webdav_users (
    id uuid NOT NULL,
    username character varying NOT NULL,
    password character varying NOT NULL,
    permission character varying NOT NULL,
    directorie character varying NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_users (
    id integer NOT NULL,
    username character varying,
    password character varying
);

CREATE TABLE IF NOT EXISTS user_directories (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    path character varying NOT NULL,
    label character varying DEFAULT '',
    created_at timestamp with time zone DEFAULT now(),
    allow_list text[] DEFAULT '{}'
);

-- 媒体类型字典（含代码使用的全部 6 种）
CREATE TABLE IF NOT EXISTS media_type (
    id character varying NOT NULL,
    label character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);

-- 媒体路径（path 全局唯一，allow_list 为授权用户名单）
CREATE TABLE IF NOT EXISTS media_paths (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    directory_id uuid NOT NULL,
    media_type character varying NOT NULL,
    path character varying NOT NULL,
    label character varying DEFAULT '',
    scan_when_start boolean DEFAULT false,
    scan_when_change boolean DEFAULT true,
    last_scan_time timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    allow_list text[] DEFAULT '{}'
);

-- ─── 歌曲与歌单 ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS songs (
    id uuid NOT NULL,
    title character varying,
    path character varying,
    artist character varying,
    album character varying,
    cover_path character varying,
    cover_data bytea,
    lrc text,
    ttml text,
    eslrc text,
    qrc text,
    yrc text,
    lys text,
    media_path_id uuid,
    vocal bytea,
    auto_ttml text
);

CREATE TABLE IF NOT EXISTS albums (
    id uuid NOT NULL,
    title character varying
);

CREATE TABLE IF NOT EXISTS artists (
    id uuid NOT NULL,
    name character varying
);

CREATE TABLE IF NOT EXISTS playlists (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

CREATE TABLE IF NOT EXISTS playlist_songs (
    playlist_id uuid NOT NULL,
    song_id uuid NOT NULL,
    sort_order integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now()
);

-- ─── 图片 / 视频 / Melatonin ─────────────────────────────

CREATE TABLE IF NOT EXISTS images (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying NOT NULL,
    path character varying NOT NULL,
    folder_path character varying NOT NULL,
    media_path_id uuid NOT NULL,
    size bigint DEFAULT 0,
    width integer,
    height integer,
    format character varying,
    created_at timestamp with time zone DEFAULT now(),
    thumbnail bytea
);

CREATE TABLE IF NOT EXISTS videos (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying NOT NULL,
    path character varying NOT NULL,
    folder_path character varying NOT NULL,
    media_path_id uuid NOT NULL,
    size bigint DEFAULT 0,
    duration_ms bigint,
    format character varying,
    width integer,
    height integer,
    created_at timestamp with time zone DEFAULT now(),
    preview bytea
);

CREATE TABLE IF NOT EXISTS melatonin_movies (
    id uuid NOT NULL,
    title character varying NOT NULL,
    cover_path character varying NOT NULL,
    video_path character varying NOT NULL,
    nfo_json jsonb NOT NULL,
    media_path_id uuid,
    video_paths text[] DEFAULT '{}' NOT NULL,
    bt_list jsonb
);

-- ─── 笔记 / 短笔记 / 视图 ────────────────────────────────

CREATE TABLE IF NOT EXISTS notes (
    id uuid NOT NULL,
    text text,
    simple_text text,
    raw_content bytea,
    filepath character varying,
    filename character varying,
    file_info jsonb,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    sha256 character varying,
    url character varying,
    subject character varying,
    user_id uuid
);

CREATE TABLE IF NOT EXISTS short_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    content text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    view_name character varying,
    view_id uuid,
    user_id uuid
);

CREATE TABLE IF NOT EXISTS view_names (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    view_name character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

-- ─── 聊天 / 剪贴板 / OpenAI ──────────────────────────────

CREATE TABLE IF NOT EXISTS chat_messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sender_id uuid NOT NULL,
    content text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS clipboard_entries (
    id bigint NOT NULL,
    entry_type text NOT NULL,
    text_content text,
    image_path text,
    content_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS openai_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    title character varying(255) NOT NULL,
    user_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    message_count integer DEFAULT 0
);

CREATE TABLE IF NOT EXISTS openai_messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    session_id uuid NOT NULL,
    role character varying(50) NOT NULL,
    content text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    think text,
    cite jsonb
);

-- ─── 词频 / 词典数据表 ───────────────────────────────────

CREATE TABLE IF NOT EXISTS words (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    word character varying(255) NOT NULL,
    has_searched_times integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS word_histories (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    word character varying(255) NOT NULL,
    "time" timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    user_id uuid
);

CREATE TABLE IF NOT EXISTS pinyin_dict (
    pinyin text NOT NULL,
    words jsonb NOT NULL
);

CREATE TABLE IF NOT EXISTS pinyin_quick_words (
    id integer NOT NULL,
    pinyin text NOT NULL,
    words jsonb DEFAULT '[]' NOT NULL
);

CREATE TABLE IF NOT EXISTS single_char_pinyin (
    id integer NOT NULL,
    pinyin text NOT NULL,
    ori text NOT NULL,
    count integer NOT NULL,
    pinyin_length integer NOT NULL,
    first_letter text NOT NULL
);

CREATE TABLE IF NOT EXISTS haiyu_dict (
    id integer NOT NULL,
    word text NOT NULL,
    pinyin_flat text NOT NULL,
    frequency integer NOT NULL,
    word_length integer NOT NULL,
    first_char text NOT NULL,
    pinyin_no_flat jsonb
);

-- ─── 默认值（序列） ───────────────────────────────────────

ALTER SEQUENCE clipboard_entries_id_seq OWNED BY clipboard_entries.id;
ALTER SEQUENCE haiyu_dict_id_seq OWNED BY haiyu_dict.id;
ALTER SEQUENCE pinyin_quick_words_id_seq OWNED BY pinyin_quick_words.id;
ALTER SEQUENCE single_char_pinyin_id_seq OWNED BY single_char_pinyin.id;

ALTER TABLE ONLY clipboard_entries ALTER COLUMN id SET DEFAULT nextval('clipboard_entries_id_seq'::regclass);
ALTER TABLE ONLY haiyu_dict ALTER COLUMN id SET DEFAULT nextval('haiyu_dict_id_seq'::regclass);
ALTER TABLE ONLY pinyin_quick_words ALTER COLUMN id SET DEFAULT nextval('pinyin_quick_words_id_seq'::regclass);
ALTER TABLE ONLY single_char_pinyin ALTER COLUMN id SET DEFAULT nextval('single_char_pinyin_id_seq'::regclass);

-- ─── 主键与唯一约束 ───────────────────────────────────────

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'admin_users_pk' AND conrelid = 'admin_users'::regclass) THEN
        ALTER TABLE admin_users ADD CONSTRAINT admin_users_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'albums_pk' AND conrelid = 'albums'::regclass) THEN
        ALTER TABLE albums ADD CONSTRAINT albums_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'artists_pk' AND conrelid = 'artists'::regclass) THEN
        ALTER TABLE artists ADD CONSTRAINT artists_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chat_messages_pkey' AND conrelid = 'chat_messages'::regclass) THEN
        ALTER TABLE chat_messages ADD CONSTRAINT chat_messages_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'clipboard_entries_pkey' AND conrelid = 'clipboard_entries'::regclass) THEN
        ALTER TABLE clipboard_entries ADD CONSTRAINT clipboard_entries_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'haiyu_dict_pkey' AND conrelid = 'haiyu_dict'::regclass) THEN
        ALTER TABLE haiyu_dict ADD CONSTRAINT haiyu_dict_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'haiyu_dict_word_key' AND conrelid = 'haiyu_dict'::regclass) THEN
        ALTER TABLE haiyu_dict ADD CONSTRAINT haiyu_dict_word_key UNIQUE (word);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'images_pkey' AND conrelid = 'images'::regclass) THEN
        ALTER TABLE images ADD CONSTRAINT images_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'images_path_key' AND conrelid = 'images'::regclass) THEN
        ALTER TABLE images ADD CONSTRAINT images_path_key UNIQUE (path);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'media_paths_pkey' AND conrelid = 'media_paths'::regclass) THEN
        ALTER TABLE media_paths ADD CONSTRAINT media_paths_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'media_paths_path_key' AND conrelid = 'media_paths'::regclass) THEN
        ALTER TABLE media_paths ADD CONSTRAINT media_paths_path_key UNIQUE (path);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'media_type_pkey' AND conrelid = 'media_type'::regclass) THEN
        ALTER TABLE media_type ADD CONSTRAINT media_type_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'newtable_pk' AND conrelid = 'melatonin_movies'::regclass) THEN
        ALTER TABLE melatonin_movies ADD CONSTRAINT newtable_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notes_pkey' AND conrelid = 'notes'::regclass) THEN
        ALTER TABLE notes ADD CONSTRAINT notes_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'openai_messages_pkey' AND conrelid = 'openai_messages'::regclass) THEN
        ALTER TABLE openai_messages ADD CONSTRAINT openai_messages_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'openai_sessions_pkey' AND conrelid = 'openai_sessions'::regclass) THEN
        ALTER TABLE openai_sessions ADD CONSTRAINT openai_sessions_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pinyin_dict_pkey' AND conrelid = 'pinyin_dict'::regclass) THEN
        ALTER TABLE pinyin_dict ADD CONSTRAINT pinyin_dict_pkey PRIMARY KEY (pinyin);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pinyin_quick_words_pkey' AND conrelid = 'pinyin_quick_words'::regclass) THEN
        ALTER TABLE pinyin_quick_words ADD CONSTRAINT pinyin_quick_words_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'pinyin_quick_words_pinyin_key' AND conrelid = 'pinyin_quick_words'::regclass) THEN
        ALTER TABLE pinyin_quick_words ADD CONSTRAINT pinyin_quick_words_pinyin_key UNIQUE (pinyin);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'playlist_songs_pkey' AND conrelid = 'playlist_songs'::regclass) THEN
        ALTER TABLE playlist_songs ADD CONSTRAINT playlist_songs_pkey PRIMARY KEY (playlist_id, song_id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'playlists_pkey' AND conrelid = 'playlists'::regclass) THEN
        ALTER TABLE playlists ADD CONSTRAINT playlists_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'playlists_user_id_name_key' AND conrelid = 'playlists'::regclass) THEN
        ALTER TABLE playlists ADD CONSTRAINT playlists_user_id_name_key UNIQUE (user_id, name);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'short_notes_pkey' AND conrelid = 'short_notes'::regclass) THEN
        ALTER TABLE short_notes ADD CONSTRAINT short_notes_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'single_char_pinyin_pkey' AND conrelid = 'single_char_pinyin'::regclass) THEN
        ALTER TABLE single_char_pinyin ADD CONSTRAINT single_char_pinyin_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'single_char_pinyin_pinyin_key' AND conrelid = 'single_char_pinyin'::regclass) THEN
        ALTER TABLE single_char_pinyin ADD CONSTRAINT single_char_pinyin_pinyin_key UNIQUE (pinyin);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'songs_pk' AND conrelid = 'songs'::regclass) THEN
        ALTER TABLE songs ADD CONSTRAINT songs_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_directories_pkey' AND conrelid = 'user_directories'::regclass) THEN
        ALTER TABLE user_directories ADD CONSTRAINT user_directories_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_directories_path_key' AND conrelid = 'user_directories'::regclass) THEN
        ALTER TABLE user_directories ADD CONSTRAINT user_directories_path_key UNIQUE (path);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_pkey' AND conrelid = 'users'::regclass) THEN
        ALTER TABLE users ADD CONSTRAINT users_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_username_key' AND conrelid = 'users'::regclass) THEN
        ALTER TABLE users ADD CONSTRAINT users_username_key UNIQUE (username);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'videos_pkey' AND conrelid = 'videos'::regclass) THEN
        ALTER TABLE videos ADD CONSTRAINT videos_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'videos_path_key' AND conrelid = 'videos'::regclass) THEN
        ALTER TABLE videos ADD CONSTRAINT videos_path_key UNIQUE (path);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'view_names_pkey' AND conrelid = 'view_names'::regclass) THEN
        ALTER TABLE view_names ADD CONSTRAINT view_names_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'webdav_users_pk' AND conrelid = 'webdav_users'::regclass) THEN
        ALTER TABLE webdav_users ADD CONSTRAINT webdav_users_pk PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'word_historys_pkey' AND conrelid = 'word_histories'::regclass) THEN
        ALTER TABLE word_histories ADD CONSTRAINT word_historys_pkey PRIMARY KEY (id);
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'words_pkey' AND conrelid = 'words'::regclass) THEN
        ALTER TABLE words ADD CONSTRAINT words_pkey PRIMARY KEY (id);
    END IF;
END $$;
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'words_word_key' AND conrelid = 'words'::regclass) THEN
        ALTER TABLE words ADD CONSTRAINT words_word_key UNIQUE (word);
    END IF;
END $$;

-- ─── 外键 ─────────────────────────────────────────────────

DO $$ BEGIN
    ALTER TABLE ONLY chat_messages ADD CONSTRAINT chat_messages_sender_id_fkey
        FOREIGN KEY (sender_id) REFERENCES users(id);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY melatonin_movies ADD CONSTRAINT dmm_movies_media_path_id_fkey
        FOREIGN KEY (media_path_id) REFERENCES media_paths(id);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY media_paths ADD CONSTRAINT fk_media_paths_media_type
        FOREIGN KEY (media_type) REFERENCES media_type(id);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY notes ADD CONSTRAINT fk_notes_user_id
        FOREIGN KEY (user_id) REFERENCES users(id);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY word_histories ADD CONSTRAINT fk_word_histories_user_id
        FOREIGN KEY (user_id) REFERENCES users(id);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY images ADD CONSTRAINT images_media_path_id_fkey
        FOREIGN KEY (media_path_id) REFERENCES media_paths(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY media_paths ADD CONSTRAINT media_paths_directory_id_fkey
        FOREIGN KEY (directory_id) REFERENCES user_directories(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY openai_messages ADD CONSTRAINT openai_messages_session_id_fkey
        FOREIGN KEY (session_id) REFERENCES openai_sessions(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY playlist_songs ADD CONSTRAINT playlist_songs_playlist_id_fkey
        FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY playlist_songs ADD CONSTRAINT playlist_songs_song_id_fkey
        FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY playlists ADD CONSTRAINT playlists_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY songs ADD CONSTRAINT songs_media_path_id_fkey
        FOREIGN KEY (media_path_id) REFERENCES media_paths(id) ON DELETE SET NULL;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE ONLY videos ADD CONSTRAINT videos_media_path_id_fkey
        FOREIGN KEY (media_path_id) REFERENCES media_paths(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
