//! Transaktionales Repository für Story-Bibel-Einträge, Beziehungen und Rückverweise.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{
    DocumentKind, EntityId, EntityKind, ProjectId, StoryBacklink, StoryEntity, StoryEntityDetails,
    StoryRelation,
};
use rusqlite::{OptionalExtension, Row, Transaction, params};

use crate::{Database, StorageError};

const MAX_NAME_CHARS: usize = 240;
const MAX_PROPERTY_COUNT: usize = 50;
const MAX_PROPERTY_KEY_CHARS: usize = 80;
const MAX_PROPERTY_VALUE_CHARS: usize = 2_000;
const MAX_PROPERTIES_BYTES: usize = 64 * 1024;
const MAX_RELATION_KIND_CHARS: usize = 80;

/// Fachlich unterscheidbare Fehler des Story-Bibel-Repositorys.
#[derive(Debug)]
pub enum StoryBibleError {
    /// Eingaben verletzen den öffentlichen Vertrag.
    InvalidInput,
    /// Projekt, Eintrag, Beziehung oder Dokument wurde nicht gefunden.
    NotFound,
    /// Gespeicherte Daten verletzen den erwarteten internen Vertrag.
    InvalidStoredData,
    /// Beide Einträge besitzen Daten, die nicht verlustfrei automatisch vereinigt werden können.
    MergeConflict,
    /// SQLite- oder Verbindungsfehler der Speicherschicht.
    Storage(StorageError),
    /// Die Systemzeit liegt vor der Unix-Epoche.
    InvalidSystemTime,
}

impl fmt::Display for StoryBibleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die Story-Bibel-Daten sind ungültig."),
            Self::NotFound => formatter.write_str("Der Story-Bibel-Eintrag wurde nicht gefunden."),
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte Story-Bibel-Daten sind ungültig.")
            }
            Self::MergeConflict => formatter.write_str(
                "Die Story-Bibel-Einträge können nicht verlustfrei zusammengeführt werden.",
            ),
            Self::Storage(error) => error.fmt(formatter),
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
        }
    }
}

impl Error for StoryBibleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::InvalidInput
            | Self::NotFound
            | Self::InvalidStoredData
            | Self::MergeConflict
            | Self::InvalidSystemTime => None,
        }
    }
}

impl From<rusqlite::Error> for StoryBibleError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

impl From<StorageError> for StoryBibleError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Repository über einer bereits verwalteten Projektdatenbank.
pub struct StoryBibleRepository<'database> {
    database: &'database mut Database,
}

impl<'database> StoryBibleRepository<'database> {
    /// Bindet das Repository an eine Projektdatenbank.
    pub const fn new(database: &'database mut Database) -> Self {
        Self { database }
    }

    /// Listet Einträge stabil sortiert und filtert optional nach Art und Suchtext.
    ///
    /// Die Filterung erfolgt nach dem Laden in Rust, damit Unicode-Kleinschreibung nicht von der
    /// ASCII-begrenzten SQLite-Funktion `lower` abhängt.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei zu langem Suchtext, ungültigen gespeicherten Daten oder SQLite-
    /// Problemen.
    pub fn list(
        &self,
        project_id: ProjectId,
        query: Option<&str>,
        kind: Option<EntityKind>,
    ) -> Result<Vec<StoryEntity>, StoryBibleError> {
        if query.is_some_and(|value| value.chars().count() > 240) {
            return Err(StoryBibleError::InvalidInput);
        }
        let mut statement = self.database.connection().prepare(
            "SELECT id, project_id, kind, name, properties_json FROM entity
             WHERE project_id = ?1 ORDER BY kind, name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([project_id.to_string()], raw_entity)?;
        let needle = query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_lowercase);
        rows.map(|row| row.map_err(StoryBibleError::from).and_then(parse_entity))
            .filter_map(|result| match result {
                Ok(entity) if kind.is_some_and(|expected| entity.kind != expected) => None,
                Ok(entity)
                    if needle
                        .as_ref()
                        .is_some_and(|value| !entity_matches(&entity, value)) =>
                {
                    None
                }
                other => Some(other),
            })
            .collect()
    }

    /// Erstellt einen validierten Story-Bibel-Eintrag.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei ungültigem Namen, ungültigen Eigenschaften, fehlendem Projekt oder
    /// fehlgeschlagener Transaktion.
    pub fn create_entity(
        &mut self,
        project_id: ProjectId,
        kind: EntityKind,
        name: &str,
        properties: &BTreeMap<String, String>,
    ) -> Result<StoryEntity, StoryBibleError> {
        validate_name(name)?;
        let properties_json = validate_and_serialize_properties(properties)?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        require_project(&transaction, project_id)?;
        transaction.execute(
            "INSERT INTO entity (id, project_id, kind, name, properties_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id.to_string(), project_id.to_string(), kind_name(kind), name.trim(), properties_json, timestamp],
        )?;
        transaction.commit()?;
        Ok(StoryEntity {
            id,
            project_id,
            kind,
            name: name.trim().to_owned(),
            properties: properties.clone(),
        })
    }

