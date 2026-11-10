CREATE TABLE visual_profile (
    entity_id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    traits_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(traits_json) AND json_type(traits_json) = 'object'),
    canonical_asset_id TEXT,
    exclusions_json TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(exclusions_json) AND json_type(exclusions_json) = 'array'),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id, entity_id)
        REFERENCES entity(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (canonical_asset_id)
        REFERENCES media_asset(id) ON DELETE SET NULL
) STRICT;

CREATE TABLE style_preset (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    visual_rules_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(visual_rules_json) AND json_type(visual_rules_json) = 'object'),
    applicable_kinds_json TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(applicable_kinds_json) AND json_type(applicable_kinds_json) = 'array'),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX style_preset_project_name_idx
    ON style_preset(project_id, name COLLATE NOCASE);

CREATE TRIGGER visual_profile_asset_before_insert
BEFORE INSERT ON visual_profile
WHEN new.canonical_asset_id IS NOT NULL
     AND NOT EXISTS (
         SELECT 1 FROM media_asset
         WHERE id = new.canonical_asset_id AND project_id = new.project_id
     )
BEGIN
    SELECT RAISE(ABORT, 'canonical asset belongs to another project');
END;

CREATE TRIGGER visual_profile_asset_before_update
BEFORE UPDATE OF canonical_asset_id, project_id ON visual_profile
WHEN new.canonical_asset_id IS NOT NULL
     AND NOT EXISTS (
         SELECT 1 FROM media_asset
         WHERE id = new.canonical_asset_id AND project_id = new.project_id
     )
BEGIN
    SELECT RAISE(ABORT, 'canonical asset belongs to another project');
END;
