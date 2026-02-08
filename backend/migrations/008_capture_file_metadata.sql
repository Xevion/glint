ALTER TABLE captures
  ADD COLUMN file_size_bytes BIGINT,
  ADD COLUMN content_type TEXT;
