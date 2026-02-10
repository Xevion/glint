-- Fix parent_scene_id FK: default RESTRICT → SET NULL so deleting a parent
-- gracefully orphans derivatives instead of causing FK violations.
ALTER TABLE scenes DROP CONSTRAINT scenes_parent_scene_id_fkey;
ALTER TABLE scenes ADD CONSTRAINT scenes_parent_scene_id_fkey
    FOREIGN KEY (parent_scene_id) REFERENCES scenes(id) ON DELETE SET NULL;
