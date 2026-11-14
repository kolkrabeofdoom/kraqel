-- Foreign keys are disabled only by the migration runner outside this transaction.
-- Preserve rowids and all referencing tables; validate every foreign key before commit.
CREATE TABLE document_new (
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
    title TEXT NOT NULL CHECK (length(title) BETWEEN 0 AND 240 AND instr(title, char(0)) = 0),
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


INSERT INTO document_new (rowid, id, project_id, parent_id, kind, title, position, revision,
 content_json, plain_text, created_at, updated_at)
SELECT rowid, id, project_id, parent_id, kind, title, position, revision,
 content_json, plain_text, created_at, updated_at FROM document;
DROP TRIGGER document_fts_after_insert;
DROP TRIGGER document_fts_after_delete;
DROP TRIGGER document_fts_after_update;
DROP TRIGGER asset_link_document_after_delete;
DROP TABLE document;
ALTER TABLE document_new RENAME TO document;
CREATE INDEX document_project_parent_position_idx ON document(project_id, parent_id, position);
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

CREATE TRIGGER asset_link_document_after_delete
AFTER DELETE ON document
BEGIN
    DELETE FROM asset_link
    WHERE project_id = old.project_id AND target_type = 'document' AND target_id = old.id;
END;


INSERT INTO document_fts(document_fts) VALUES ('rebuild');
PRAGMA user_version = 6;