    /// Aktualisiert Art, Name und freie Eigenschaften eines Eintrags.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei ungültigen Eingaben, fehlendem Eintrag oder SQLite-Problemen.
    pub fn update_entity(
        &mut self,
        project_id: ProjectId,
        entity_id: EntityId,
        kind: EntityKind,
        name: &str,
        properties: &BTreeMap<String, String>,
    ) -> Result<StoryEntity, StoryBibleError> {
        validate_name(name)?;
        let properties_json = validate_and_serialize_properties(properties)?;
        let changed = self.database.connection().execute(
            "UPDATE entity SET kind = ?3, name = ?4, properties_json = ?5, updated_at = ?6
             WHERE project_id = ?1 AND id = ?2",
            params![
                project_id.to_string(),
                entity_id.to_string(),
                kind_name(kind),
                name.trim(),
                properties_json,
                current_timestamp()?
            ],
        )?;
        if changed == 0 {
            return Err(StoryBibleError::NotFound);
        }
        Ok(StoryEntity {
            id: entity_id,
            project_id,
            kind,
            name: name.trim().to_owned(),
            properties: properties.clone(),
        })
    }

    /// Löscht einen Eintrag; Beziehungen und Rückverweise folgen den Schema-Löschregeln.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei fehlendem Eintrag oder SQLite-Problemen.
    pub fn delete_entity(
        &mut self,
        project_id: ProjectId,
        entity_id: EntityId,
    ) -> Result<(), StoryBibleError> {
        let changed = self.database.connection().execute(
            "DELETE FROM entity WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), entity_id.to_string()],
        )?;
        if changed == 0 {
            Err(StoryBibleError::NotFound)
        } else {
            Ok(())
        }
    }

    /// Führt einen doppelten Quell-Eintrag atomar in einen bestehenden Ziel-Eintrag derselben Art.
    ///
    /// Der Zielname und vorhandene Ziel-Eigenschaften bleiben maßgeblich. Fehlende Eigenschaften
    /// sowie sämtliche fachlichen Referenzen werden von der Quelle übernommen. Haben beide
    /// Einträge ein visuelles Profil, wird ohne Änderung abgebrochen.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei identischen oder artfremden Einträgen, fehlenden Datensätzen,
    /// unvereinbaren visuellen Profilen, ungültigen kombinierten Eigenschaften oder SQLite-
    /// Problemen.
    pub fn merge_entities(
        &mut self,
        project_id: ProjectId,
        source_entity_id: EntityId,
        target_entity_id: EntityId,
    ) -> Result<StoryEntityDetails, StoryBibleError> {
        if source_entity_id == target_entity_id {
            return Err(StoryBibleError::InvalidInput);
        }

        let project = project_id.to_string();
        let source_id = source_entity_id.to_string();
        let target_id = target_entity_id.to_string();
        let transaction = self.database.transaction()?;
        let source = transaction
            .query_row(
                "SELECT id, project_id, kind, name, properties_json FROM entity
                 WHERE project_id = ?1 AND id = ?2",
                params![project, source_id],
                raw_entity,
            )
            .optional()?
            .map(parse_entity)
            .transpose()?
            .ok_or(StoryBibleError::NotFound)?;
        let target = transaction
            .query_row(
                "SELECT id, project_id, kind, name, properties_json FROM entity
                 WHERE project_id = ?1 AND id = ?2",
                params![project, target_id],
                raw_entity,
            )
            .optional()?
            .map(parse_entity)
            .transpose()?
            .ok_or(StoryBibleError::NotFound)?;
        if source.kind != target.kind {
            return Err(StoryBibleError::InvalidInput);
        }

        let source_has_profile = has_visual_profile(&transaction, project_id, source_entity_id)?;
        let target_has_profile = has_visual_profile(&transaction, project_id, target_entity_id)?;
        if source_has_profile && target_has_profile {
            return Err(StoryBibleError::MergeConflict);
        }

        let mut merged_properties = target.properties.clone();
        for (key, value) in source.properties {
            merged_properties.entry(key).or_insert(value);
        }
        let merged_properties_json = validate_and_serialize_properties(&merged_properties)?;
        let timestamp = current_timestamp()?;

        if source_has_profile {
            move_visual_profile(&transaction, &project, &source_id, &target_id, &timestamp)?;
        }
        retarget_unique_links(&transaction, &project, &source_id, &target_id)?;
        retarget_relations_and_pov(&transaction, &project, &source_id, &target_id, &timestamp)?;
        transaction.execute(
            "UPDATE entity SET properties_json = ?3, updated_at = ?4
             WHERE project_id = ?1 AND id = ?2",
            params![project, target_id, merged_properties_json, timestamp],
        )?;
        transaction.execute(
            "DELETE FROM entity WHERE project_id = ?1 AND id = ?2",
            params![project, source_id],
        )?;
        transaction.commit()?;
        self.load(project_id, target_entity_id)
    }

