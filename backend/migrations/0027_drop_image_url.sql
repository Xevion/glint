-- Drop the image_url column from captures and backgrounds.
-- image_path (the S3 object key) is now the sole source of truth;
-- public URLs are derived at runtime from config.
--
-- Views that use `c.*` from captures transitively depend on image_url,
-- so we must drop them first. They are recreated on app startup via
-- db::apply_views() from views.sql.

DROP VIEW IF EXISTS capture_details CASCADE;
DROP VIEW IF EXISTS capture_contexts CASCADE;
DROP VIEW IF EXISTS captures_with_freshness CASCADE;

ALTER TABLE captures DROP COLUMN IF EXISTS image_url;
ALTER TABLE backgrounds DROP COLUMN IF EXISTS image_url;
