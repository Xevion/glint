-- Prevent id/slug collisions on tables where both columns exist.
-- The ID-or-slug resolver uses a length heuristic, so ensuring id != slug
-- on the same row avoids ambiguous lookups.

-- Helper: generate a 10-char random ID from the unambiguous alphabet.
CREATE OR REPLACE FUNCTION _generate_nanoid() RETURNS text AS $$
DECLARE
    alphabet text := 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789';
    result text := '';
    i int;
BEGIN
    FOR i IN 1..10 LOOP
        result := result || substr(alphabet, floor(random() * 56 + 1)::int, 1);
    END LOOP;
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Temporarily drop FK constraints that reference shaders(id), worlds(id), scenes(id)
-- so we can update IDs without ordering issues.
ALTER TABLE shader_versions DROP CONSTRAINT shader_versions_shader_id_fkey;
ALTER TABLE shader_authors DROP CONSTRAINT shader_authors_shader_id_fkey;
ALTER TABLE shader_categories DROP CONSTRAINT shader_categories_shader_id_fkey;
ALTER TABLE shader_features DROP CONSTRAINT shader_features_shader_id_fkey;

ALTER TABLE world_versions DROP CONSTRAINT world_versions_world_id_fkey;
ALTER TABLE scenes DROP CONSTRAINT scenes_world_id_fkey;

ALTER TABLE scene_versions DROP CONSTRAINT scene_versions_scene_id_fkey;
ALTER TABLE captures DROP CONSTRAINT captures_scene_id_fkey;
ALTER TABLE capture_run_items DROP CONSTRAINT capture_run_items_scene_id_fkey;
ALTER TABLE scene_tags DROP CONSTRAINT scene_tags_scene_id_fkey;
ALTER TABLE scenes DROP CONSTRAINT IF EXISTS scenes_parent_scene_id_fkey;

-- Reassign IDs for rows where id = slug.
DO $$
DECLARE
    rec RECORD;
    new_id text;
BEGIN
    FOR rec IN SELECT id FROM shaders WHERE id = slug LOOP
        new_id := _generate_nanoid();
        UPDATE shader_versions SET shader_id = new_id WHERE shader_id = rec.id;
        UPDATE shader_authors SET shader_id = new_id WHERE shader_id = rec.id;
        UPDATE shader_categories SET shader_id = new_id WHERE shader_id = rec.id;
        UPDATE shader_features SET shader_id = new_id WHERE shader_id = rec.id;
        UPDATE shaders SET id = new_id WHERE id = rec.id;
    END LOOP;

    FOR rec IN SELECT id FROM worlds WHERE id = slug LOOP
        new_id := _generate_nanoid();
        UPDATE world_versions SET world_id = new_id WHERE world_id = rec.id;
        UPDATE scenes SET world_id = new_id WHERE world_id = rec.id;
        UPDATE pending_uploads SET world_id = new_id WHERE world_id = rec.id;
        UPDATE worlds SET id = new_id WHERE id = rec.id;
    END LOOP;

    FOR rec IN SELECT id FROM scenes WHERE id = slug LOOP
        new_id := _generate_nanoid();
        UPDATE scene_versions SET scene_id = new_id WHERE scene_id = rec.id;
        UPDATE captures SET scene_id = new_id WHERE scene_id = rec.id;
        UPDATE capture_run_items SET scene_id = new_id WHERE scene_id = rec.id;
        UPDATE scene_tags SET scene_id = new_id WHERE scene_id = rec.id;
        UPDATE scenes SET parent_scene_id = new_id WHERE parent_scene_id = rec.id;
        UPDATE scenes SET id = new_id WHERE id = rec.id;
    END LOOP;
END $$;

-- Restore FK constraints.
ALTER TABLE shader_versions ADD CONSTRAINT shader_versions_shader_id_fkey
    FOREIGN KEY (shader_id) REFERENCES shaders(id) ON DELETE CASCADE;
ALTER TABLE shader_authors ADD CONSTRAINT shader_authors_shader_id_fkey
    FOREIGN KEY (shader_id) REFERENCES shaders(id) ON DELETE CASCADE;
ALTER TABLE shader_categories ADD CONSTRAINT shader_categories_shader_id_fkey
    FOREIGN KEY (shader_id) REFERENCES shaders(id) ON DELETE CASCADE;
ALTER TABLE shader_features ADD CONSTRAINT shader_features_shader_id_fkey
    FOREIGN KEY (shader_id) REFERENCES shaders(id) ON DELETE CASCADE;

ALTER TABLE world_versions ADD CONSTRAINT world_versions_world_id_fkey
    FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE;
ALTER TABLE scenes ADD CONSTRAINT scenes_world_id_fkey
    FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE;

ALTER TABLE scene_versions ADD CONSTRAINT scene_versions_scene_id_fkey
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE;
ALTER TABLE captures ADD CONSTRAINT captures_scene_id_fkey
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE;
ALTER TABLE capture_run_items ADD CONSTRAINT capture_run_items_scene_id_fkey
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE;
ALTER TABLE scene_tags ADD CONSTRAINT scene_tags_scene_id_fkey
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE;
ALTER TABLE scenes ADD CONSTRAINT scenes_parent_scene_id_fkey
    FOREIGN KEY (parent_scene_id) REFERENCES scenes(id) ON DELETE SET NULL;

DROP FUNCTION _generate_nanoid();

-- Add CHECK constraints.
ALTER TABLE shaders ADD CONSTRAINT chk_shaders_id_ne_slug CHECK (id <> slug);
ALTER TABLE worlds ADD CONSTRAINT chk_worlds_id_ne_slug CHECK (id <> slug);
ALTER TABLE scenes ADD CONSTRAINT chk_scenes_id_ne_slug CHECK (id <> slug);
