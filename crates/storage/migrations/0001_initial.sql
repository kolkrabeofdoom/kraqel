CREATE TABLE project (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (
            length(id) = 36
            AND substr(id, 9, 1) = '-'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 24, 1) = '-'
            AND lower(id) = id
            AND id NOT GLOB '*[^0-9a-f-]*'
        ),
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    settings_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(settings_json)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE document (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (
            length(id) = 36
            AND substr(id, 9, 1) = '-'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 24, 1) = '-'
            AND lower(id) = id
            AND id NOT GLOB '*[^0-9a-f-]*'
        ),
    project_id TEXT NOT NULL,
    parent_id TEXT,
    kind TEXT NOT NULL CHECK (kind IN ('chapter', 'scene')),
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 240),
    position INTEGER NOT NULL CHECK (position >= 0),
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0),
    content_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(content_json)),
    plain_text TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, parent_id)
        REFERENCES document(project_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX document_project_parent_position_idx
    ON document(project_id, parent_id, position);

CREATE TABLE entity (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (
            length(id) = 36
            AND substr(id, 9, 1) = '-'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 24, 1) = '-'
            AND lower(id) = id
            AND id NOT GLOB '*[^0-9a-f-]*'
        ),
    project_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (length(trim(kind)) > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 240),
    properties_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(properties_json)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX entity_project_kind_name_idx
    ON entity(project_id, kind, name);

CREATE TABLE relation (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (
            length(id) = 36
            AND substr(id, 9, 1) = '-'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 24, 1) = '-'
            AND lower(id) = id
            AND id NOT GLOB '*[^0-9a-f-]*'
        ),
    project_id TEXT NOT NULL,
    source_entity_id TEXT NOT NULL,
    target_entity_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (length(trim(kind)) > 0),
    properties_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(properties_json)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, source_entity_id)
        REFERENCES entity(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, target_entity_id)
        REFERENCES entity(project_id, id) ON DELETE CASCADE,
    CHECK (source_entity_id <> target_entity_id)
) STRICT;

CREATE INDEX relation_project_source_idx
    ON relation(project_id, source_entity_id);

CREATE INDEX relation_project_target_idx
    ON relation(project_id, target_entity_id);

CREATE TABLE link (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (
            length(id) = 36
            AND substr(id, 9, 1) = '-'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 24, 1) = '-'
            AND lower(id) = id
            AND id NOT GLOB '*[^0-9a-f-]*'
        ),
    project_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'reference' CHECK (length(trim(role)) > 0),
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, document_id)
        REFERENCES document(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, entity_id)
        REFERENCES entity(project_id, id) ON DELETE CASCADE,
    UNIQUE (project_id, document_id, entity_id, role)
) STRICT;

CREATE INDEX link_project_entity_idx
    ON link(project_id, entity_id);

CREATE VIRTUAL TABLE document_fts USING fts5(
    title,
    plain_text,
    content = 'document',
    content_rowid = 'rowid',
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER document_fts_after_insert
AFTER INSERT ON document
BEGIN
    INSERT INTO document_fts(rowid, title, plain_text)
    VALUES (new.rowid, new.title, new.plain_text);
END;

CREATE TRIGGER document_fts_after_delete
AFTER DELETE ON document
BEGIN
    INSERT INTO document_fts(document_fts, rowid, title, plain_text)
    VALUES ('delete', old.rowid, old.title, old.plain_text);
END;

CREATE TRIGGER document_fts_after_update
AFTER UPDATE OF title, plain_text ON document
BEGIN
    INSERT INTO document_fts(document_fts, rowid, title, plain_text)
    VALUES ('delete', old.rowid, old.title, old.plain_text);
    INSERT INTO document_fts(rowid, title, plain_text)
    VALUES (new.rowid, new.title, new.plain_text);
END;
