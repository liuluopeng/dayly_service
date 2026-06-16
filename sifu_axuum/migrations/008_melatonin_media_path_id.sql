ALTER TABLE melatonin_movies ADD COLUMN IF NOT EXISTS media_path_id UUID REFERENCES media_paths(id);
CREATE INDEX IF NOT EXISTS idx_melatonin_movies_media_path_id ON melatonin_movies(media_path_id);
