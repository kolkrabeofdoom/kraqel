CREATE UNIQUE INDEX visual_profile_project_entity_idx
    ON visual_profile(project_id, entity_id);

CREATE TABLE image_prompt_draft (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    source_document_id TEXT NOT NULL,
    source_document_revision INTEGER NOT NULL CHECK (source_document_revision >= 0),
    style_preset_id TEXT,
    prompt_text TEXT NOT NULL CHECK (length(trim(prompt_text)) > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, source_document_id)
        REFERENCES document(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (style_preset_id)
        REFERENCES style_preset(id) ON DELETE SET NULL
) STRICT;

CREATE INDEX image_prompt_draft_project_idx
    ON image_prompt_draft(project_id, updated_at, id);

CREATE TABLE image_prompt_draft_profile (
    draft_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (draft_id, entity_id),
    UNIQUE (draft_id, position),
    FOREIGN KEY (project_id, draft_id)
        REFERENCES image_prompt_draft(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, entity_id)
        REFERENCES visual_profile(project_id, entity_id) ON DELETE CASCADE
) STRICT;

CREATE TRIGGER image_prompt_profile_after_delete
AFTER DELETE ON image_prompt_draft_profile
WHEN NOT EXISTS (
    SELECT 1 FROM image_prompt_draft_profile WHERE draft_id = old.draft_id
)
BEGIN
    DELETE FROM image_prompt_draft WHERE id = old.draft_id;
END;

CREATE TRIGGER image_prompt_style_before_insert
BEFORE INSERT ON image_prompt_draft
WHEN new.style_preset_id IS NOT NULL
     AND NOT EXISTS (
         SELECT 1 FROM style_preset
         WHERE id = new.style_preset_id AND project_id = new.project_id
     )
BEGIN
    SELECT RAISE(ABORT, 'style preset belongs to another project');
END;

CREATE TRIGGER image_prompt_style_before_update
BEFORE UPDATE OF style_preset_id, project_id ON image_prompt_draft
WHEN new.style_preset_id IS NOT NULL
     AND NOT EXISTS (
         SELECT 1 FROM style_preset
         WHERE id = new.style_preset_id AND project_id = new.project_id
     )
BEGIN
    SELECT RAISE(ABORT, 'style preset belongs to another project');
END;
