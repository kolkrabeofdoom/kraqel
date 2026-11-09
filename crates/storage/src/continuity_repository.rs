//! Lesende Projektion strukturierter Projektdaten für die Kontinuitätsregeln.

use std::{collections::HashSet, error::Error, fmt};

use kraqel_continuity::{
    DocumentFact, EntityFact, Finding, PovFact, ProjectFacts, TimelinePlacementFact, analyze,
};
use kraqel_domain::{EntityId, EntityKind, ProjectId};
use rusqlite::params;

use crate::{Database, StorageError};

/// Fehler beim Laden eines regelbasierten Prüfstands.
#[derive(Debug)]
pub enum ContinuityError {
    InvalidStoredData,
    Storage(StorageError),
}

impl fmt::Display for ContinuityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte Kontinuitätsdaten sind ungültig.")
            }
            Self::Storage(error) => error.fmt(formatter),
        }
    }
}

impl Error for ContinuityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::InvalidStoredData => None,
        }
    }
}

impl From<rusqlite::Error> for ContinuityError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

/// Rein lesender Zugang zu reproduzierbaren Kontinuitätsbefunden.
pub struct ContinuityRepository<'database> {
    database: &'database Database,
}

impl<'database> ContinuityRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }

    /// Lädt einen konsistenten Projektstand und führt die Offline-Regeln aus.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn gespeicherte IDs oder Arten ungültig sind oder SQLite scheitert.
    pub fn analyze(&self, project_id: ProjectId) -> Result<Vec<Finding>, ContinuityError> {
        Ok(analyze(&ProjectFacts {
            project_id,
            documents: self.documents(project_id)?,
            entities: self.entities(project_id)?,
            pov_assignments: self.pov_assignments(project_id)?,
            timeline_placements: self.timeline_placements(project_id)?,
        }))
    }

    fn documents(&self, project_id: ProjectId) -> Result<Vec<DocumentFact>, ContinuityError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, title, plain_text FROM document
             WHERE project_id = ?1 AND kind = 'scene' ORDER BY position, id",
        )?;
        let raw = statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        raw.into_iter()
            .map(|(id, title, plain_text)| {
                let id = parse_id(&id)?;
                Ok(DocumentFact {
                    id,
                    title,
                    plain_text,
                    linked_entity_ids: self.linked_entities(project_id, id)?,
                })
            })
            .collect()
    }

    fn linked_entities(
        &self,
        project_id: ProjectId,
        document_id: EntityId,
    ) -> Result<HashSet<EntityId>, ContinuityError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT entity_id FROM link WHERE project_id = ?1 AND document_id = ?2")?;
        statement
            .query_map(
                params![project_id.to_string(), document_id.to_string()],
                |row| row.get::<_, String>(0),
            )?
            .map(|row| parse_id(&row?))
            .collect()
    }

    fn entities(&self, project_id: ProjectId) -> Result<Vec<EntityFact>, ContinuityError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, kind, name FROM entity WHERE project_id = ?1 ORDER BY kind, name, id",
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .map(|row| {
                let (id, kind, name) = row?;
                Ok(EntityFact {
                    id: parse_id(&id)?,
                    kind: parse_kind(&kind)?,
                    name,
                })
            })
            .collect()
    }

    fn pov_assignments(&self, project_id: ProjectId) -> Result<Vec<PovFact>, ContinuityError> {
        let mut statement = self.database.connection().prepare(
            "SELECT p.document_id, d.title, e.id, e.name, e.kind
             FROM plot_card p
             JOIN document d ON d.project_id = p.project_id AND d.id = p.document_id
             JOIN entity e ON e.project_id = p.project_id AND e.id = p.pov_entity_id
             WHERE p.project_id = ?1 AND p.pov_entity_id IS NOT NULL
             ORDER BY p.position, p.document_id",
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?
            .map(|row| {
                let (document, title, entity, name, kind) = row?;
                Ok(PovFact {
                    document_id: parse_id(&document)?,
                    document_title: title,
                    entity_id: parse_id(&entity)?,
                    entity_name: name,
                    entity_kind: parse_kind(&kind)?,
                })
            })
            .collect()
    }

    fn timeline_placements(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<TimelinePlacementFact>, ContinuityError> {
        let mut statement = self.database.connection().prepare(
            "SELECT t.id, t.name, e.id, e.title, d.id, d.title
             FROM event_document_link l
             JOIN event e ON e.project_id = l.project_id AND e.id = l.event_id
             JOIN timeline t ON t.project_id = e.project_id AND t.id = e.timeline_id
             JOIN document d ON d.project_id = l.project_id AND d.id = l.document_id
             WHERE l.project_id = ?1
             ORDER BY t.position, e.position, d.position, e.id, d.id",
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?
            .map(|row| {
                let (timeline, timeline_name, event, event_title, document, document_title) = row?;
                Ok(TimelinePlacementFact {
                    timeline_id: parse_id(&timeline)?,
                    timeline_name,
                    event_id: parse_id(&event)?,
                    event_title,
                    document_id: parse_id(&document)?,
                    document_title,
                })
            })
            .collect()
    }
}

fn parse_id(value: &str) -> Result<EntityId, ContinuityError> {
    EntityId::parse(value).ok_or(ContinuityError::InvalidStoredData)
}

fn parse_kind(value: &str) -> Result<EntityKind, ContinuityError> {
    match value {
        "character" => Ok(EntityKind::Character),
        "location" => Ok(EntityKind::Location),
        "faction" => Ok(EntityKind::Faction),
        "item" => Ok(EntityKind::Item),
        _ => Err(ContinuityError::InvalidStoredData),
    }
}

#[cfg(test)]
mod tests {
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId, ProjectId};
    use rusqlite::params;

    use super::ContinuityRepository;
    use crate::Database;

    #[test]
    fn repository_reports_anchored_mentions_without_writing() {
        let database = Database::open_in_memory().expect("database should open");
        let project = ProjectId::new();
        let document = EntityId::new();
        let entity = EntityId::new();
        let timestamp = "2026-11-09T12:00:00Z";
        database.connection().execute(
            "INSERT INTO project (id, name, schema_version, created_at, updated_at) VALUES (?1, 'Test', ?2, ?3, ?3)",
            params![project.to_string(), CURRENT_SCHEMA_VERSION.0, timestamp],
        ).expect("project should be inserted");
        database.connection().execute(
            "INSERT INTO document (id, project_id, kind, title, position, plain_text, created_at, updated_at) VALUES (?1, ?2, 'scene', 'Szene', 0, 'Ada wartet.', ?3, ?3)",
            params![document.to_string(), project.to_string(), timestamp],
        ).expect("document should be inserted");
        database.connection().execute(
            "INSERT INTO entity (id, project_id, kind, name, created_at, updated_at) VALUES (?1, ?2, 'character', 'Ada', ?3, ?3)",
            params![entity.to_string(), project.to_string(), timestamp],
        ).expect("entity should be inserted");

        let findings = ContinuityRepository::new(&database)
            .analyze(project)
            .expect("analysis should succeed");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "story-bible.unlinked-mention");
        let links: i64 = database
            .connection()
            .query_row("SELECT count(*) FROM link", [], |row| row.get(0))
            .expect("link count should load");
        assert_eq!(links, 0);
    }
}