    /// Lädt einen Eintrag einschließlich gerichteter Beziehungen und Dokument-Rückverweise.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei fehlendem Eintrag, ungültigen gespeicherten Daten oder SQLite-
    /// Problemen.
    pub fn load(
        &self,
        project_id: ProjectId,
        entity_id: EntityId,
    ) -> Result<StoryEntityDetails, StoryBibleError> {
        let entity = self.database.connection().query_row(
            "SELECT id, project_id, kind, name, properties_json FROM entity WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), entity_id.to_string()], raw_entity,
        ).optional()?.map(parse_entity).transpose()?.ok_or(StoryBibleError::NotFound)?;
        let outgoing_relations = self.relations(project_id, entity_id, true)?;
        let incoming_relations = self.relations(project_id, entity_id, false)?;
        let backlinks = self.backlinks(project_id, entity_id)?;
        Ok(StoryEntityDetails {
            entity,
            outgoing_relations,
            incoming_relations,
            backlinks,
        })
    }

    /// Erstellt eine gerichtete Beziehung zwischen zwei vorhandenen Einträgen desselben Projekts.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei ungültiger oder selbstbezüglicher Relation, fehlenden Einträgen
    /// oder SQLite-Problemen.
    pub fn create_relation(
        &mut self,
        project_id: ProjectId,
        source_entity_id: EntityId,
        target_entity_id: EntityId,
        kind: &str,
        properties: &BTreeMap<String, String>,
    ) -> Result<StoryRelation, StoryBibleError> {
        if source_entity_id == target_entity_id {
            return Err(StoryBibleError::InvalidInput);
        }
        validate_short_text(kind, MAX_RELATION_KIND_CHARS)?;
        let properties_json = validate_and_serialize_properties(properties)?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        require_entity(&transaction, project_id, source_entity_id)?;
        require_entity(&transaction, project_id, target_entity_id)?;
        transaction.execute(
            "INSERT INTO relation (id, project_id, source_entity_id, target_entity_id, kind, properties_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![id.to_string(), project_id.to_string(), source_entity_id.to_string(), target_entity_id.to_string(), kind.trim(), properties_json, timestamp],
        )?;
        transaction.commit()?;
        Ok(StoryRelation {
            id,
            project_id,
            source_entity_id,
            target_entity_id,
            kind: kind.trim().to_owned(),
            properties: properties.clone(),
        })
    }

    /// Löscht eine einzelne Beziehung desselben Projekts.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei fehlender Beziehung oder SQLite-Problemen.
    pub fn delete_relation(
        &mut self,
        project_id: ProjectId,
        relation_id: EntityId,
    ) -> Result<(), StoryBibleError> {
        let changed = self.database.connection().execute(
            "DELETE FROM relation WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), relation_id.to_string()],
        )?;
        if changed == 0 {
            Err(StoryBibleError::NotFound)
        } else {
            Ok(())
        }
    }

    /// Verknüpft ein vorhandenes Dokument idempotent mit einem Eintrag.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei fehlendem Dokument oder Eintrag sowie bei SQLite-Problemen.
    pub fn link_document(
        &mut self,
        project_id: ProjectId,
        document_id: EntityId,
        entity_id: EntityId,
    ) -> Result<(), StoryBibleError> {
        let transaction = self.database.transaction()?;
        require_entity(&transaction, project_id, entity_id)?;
        let document_exists = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM document WHERE project_id = ?1 AND id = ?2)",
            params![project_id.to_string(), document_id.to_string()],
            |row| row.get::<_, bool>(0),
        )?;
        if !document_exists {
            return Err(StoryBibleError::NotFound);
        }
        transaction.execute(
            "INSERT OR IGNORE INTO link (id, project_id, document_id, entity_id, role, created_at)
             VALUES (?1, ?2, ?3, ?4, 'reference', ?5)",
            params![
                EntityId::new().to_string(),
                project_id.to_string(),
                document_id.to_string(),
                entity_id.to_string(),
                current_timestamp()?
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn relations(
        &self,
        project_id: ProjectId,
        entity_id: EntityId,
        outgoing: bool,
    ) -> Result<Vec<StoryRelation>, StoryBibleError> {
        let column = if outgoing {
            "source_entity_id"
        } else {
            "target_entity_id"
        };
        let sql = format!(
            "SELECT id, project_id, source_entity_id, target_entity_id, kind, properties_json
             FROM relation WHERE project_id = ?1 AND {column} = ?2 ORDER BY kind COLLATE NOCASE, id"
        );
        let mut statement = self.database.connection().prepare(&sql)?;
        let rows = statement.query_map(
            params![project_id.to_string(), entity_id.to_string()],
            raw_relation,
        )?;
        rows.map(|row| row.map_err(StoryBibleError::from).and_then(parse_relation))
            .collect()
    }

    fn backlinks(
        &self,
        project_id: ProjectId,
        entity_id: EntityId,
    ) -> Result<Vec<StoryBacklink>, StoryBibleError> {
        let mut statement = self.database.connection().prepare(
            "SELECT d.id, d.title, d.kind, l.role FROM link l
             JOIN document d ON d.project_id = l.project_id AND d.id = l.document_id
             WHERE l.project_id = ?1 AND l.entity_id = ?2
             ORDER BY d.title COLLATE NOCASE, d.id, l.role",
        )?;
        let rows = statement.query_map(
            params![project_id.to_string(), entity_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (id, title, kind, role) = row?;
            Ok(StoryBacklink {
                document_id: EntityId::parse(&id).ok_or(StoryBibleError::InvalidStoredData)?,
                document_title: title,
                document_kind: parse_document_kind(&kind)?,
                role,
            })
        })
        .collect()
    }
}

type RawEntity = (String, String, String, String, String);
type RawRelation = (String, String, String, String, String, String);

fn raw_entity(row: &Row<'_>) -> rusqlite::Result<RawEntity> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn raw_relation(row: &Row<'_>) -> rusqlite::Result<RawRelation> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
    ))
}

