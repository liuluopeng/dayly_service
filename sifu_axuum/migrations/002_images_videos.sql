-- images 表：存储扫描到的图片文件信息
CREATE TABLE IF NOT EXISTS images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    path VARCHAR NOT NULL UNIQUE,
    folder_path VARCHAR NOT NULL,
    media_path_id UUID NOT NULL REFERENCES media_paths(id) ON DELETE CASCADE,
    size BIGINT DEFAULT 0,
    width INT,
    height INT,
    format VARCHAR,
    thumbnail BYTEA,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_images_media_path_id ON images(media_path_id);
CREATE INDEX IF NOT EXISTS idx_images_folder_path ON images(folder_path);

-- videos 表：存储扫描到的视频文件信息
CREATE TABLE IF NOT EXISTS videos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    path VARCHAR NOT NULL UNIQUE,
    folder_path VARCHAR NOT NULL,
    media_path_id UUID NOT NULL REFERENCES media_paths(id) ON DELETE CASCADE,
    size BIGINT DEFAULT 0,
    duration_ms BIGINT,
    format VARCHAR,
    width INT,
    height INT,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_videos_media_path_id ON videos(media_path_id);
CREATE INDEX IF NOT EXISTS idx_videos_folder_path ON videos(folder_path);
