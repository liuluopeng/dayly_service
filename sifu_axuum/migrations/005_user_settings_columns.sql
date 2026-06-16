-- 新增 language 和 flutter_theme 列
ALTER TABLE users ADD COLUMN IF NOT EXISTS language VARCHAR DEFAULT 'zh';
ALTER TABLE users ADD COLUMN IF NOT EXISTS flutter_theme VARCHAR DEFAULT 'dark';

-- 从 settings JSONB 迁移数据
UPDATE users SET
    flutter_theme = COALESCE(settings->>'flutter_theme', 'dark'),
    language = COALESCE(settings->>'language', 'zh')
WHERE settings IS NOT NULL AND settings != '{}'::jsonb;

-- 弃用 settings 列
ALTER TABLE users DROP COLUMN IF EXISTS settings;