fn parse_entity(raw: RawEntity) -> Result<StoryEntity, StoryBibleError> {
    let entity = StoryEntity {
        id: EntityId::parse(&raw.0).ok_or(StoryBibleError::InvalidStoredData)?,
        project_id: ProjectId::parse(&raw.1).ok_or(StoryBibleError::InvalidStoredData)?,
        kind: parse_entity_kind(&raw.2)?,
        name: raw.3,
        properties: parse_stored_properties(&raw.4)?,
    };
    validate_name(&entity.name).map_err(|_| StoryBibleError::InvalidStoredData)?;
    Ok(entity)
}

fn parse_relation(raw: RawRelation) -> Result<StoryRelation, StoryBibleError> {
    let relation = StoryRelation {
        id: EntityId::parse(&raw.0).ok_or(StoryBibleError::InvalidStoredData)?,
        project_id: ProjectId::parse(&raw.1).ok_or(StoryBibleError::InvalidStoredData)?,
        source_entity_id: EntityId::parse(&raw.2).ok_or(StoryBibleError::InvalidStoredData)?,
        target_entity_id: EntityId::parse(&raw.3).ok_or(StoryBibleError::InvalidStoredData)?,
        kind: raw.4,
        properties: parse_stored_properties(&raw.5)?,
    };
    if relation.source_entity_id == relation.target_entity_id {
        return Err(StoryBibleError::InvalidStoredData);
    }
    validate_short_text(&relation.kind, MAX_RELATION_KIND_CHARS)
        .map_err(|_| StoryBibleError::InvalidStoredData)?;
    Ok(relation)
}

fn parse_stored_properties(value: &str) -> Result<BTreeMap<String, String>, StoryBibleError> {
    let properties = serde_json::from_str(value).map_err(|_| StoryBibleError::InvalidStoredData)?;
    validate_and_serialize_properties(&properties)
        .map_err(|_| StoryBibleError::InvalidStoredData)?;
    Ok(properties)
}

fn entity_matches(entity: &StoryEntity, needle: &str) -> bool {
    entity.name.to_lowercase().contains(needle)
        || entity.properties.iter().any(|(key, value)| {
            key.to_lowercase().contains(needle) || value.to_lowercase().contains(needle)
        })
}

fn validate_name(name: &str) -> Result<(), StoryBibleError> {
    validate_short_text(name, MAX_NAME_CHARS)
}

fn validate_short_text(value: &str, maximum: usize) -> Result<(), StoryBibleError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > maximum
        || trimmed.chars().any(char::is_control)
    {
        Err(StoryBibleError::InvalidInput)
    } else {
        Ok(())
    }
}

fn validate_and_serialize_properties(
    properties: &BTreeMap<String, String>,
) -> Result<String, StoryBibleError> {
    if properties.len() > MAX_PROPERTY_COUNT {
        return Err(StoryBibleError::InvalidInput);
    }
    for (key, value) in properties {
        validate_short_text(key, MAX_PROPERTY_KEY_CHARS)?;
        if value.chars().count() > MAX_PROPERTY_VALUE_CHARS
            || value.chars().any(|character| character == '\0')
        {
            return Err(StoryBibleError::InvalidInput);
        }
    }
    let serialized =
        serde_json::to_string(properties).map_err(|_| StoryBibleError::InvalidInput)?;
    if serialized.len() > MAX_PROPERTIES_BYTES {
        Err(StoryBibleError::InvalidInput)
    } else {
        Ok(serialized)
    }
}

fn require_project(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
) -> Result<(), StoryBibleError> {
    let exists = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1)",
        [project_id.to_string()],
        |row| row.get::<_, bool>(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(StoryBibleError::NotFound)
    }
}

fn require_entity(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    entity_id: EntityId,
) -> Result<(), StoryBibleError> {
    let exists = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM entity WHERE project_id = ?1 AND id = ?2)",
        params![project_id.to_string(), entity_id.to_string()],
        |row| row.get::<_, bool>(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(StoryBibleError::NotFound)
    }
}

fn has_visual_profile(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    entity_id: EntityId,
) -> Result<bool, StoryBibleError> {
    transaction
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM visual_profile WHERE project_id = ?1 AND entity_id = ?2
             )",
            params![project_id.to_string(), entity_id.to_string()],
            |row| row.get(0),
        )
        .map_err(StoryBibleError::from)
}

