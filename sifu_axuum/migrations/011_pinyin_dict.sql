CREATE TABLE IF NOT EXISTS pinyin_dict (
    pinyin TEXT PRIMARY KEY,
    words JSONB NOT NULL
);
