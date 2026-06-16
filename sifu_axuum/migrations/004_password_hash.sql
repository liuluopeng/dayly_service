-- 添加密码哈希列
ALTER TABLE users ADD COLUMN IF NOT EXISTS hash VARCHAR;

-- 注意：现有的明文密码需要通过 hash_existing_passwords 工具迁移到 hash 列
-- 迁移完成后可以删除 password 列：
-- ALTER TABLE users DROP COLUMN password;
