CREATE TABLE storyline (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    color TEXT NOT NULL CHECK (length(color) = 7 AND substr(color, 1, 1) = '#'),
    position INTEGER NOT NULL CHECK (position >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX storyline_project_position_idx ON storyline(project_id, position, id);

CREATE TABLE plot_card (
    project_id TEXT NOT NULL,
    document_id TEXT PRIMARY KEY NOT NULL,
    storyline_id TEXT,
    pov_entity_id TEXT,
    status TEXT NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'draft', 'revised', 'done')),
    position INTEGER NOT NULL CHECK (position >= 0),
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id, document_id)
        REFERENCES document(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (storyline_id) REFERENCES storyline(id) ON DELETE SET NULL,
    FOREIGN KEY (pov_entity_id) REFERENCES entity(id) ON DELETE SET NULL
) STRICT;

CREATE INDEX plot_card_project_position_idx ON plot_card(project_id, position, document_id);
CREATE INDEX plot_card_storyline_idx ON plot_card(project_id, storyline_id);

CREATE TABLE timeline (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    mode TEXT NOT NULL CHECK (mode IN ('relative', 'absolute')),
    position INTEGER NOT NULL CHECK (position >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX timeline_project_position_idx ON timeline(project_id, position, id);

CREATE TABLE event (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36 AND lower(id) = id),
    project_id TEXT NOT NULL,
    timeline_id TEXT NOT NULL,
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 240),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 8000),
    time_value TEXT CHECK (time_value IS NULL OR length(trim(time_value)) BETWEEN 1 AND 80),
    position INTEGER NOT NULL CHECK (position >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, id),
    FOREIGN KEY (project_id, timeline_id)
        REFERENCES timeline(project_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX event_timeline_position_idx ON event(project_id, timeline_id, position, id);

CREATE TABLE event_document_link (
    project_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (event_id, document_id),
    FOREIGN KEY (project_id, event_id) REFERENCES event(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, document_id) REFERENCES document(project_id, id) ON DELETE CASCADE
) STRICT;

CREATE TABLE event_entity_link (
    project_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (event_id, entity_id),
    FOREIGN KEY (project_id, event_id) REFERENCES event(project_id, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, entity_id) REFERENCES entity(project_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX event_document_project_idx ON event_document_link(project_id, document_id);
CREATE INDEX event_entity_project_idx ON event_entity_link(project_id, entity_id);