fn move_visual_profile(
    transaction: &Transaction<'_>,
    project: &str,
    source_id: &str,
    target_id: &str,
    timestamp: &str,
) -> Result<(), StoryBibleError> {
    transaction.execute(
        "INSERT INTO visual_profile
            (entity_id, project_id, traits_json, canonical_asset_id, exclusions_json, created_at, updated_at)
         SELECT ?3, project_id, traits_json, canonical_asset_id, exclusions_json, created_at, ?4
         FROM visual_profile WHERE project_id = ?1 AND entity_id = ?2",
        params![project, source_id, target_id, timestamp],
    )?;
    transaction.execute(
        "UPDATE image_prompt_draft_profile SET entity_id = ?3
         WHERE project_id = ?1 AND entity_id = ?2",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "DELETE FROM visual_profile WHERE project_id = ?1 AND entity_id = ?2",
        params![project, source_id],
    )?;
    Ok(())
}

fn retarget_unique_links(
    transaction: &Transaction<'_>,
    project: &str,
    source_id: &str,
    target_id: &str,
) -> Result<(), StoryBibleError> {
    transaction.execute(
        "DELETE FROM link AS source_link
         WHERE project_id = ?1 AND entity_id = ?2 AND EXISTS (
             SELECT 1 FROM link AS target_link
             WHERE target_link.project_id = source_link.project_id
               AND target_link.document_id = source_link.document_id
               AND target_link.entity_id = ?3
               AND target_link.role = source_link.role
         )",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "UPDATE link SET entity_id = ?3 WHERE project_id = ?1 AND entity_id = ?2",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "DELETE FROM event_entity_link AS source_link
         WHERE project_id = ?1 AND entity_id = ?2 AND EXISTS (
             SELECT 1 FROM event_entity_link AS target_link
             WHERE target_link.project_id = source_link.project_id
               AND target_link.event_id = source_link.event_id
               AND target_link.entity_id = ?3
         )",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "UPDATE event_entity_link SET entity_id = ?3
         WHERE project_id = ?1 AND entity_id = ?2",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "DELETE FROM asset_link AS source_link
         WHERE project_id = ?1 AND target_type = 'entity' AND target_id = ?2 AND EXISTS (
             SELECT 1 FROM asset_link AS target_link
             WHERE target_link.project_id = source_link.project_id
               AND target_link.asset_id = source_link.asset_id
               AND target_link.target_type = 'entity'
               AND target_link.target_id = ?3
               AND target_link.role = source_link.role
         )",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "UPDATE asset_link SET target_id = ?3
         WHERE project_id = ?1 AND target_type = 'entity' AND target_id = ?2",
        params![project, source_id, target_id],
    )?;
    Ok(())
}

fn retarget_relations_and_pov(
    transaction: &Transaction<'_>,
    project: &str,
    source_id: &str,
    target_id: &str,
    timestamp: &str,
) -> Result<(), StoryBibleError> {
    transaction.execute(
        "DELETE FROM relation WHERE project_id = ?1 AND (
            (source_entity_id = ?2 AND target_entity_id = ?3)
            OR (source_entity_id = ?3 AND target_entity_id = ?2)
         )",
        params![project, source_id, target_id],
    )?;
    transaction.execute(
        "UPDATE relation SET source_entity_id = ?3, updated_at = ?4
         WHERE project_id = ?1 AND source_entity_id = ?2",
        params![project, source_id, target_id, timestamp],
    )?;
    transaction.execute(
        "UPDATE relation SET target_entity_id = ?3, updated_at = ?4
         WHERE project_id = ?1 AND target_entity_id = ?2",
        params![project, source_id, target_id, timestamp],
    )?;
    transaction.execute(
        "UPDATE plot_card SET pov_entity_id = ?3, updated_at = ?4
         WHERE project_id = ?1 AND pov_entity_id = ?2",
        params![project, source_id, target_id, timestamp],
    )?;
    Ok(())
}

const fn kind_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "character",
        EntityKind::Location => "location",
        EntityKind::Faction => "faction",
        EntityKind::Item => "item",
    }
}

fn parse_entity_kind(value: &str) -> Result<EntityKind, StoryBibleError> {
    match value {
        "character" => Ok(EntityKind::Character),
        "location" => Ok(EntityKind::Location),
        "faction" => Ok(EntityKind::Faction),
        "item" => Ok(EntityKind::Item),
        _ => Err(StoryBibleError::InvalidStoredData),
    }
}

fn parse_document_kind(value: &str) -> Result<DocumentKind, StoryBibleError> {
    match value {
        "chapter" => Ok(DocumentKind::Chapter),
        "scene" => Ok(DocumentKind::Scene),
        _ => Err(StoryBibleError::InvalidStoredData),
    }
}

