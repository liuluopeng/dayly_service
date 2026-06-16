-- 1. 用户授权文件夹
CREATE TABLE IF NOT EXISTS user_directories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    path VARCHAR NOT NULL,
    label VARCHAR DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, path)
);
CREATE INDEX IF NOT EXISTS idx_user_directories_user_id ON user_directories(user_id);

-- 2. 媒体类型表
CREATE TABLE IF NOT EXISTS media_type (
    id VARCHAR PRIMARY KEY,
    label VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);
INSERT INTO media_type (id, label) VALUES
    ('song', '歌曲'), ('video', '视频'), ('photo', '照片'), ('book', '图书')
ON CONFLICT (id) DO NOTHING;

-- 3. 媒体路径表
CREATE TABLE IF NOT EXISTS media_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    directory_id UUID NOT NULL REFERENCES user_directories(id) ON DELETE CASCADE,
    media_type VARCHAR NOT NULL REFERENCES media_type(id),
    path VARCHAR NOT NULL,
    label VARCHAR DEFAULT '',
    scan_when_start BOOLEAN DEFAULT false,
    scan_when_change BOOLEAN DEFAULT true,
    last_scan_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, path)
);
CREATE INDEX IF NOT EXISTS idx_media_paths_user_id ON media_paths(user_id);
CREATE INDEX IF NOT EXISTS idx_media_paths_directory_id ON media_paths(directory_id);
CREATE INDEX IF NOT EXISTS idx_media_paths_user_type ON media_paths(user_id, media_type);

-- 4. 触发器: media_paths.path 必须是 user_directories.path 的子路径
CREATE OR REPLACE FUNCTION check_media_path_in_directory()
RETURNS TRIGGER AS $$
DECLARE dir_path VARCHAR;
BEGIN
    SELECT path INTO dir_path FROM user_directories WHERE id = NEW.directory_id;
    IF dir_path IS NULL THEN
        RAISE EXCEPTION 'directory_id % 不存在', NEW.directory_id;
    END IF;
    IF NEW.path NOT LIKE dir_path || '%' THEN
        RAISE EXCEPTION '媒体路径 % 不在授权目录 % 内', NEW.path, dir_path;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_media_path_check ON media_paths;
CREATE TRIGGER trg_media_path_check
    BEFORE INSERT OR UPDATE OF path, directory_id ON media_paths
    FOR EACH ROW EXECUTE FUNCTION check_media_path_in_directory();

-- 5. songs 表新增 media_path_id
ALTER TABLE songs ADD COLUMN IF NOT EXISTS media_path_id UUID REFERENCES media_paths(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_songs_media_path_id ON songs(media_path_id);

-- 6. 歌单表
CREATE TABLE IF NOT EXISTS playlists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, name)
);
CREATE INDEX IF NOT EXISTS idx_playlists_user_id ON playlists(user_id);

-- 7. 歌单-歌曲关联表
CREATE TABLE IF NOT EXISTS playlist_songs (
    playlist_id UUID NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (playlist_id, song_id)
);
CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist_id ON playlist_songs(playlist_id);
CREATE INDEX IF NOT EXISTS idx_playlist_songs_song_id ON playlist_songs(song_id);
