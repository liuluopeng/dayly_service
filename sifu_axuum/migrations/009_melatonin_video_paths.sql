-- 添加多视频支持
ALTER TABLE melatonin_movies ADD COLUMN IF NOT EXISTS video_paths TEXT[] NOT NULL DEFAULT '{}';

-- 迁移现有数据：将单个 video_path 转为数组
UPDATE melatonin_movies SET video_paths = ARRAY[video_path] WHERE video_path != '' AND cardinality(video_paths) = 0;
