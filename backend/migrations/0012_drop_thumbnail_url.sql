-- Drop thumbnail_url column from captures.
-- Thumbnails are now derived at the edge via Cloudflare cdn-cgi image transforms on image_url.
ALTER TABLE captures DROP COLUMN IF EXISTS thumbnail_url;