fn current_timestamp() -> Result<String, StoryBibleError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StoryBibleError::InvalidSystemTime)?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kraqel_domain::{
        CURRENT_SCHEMA_VERSION, DocumentKind, EntityId, EntityKind, Project, ProjectId,
    };
    use rusqlite::params;
    use tempfile::tempdir;

    use super::{StoryBibleError, StoryBibleRepository};
    use crate::{document_repository::DocumentRepository, project_manager::ProjectManager};

    #[test]
    fn stores_searches_and_updates_free_properties() {
        let (_root, project, mut opened) = fixture();
        let mut repository = StoryBibleRepository::new(opened.database_mut());
        let properties = BTreeMap::from([("Funktion".to_owned(), "Navigatorin".to_owned())]);
        let entity = repository
            .create_entity(project.id, EntityKind::Character, "Änne", &properties)
            .expect("entity should be created");
        assert_eq!(
            repository
                .list(project.id, Some("änne"), None)
                .expect("search should work"),
            vec![entity.clone()]
        );
        assert_eq!(
            repository
                .list(project.id, Some("navigatorin"), Some(EntityKind::Character))
                .expect("property search should work")
                .len(),
            1
        );

        let updated_properties = BTreeMap::from([("Wohnort".to_owned(), "Mars".to_owned())]);
        let updated = repository
            .update_entity(
                project.id,
                entity.id,
                EntityKind::Character,
                "Dr. Änne",
                &updated_properties,
            )
            .expect("entity should update");
        assert_eq!(
            repository
                .load(project.id, entity.id)
                .expect("entity should load")
                .entity,
            updated
        );
    }

    #[test]
    fn keeps_relation_direction_and_cascades_deleted_entities() {
        let (_root, project, mut opened) = fixture();
        let mut repository = StoryBibleRepository::new(opened.database_mut());
        let source = repository
            .create_entity(project.id, EntityKind::Character, "Ada", &BTreeMap::new())
            .expect("source should exist");
        let target = repository
            .create_entity(project.id, EntityKind::Faction, "Archiv", &BTreeMap::new())
            .expect("target should exist");
        let relation = repository
            .create_relation(
                project.id,
                source.id,
                target.id,
                "gründet",
                &BTreeMap::new(),
            )
            .expect("relation should exist");
        assert_eq!(
            repository
                .load(project.id, source.id)
                .expect("source should load")
                .outgoing_relations,
            vec![relation.clone()]
        );
        assert_eq!(
            repository
                .load(project.id, target.id)
                .expect("target should load")
                .incoming_relations,
            vec![relation]
        );
        assert!(matches!(
            repository.create_relation(project.id, source.id, source.id, "kennt", &BTreeMap::new()),
            Err(StoryBibleError::InvalidInput)
        ));

        repository
            .delete_entity(project.id, target.id)
            .expect("target should delete");
        assert_eq!(
            repository
                .load(project.id, source.id)
                .expect("source should remain")
                .outgoing_relations,
            Vec::new()
        );
    }

    #[test]
    fn exposes_scene_backlinks_idempotently() {
        let (_root, project, mut opened) = fixture();
        let scene = {
            let mut documents = DocumentRepository::new(opened.database_mut());
            let chapter = documents
                .create(project.id, None, DocumentKind::Chapter, "Kapitel")
                .expect("chapter should exist");
            documents
                .create(
                    project.id,
                    Some(chapter.id),
                    DocumentKind::Scene,
                    "Begegnung",
                )
                .expect("scene should exist")
        };
        let mut repository = StoryBibleRepository::new(opened.database_mut());
        let entity = repository
            .create_entity(
                project.id,
                EntityKind::Location,
                "Bahnhof",
                &BTreeMap::new(),
            )
            .expect("entity should exist");
        repository
            .link_document(project.id, scene.id, entity.id)
            .expect("link should exist");
        repository
            .link_document(project.id, scene.id, entity.id)
            .expect("duplicate should be harmless");
        let details = repository
            .load(project.id, entity.id)
            .expect("entity should load");
        assert_eq!(details.backlinks.len(), 1);
        assert_eq!(details.backlinks[0].document_title, "Begegnung");
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Der vertikale Transaktionstest belegt alle Referenzarten und den atomaren Gesamtvertrag"
    )]
    fn merges_duplicate_entities_and_retargets_every_reference_atomically() {
        let (_root, project, mut opened) = fixture();
        let scene = {
            let mut documents = DocumentRepository::new(opened.database_mut());
            let chapter = documents
                .create(project.id, None, DocumentKind::Chapter, "Kapitel")
                .expect("chapter should exist");
            documents
                .create(
                    project.id,
                    Some(chapter.id),
                    DocumentKind::Scene,
                    "Begegnung",
                )
                .expect("scene should exist")
        };
        let (source, target, peer) = {
            let mut repository = StoryBibleRepository::new(opened.database_mut());
            let source = repository
                .create_entity(
                    project.id,
                    EntityKind::Character,
                    "Mara (Dublette)",
                    &BTreeMap::from([
                        ("Rolle".to_owned(), "Späherin".to_owned()),
                        ("Herkunft".to_owned(), "Veyra".to_owned()),
                    ]),
                )
                .expect("source should exist");
            let target = repository
                .create_entity(
                    project.id,
                    EntityKind::Character,
                    "Mara",
                    &BTreeMap::from([
                        ("Rolle".to_owned(), "Navigatorin".to_owned()),
                        ("Ziel".to_owned(), "Dienstag finden".to_owned()),
                    ]),
                )
                .expect("target should exist");
            let peer = repository
                .create_entity(project.id, EntityKind::Faction, "Archiv", &BTreeMap::new())
                .expect("peer should exist");
            repository
                .link_document(project.id, scene.id, source.id)
                .expect("source link should exist");
            repository
                .link_document(project.id, scene.id, target.id)
                .expect("target link should exist");
            repository
                .create_relation(project.id, source.id, peer.id, "kennt", &BTreeMap::new())
                .expect("outgoing relation should exist");
            repository
                .create_relation(project.id, peer.id, source.id, "sucht", &BTreeMap::new())
                .expect("incoming relation should exist");
            repository
                .create_relation(
                    project.id,
                    source.id,
                    target.id,
                    "Dublette",
                    &BTreeMap::new(),
                )
                .expect("direct relation should exist");
            (source, target, peer)
        };

        let timeline_id = EntityId::new();
        let event_id = EntityId::new();
        let asset_id = EntityId::new();
        let draft_id = EntityId::new();
        let connection = opened.database().connection();
        connection
            .execute(
                "INSERT INTO plot_card
                    (project_id, document_id, storyline_id, pov_entity_id, status, position, updated_at)
                 VALUES (?2, ?3, NULL, ?1, 'planned', 0, 'unix:1')",
                params![source.id.to_string(), project.id.to_string(), scene.id.to_string()],
            )
            .expect("POV should reference source");
        connection
            .execute(
                "INSERT INTO timeline (id, project_id, name, mode, position, created_at, updated_at)
                 VALUES (?1, ?2, 'Haupthandlung', 'relative', 0, 'unix:1', 'unix:1')",
                params![timeline_id.to_string(), project.id.to_string()],
            )
            .expect("timeline should exist");
        connection
            .execute(
                "INSERT INTO event
                    (id, project_id, timeline_id, title, description, time_value, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'Ankunft', '', NULL, 0, 'unix:1', 'unix:1')",
                params![event_id.to_string(), project.id.to_string(), timeline_id.to_string()],
            )
            .expect("event should exist");
        for entity_id in [source.id, target.id] {
            connection
                .execute(
                    "INSERT INTO event_entity_link (project_id, event_id, entity_id, created_at)
                     VALUES (?1, ?2, ?3, 'unix:1')",
                    params![
                        project.id.to_string(),
                        event_id.to_string(),
                        entity_id.to_string()
                    ],
                )
                .expect("event link should exist");
        }
        connection
            .execute(
                "INSERT INTO media_asset
                    (id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                     mime_type, checksum, byte_count, width, height, provenance, tags_json,
                     created_at, updated_at)
                 VALUES (?1, ?2, 'image', 'assets/mara.png', 'assets/thumbnails/mara.png',
                         'mara.png', 'Mara', 'image/png', ?3, 1, 1, 1, 'imported', '[]',
                         'unix:1', 'unix:1')",
                params![asset_id.to_string(), project.id.to_string(), "a".repeat(64)],
            )
            .expect("asset should exist");
        for entity_id in [source.id, target.id] {
            connection
                .execute(
                    "INSERT INTO asset_link
                        (project_id, asset_id, target_type, target_id, role, created_at)
                     VALUES (?1, ?2, 'entity', ?3, 'reference', 'unix:1')",
                    params![
                        project.id.to_string(),
                        asset_id.to_string(),
                        entity_id.to_string()
                    ],
                )
                .expect("asset link should exist");
        }
        connection
            .execute(
                "INSERT INTO visual_profile
                    (entity_id, project_id, traits_json, canonical_asset_id, exclusions_json,
                     created_at, updated_at)
                 VALUES (?1, ?2, '{\"Augen\":\"grau\"}', ?3, '[]', 'unix:1', 'unix:1')",
                params![
                    source.id.to_string(),
                    project.id.to_string(),
                    asset_id.to_string()
                ],
            )
            .expect("source profile should exist");
        connection
            .execute(
                "INSERT INTO image_prompt_draft
                    (id, project_id, name, source_document_id, source_document_revision,
                     style_preset_id, prompt_text, created_at, updated_at)
                 VALUES (?1, ?2, 'Mara', ?3, 0, NULL, 'Porträt', 'unix:1', 'unix:1')",
                params![
                    draft_id.to_string(),
                    project.id.to_string(),
                    scene.id.to_string()
                ],
            )
            .expect("draft should exist");
        connection
            .execute(
                "INSERT INTO image_prompt_draft_profile (draft_id, project_id, entity_id, position)
                 VALUES (?1, ?2, ?3, 0)",
                params![
                    draft_id.to_string(),
                    project.id.to_string(),
                    source.id.to_string()
                ],
            )
            .expect("draft profile should exist");

        let merged = StoryBibleRepository::new(opened.database_mut())
            .merge_entities(project.id, source.id, target.id)
            .expect("merge should succeed");

        assert_eq!(merged.entity.id, target.id);
        assert_eq!(merged.entity.name, "Mara");
        assert_eq!(merged.entity.properties["Rolle"], "Navigatorin");
        assert_eq!(merged.entity.properties["Herkunft"], "Veyra");
        assert_eq!(merged.entity.properties["Ziel"], "Dienstag finden");
        assert_eq!(merged.backlinks.len(), 1);
        assert_eq!(merged.outgoing_relations.len(), 1);
        assert_eq!(merged.outgoing_relations[0].target_entity_id, peer.id);
        assert_eq!(merged.incoming_relations.len(), 1);
        assert_eq!(merged.incoming_relations[0].source_entity_id, peer.id);

        let connection = opened.database().connection();
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM entity WHERE project_id = ?1 AND id = ?2",
                    params![project.id.to_string(), source.id.to_string()],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            0
        );
        for (table, column) in [
            ("plot_card", "pov_entity_id"),
            ("event_entity_link", "entity_id"),
            ("visual_profile", "entity_id"),
            ("image_prompt_draft_profile", "entity_id"),
        ] {
            let count = connection
                .query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE {column} = ?1"),
                    [target.id.to_string()],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap();
            assert_eq!(count, 1, "{table} should reference the target exactly once");
        }
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM asset_link
                     WHERE target_type = 'entity' AND target_id = ?1",
                    [target.id.to_string()],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
    }

    #[test]
    fn rejects_two_visual_profiles_without_changing_either_entity() {
        let (_root, project, mut opened) = fixture();
        let (source, target) = {
            let mut repository = StoryBibleRepository::new(opened.database_mut());
            let source = repository
                .create_entity(
                    project.id,
                    EntityKind::Location,
                    "Alter Hafen",
                    &BTreeMap::from([("Quelle".to_owned(), "Dublette".to_owned())]),
                )
                .unwrap();
            let target = repository
                .create_entity(
                    project.id,
                    EntityKind::Location,
                    "Hafen",
                    &BTreeMap::from([("Kanon".to_owned(), "Ja".to_owned())]),
                )
                .unwrap();
            (source, target)
        };
        for entity_id in [source.id, target.id] {
            opened
                .database()
                .connection()
                .execute(
                    "INSERT INTO visual_profile
                        (entity_id, project_id, traits_json, canonical_asset_id, exclusions_json,
                         created_at, updated_at)
                     VALUES (?1, ?2, '{}', NULL, '[]', 'unix:1', 'unix:1')",
                    params![entity_id.to_string(), project.id.to_string()],
                )
                .unwrap();
        }

        assert!(matches!(
            StoryBibleRepository::new(opened.database_mut())
                .merge_entities(project.id, source.id, target.id),
            Err(StoryBibleError::MergeConflict)
        ));
        let repository = StoryBibleRepository::new(opened.database_mut());
        assert_eq!(
            repository.load(project.id, source.id).unwrap().entity,
            source
        );
        assert_eq!(
            repository.load(project.id, target.id).unwrap().entity,
            target
        );
    }

    #[test]
    fn story_bible_survives_backup_and_recovery() {
        let root = tempdir().expect("root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("quelle", &project)
            .expect("project should exist");
        StoryBibleRepository::new(opened.database_mut())
            .create_entity(project.id, EntityKind::Item, "Kompass", &BTreeMap::new())
            .expect("entity should exist");
        drop(opened);
        manager
            .backup_project("quelle", "m2")
            .expect("backup should exist");
        let mut restored = manager
            .restore_backup("quelle", "m2", "wiederhergestellt")
            .expect("backup should restore");
        assert_eq!(
            StoryBibleRepository::new(restored.database_mut())
                .list(project.id, None, None)
                .expect("entities should load")
                .len(),
            1
        );
    }

    #[test]
    fn rejects_stored_properties_outside_the_public_contract() {
        let (_root, project, mut opened) = fixture();
        let entity = StoryBibleRepository::new(opened.database_mut())
            .create_entity(project.id, EntityKind::Character, "Rhea", &BTreeMap::new())
            .expect("entity should exist");
        opened
            .database()
            .connection()
            .execute(
                "UPDATE entity SET properties_json = '{\"Rolle\": 7}' WHERE id = ?1",
                [entity.id.to_string()],
            )
            .expect("corrupt property should be stored for the test");

        assert!(matches!(
            StoryBibleRepository::new(opened.database_mut()).load(project.id, entity.id),
            Err(StoryBibleError::InvalidStoredData)
        ));
    }

    fn fixture() -> (
        tempfile::TempDir,
        Project,
        crate::project_manager::ManagedProject,
    ) {
        let root = tempdir().expect("root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let opened = manager
            .create_project("projekt", &project)
            .expect("project should exist");
        (root, project, opened)
    }

    fn project() -> Project {
        Project {
            id: ProjectId::new(),
            name: "Morgenrot".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }
}
