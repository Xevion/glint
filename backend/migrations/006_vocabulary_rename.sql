-- Rename screenshot columns to image (vocabulary compliance)
ALTER TABLE captures RENAME COLUMN screenshot_url TO image_url;
ALTER TABLE captures RENAME COLUMN screenshot_path TO image_path;
