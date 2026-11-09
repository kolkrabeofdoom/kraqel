CREATE TABLE media_asset (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind = 'image'),
    relative_path TEXT NOT NULL CHECK (relative_path LIKE 'assets/%'),
    thumbnail_path TEXT NOT NULL CHECK (thumbnail_path LIKE 'assets/thumbnails/%'),
    original_name TEXT NOT NULL CHECK (length(trim(original_name)) BETWEEN 1 AND 240),
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 240),
    mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp')),
    checksum TEXT NOT NULL CHECK (
        length(checksum) = 64 AND lower(checksum) = checksum
        AND checksum NOT GLOB '*[^0-9a-f]*'
    ),
    byte_count INTEGER NOT NULL CHECK (byte_count BETWEEN 1 AND 26214400),
    width INTEGER NOT NULL CHECK (width BETWEEN 1 AND 16384),
    height INTEGER NOT NULL CHECK (height BETWEEN 1 AND 16384),
    provenance TEXT NOT NULL CHECK (provenance = 'imported'),
    tags_json TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(tags_json) AND json_type(tags_json) = 'array'),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    UNIQUE (project_id, checksum),
    UNIQUE (project_id, relative_path),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX media_asset_project_title_idx
    ON media_asset(project_id, title COLLATE NOCASE, id);

CREATE TABLE asset_link (
    project_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    target_type TEXT NOT NULL CHECK (target_type IN ('document', 'entity', 'event')),
    target_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'reference' CHECK (length(trim(role)) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL,
    PRIMARY KEY (asset_id, target_type, target_id, role),
    FOREIGN KEY (project_id, asset_id)
        REFERENCES media_asset(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX asset_link_project_target_idx
    ON asset_link(project_id, target_type, target_id);

CREATE TRIGGER asset_link_document_after_delete
AFTER DELETE ON document
BEGIN
    DELETE FROM asset_link
    WHERE project_id = old.project_id AND target_type = 'document' AND target_id = old.id;
END;

CREATE TRIGGER asset_link_entity_after_delete
AFTER DELETE ON entity
BEGIN
    DELETE FROM asset_link
    WHERE project_id = old.project_id AND target_type = 'entity' AND target_id = old.id;
END;

CREATE TRIGGER asset_link_event_after_delete
AFTER DELETE ON event
BEGIN
    DELETE FROM asset_link
    WHERE project_id = old.project_id AND target_type = 'event' AND target_id = old.id;
END;
