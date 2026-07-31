-- 触发器函数与触发器

-- 校验 media_paths.path 必须是 user_directories.path 的子路径
-- （= 目录本身 或 以 目录 + '/' 开头，避免 /media 误匹配 /media_evil）
CREATE OR REPLACE FUNCTION check_media_path_in_directory()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE dir_path VARCHAR;
BEGIN
    SELECT path INTO dir_path FROM user_directories WHERE id = NEW.directory_id;
    IF dir_path IS NULL THEN
        RAISE EXCEPTION 'directory_id % 不存在', NEW.directory_id;
    END IF;
    IF NEW.path != dir_path AND NEW.path NOT LIKE dir_path || '/%' THEN
        RAISE EXCEPTION '媒体路径 % 不在授权目录 % 内', NEW.path, dir_path;
    END IF;
    RETURN NEW;
END;
$$;

-- notes.updated_at 自动更新
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- openai_sessions.updated_at 自动更新
CREATE OR REPLACE FUNCTION update_openai_session_timestamp()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- openai_sessions.message_count 随消息增删自动调整
CREATE OR REPLACE FUNCTION update_message_count()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE openai_sessions
        SET message_count = message_count + 1
        WHERE id = NEW.session_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE openai_sessions
        SET message_count = message_count - 1
        WHERE id = OLD.session_id;
        RETURN OLD;
    END IF;
END;
$$;

-- ─── 触发器 ───────────────────────────────────────────────

DROP TRIGGER IF EXISTS trg_media_path_check ON media_paths;
CREATE TRIGGER trg_media_path_check
    BEFORE INSERT OR UPDATE OF path, directory_id ON media_paths
    FOR EACH ROW EXECUTE FUNCTION check_media_path_in_directory();

DROP TRIGGER IF EXISTS trigger_update_updated_at ON notes;
CREATE TRIGGER trigger_update_updated_at
    BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_openai_session_timestamp ON openai_sessions;
CREATE TRIGGER update_openai_session_timestamp
    BEFORE UPDATE ON openai_sessions
    FOR EACH ROW EXECUTE FUNCTION update_openai_session_timestamp();

DROP TRIGGER IF EXISTS trigger_update_message_count ON openai_messages;
CREATE TRIGGER trigger_update_message_count
    AFTER INSERT OR DELETE ON openai_messages
    FOR EACH ROW EXECUTE FUNCTION update_message_count();
