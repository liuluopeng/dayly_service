-- 003: user_directories + media_paths 改为 allow_list 模式
-- 路径全局唯一，用 TEXT[] 存储授权用户名单

-- === user_directories ===
ALTER TABLE user_directories ADD COLUMN IF NOT EXISTS allow_list TEXT[] DEFAULT '{}';

UPDATE user_directories ud
SET allow_list = ARRAY[(SELECT username FROM users WHERE id = ud.user_id)]
WHERE allow_list = '{}' OR allow_list IS NULL;

ALTER TABLE user_directories DROP CONSTRAINT IF EXISTS user_directories_user_id_path_key;
DROP INDEX IF EXISTS idx_user_directories_user_id;
ALTER TABLE user_directories DROP COLUMN IF EXISTS user_id;
ALTER TABLE user_directories ADD CONSTRAINT user_directories_path_key UNIQUE (path);

-- === media_paths ===
ALTER TABLE media_paths ADD COLUMN IF NOT EXISTS allow_list TEXT[] DEFAULT '{}';

UPDATE media_paths mp
SET allow_list = ARRAY[(SELECT username FROM users WHERE id = mp.user_id)]
WHERE allow_list = '{}' OR allow_list IS NULL;

ALTER TABLE media_paths DROP CONSTRAINT IF EXISTS media_paths_user_id_path_key;
DROP INDEX IF EXISTS idx_media_paths_user_id;
DROP INDEX IF EXISTS idx_media_paths_user_type;
ALTER TABLE media_paths DROP COLUMN IF EXISTS user_id;
ALTER TABLE media_paths ADD CONSTRAINT media_paths_path_key UNIQUE (path);
CREATE INDEX IF NOT EXISTS idx_media_paths_media_type ON media_paths(media_type);
