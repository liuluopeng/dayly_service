-- 配置类种子数据

-- 媒体类型字典（代码 add_media_path 支持 song/video/photo/book/melatonin）
INSERT INTO media_type (id, label) VALUES
    ('song', '歌曲'),
    ('video', '视频'),
    ('photo', '照片'),
    ('book', '图书'),
    ('dmm', 'dmm'),
    ('melatonin', 'melatonin')
ON CONFLICT (id) DO NOTHING;
