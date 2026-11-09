//! Persistenz für Plotboard, Handlungsstränge und parallele Zeitebenen.

use std::{
    collections::HashSet,
    error::Error,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{
    EntityId, PlotBoard, PlotCard, PlotStatus, ProjectId, Storyline, Timeline, TimelineDetails,
    TimelineEvent, TimelineMode,
};
use rusqlite::{OptionalExtension, Transaction, params};

use crate::{Database, StorageError};

/// Fehler des Plotboard- und Timeline-Repositorys.
#[derive(Debug)]
pub enum StructureError {
    /// Eingaben verletzen den öffentlichen Vertrag.
    InvalidInput,
    /// Ein referenziertes Objekt wurde nicht gefunden.
    NotFound,
    /// Gespeicherte Werte verletzen den Vertrag.
    InvalidStoredData,
    /// Fehler der Speicherschicht.
    Storage(StorageError),
    /// Ungültige Systemzeit.
    InvalidSystemTime,
}

impl fmt::Display for StructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die Struktur-Daten sind ungültig."),
            Self::NotFound => formatter.write_str("Das Struktur-Objekt wurde nicht gefunden."),
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte Struktur-Daten sind ungültig.")
            }
            Self::Storage(error) => error.fmt(formatter),
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
        }
    }
}

impl Error for StructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StructureError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

impl From<StorageError> for StructureError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Repository über einer verwalteten Projektdatenbank.
pub struct StructureRepository<'database> {
    database: &'database mut Database,
}

impl<'database> StructureRepository<'database> {
    /// Bindet das Repository an eine Datenbank.
    pub const fn new(database: &'database mut Database) -> Self {
        Self { database }
    }

    /// Lädt das Plotboard und ergänzt fehlende Karten für vorhandene Szenen.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlendem Projekt, ungültigen gespeicherten Daten oder SQLite-Problemen.
    pub fn plot_board(&mut self, project_id: ProjectId) -> Result<PlotBoard, StructureError> {
        let transaction = self.database.transaction()?;
        require_project(&transaction, project_id)?;
        let mut missing = transaction.prepare(
            "SELECT d.id FROM document d LEFT JOIN plot_card p ON p.document_id = d.id
             WHERE d.project_id = ?1 AND d.kind = 'scene' AND p.document_id IS NULL
             ORDER BY d.parent_id, d.position, d.id",
        )?;
        let ids: Vec<String> = missing
            .query_map([project_id.to_string()], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        drop(missing);
        let mut position: u32 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM plot_card WHERE project_id = ?1",
            [project_id.to_string()],
            |row| row.get(0),
        )?;
        let timestamp = current_timestamp()?;
        for id in ids {
            transaction.execute(
                "INSERT INTO plot_card (project_id, document_id, status, position, updated_at)
                 VALUES (?1, ?2, 'planned', ?3, ?4)",
                params![project_id.to_string(), id, position, timestamp],
            )?;
            position = position
                .checked_add(1)
                .ok_or(StructureError::InvalidStoredData)?;
        }
        transaction.commit()?;
        Ok(PlotBoard {
            storylines: self.list_storylines(project_id)?,
            cards: self.list_cards(project_id)?,
        })
    }

    /// Erstellt einen Handlungsstrang am Ende der stabilen Reihenfolge.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigem Namen/Farbwert, fehlendem Projekt oder SQLite-Problemen.
    pub fn create_storyline(
        &mut self,
        project_id: ProjectId,
        name: &str,
        color: &str,
    ) -> Result<Storyline, StructureError> {
        validate_text(name, 120)?;
        if !valid_color(color) {
            return Err(StructureError::InvalidInput);
        }
        let transaction = self.database.transaction()?;
        require_project(&transaction, project_id)?;
        let position: u32 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM storyline WHERE project_id = ?1",
            [project_id.to_string()],
            |row| row.get(0),
        )?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        transaction.execute(
            "INSERT INTO storyline (id, project_id, name, color, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id.to_string(),
                project_id.to_string(),
                name.trim(),
                color,
                position,
                timestamp
            ],
        )?;
        transaction.commit()?;
        Ok(Storyline {
            id,
            project_id,
            name: name.trim().to_owned(),
            color: color.to_owned(),
            position,
        })
    }

    /// Löscht einen Handlungsstrang; zugeordnete Karten bleiben ohne Strang erhalten.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlendem Strang oder SQLite-Problemen.
    pub fn delete_storyline(
        &mut self,
        project_id: ProjectId,
        storyline_id: EntityId,
    ) -> Result<(), StructureError> {
        let transaction = self.database.transaction()?;
        transaction.execute(
            "UPDATE plot_card SET storyline_id = NULL, updated_at = ?3 WHERE project_id = ?1 AND storyline_id = ?2",
            params![project_id.to_string(), storyline_id.to_string(), current_timestamp()?],
        )?;
        let changed = transaction.execute(
            "DELETE FROM storyline WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), storyline_id.to_string()],
        )?;
        if changed == 0 {
            return Err(StructureError::NotFound);
        }
        transaction.commit()?;
        Ok(())
    }

    /// Aktualisiert Status, POV und Handlungsstrang einer Szenenkarte.
    ///
    /// # Errors
    /// Liefert einen Fehler bei projektfremden Links, fehlender Karte oder SQLite-Problemen.
    pub fn update_card(
        &mut self,
        project_id: ProjectId,
        document_id: EntityId,
        status: PlotStatus,
        storyline_id: Option<EntityId>,
        pov_entity_id: Option<EntityId>,
    ) -> Result<(), StructureError> {
        let transaction = self.database.transaction()?;
        if let Some(id) = storyline_id {
            require_storyline(&transaction, project_id, id)?;
        }
        if let Some(id) = pov_entity_id {
            require_character(&transaction, project_id, id)?;
        }
        let changed = transaction.execute(
            "UPDATE plot_card SET status = ?3, storyline_id = ?4, pov_entity_id = ?5, updated_at = ?6
             WHERE project_id = ?1 AND document_id = ?2",
            params![project_id.to_string(), document_id.to_string(), status_name(status), storyline_id.map(|id| id.to_string()), pov_entity_id.map(|id| id.to_string()), current_timestamp()?],
        )?;
        if changed == 0 {
            return Err(StructureError::NotFound);
        }
        transaction.commit()?;
        Ok(())
    }

    /// Speichert eine vollständige neue Kartenreihenfolge atomar.
    ///
    /// # Errors
    /// Liefert einen Fehler, wenn Kennungen fehlen/doppelt sind oder SQLite fehlschlägt.
    pub fn reorder_cards(
        &mut self,
        project_id: ProjectId,
        ordered_ids: &[EntityId],
    ) -> Result<(), StructureError> {
        let transaction = self.database.transaction()?;
        require_exact_ids(
            &transaction,
            "plot_card",
            "document_id",
            project_id,
            None,
            ordered_ids,
        )?;
        for (position, id) in ordered_ids.iter().enumerate() {
            transaction.execute(
                "UPDATE plot_card SET position = ?3, updated_at = ?4 WHERE project_id = ?1 AND document_id = ?2",
                params![project_id.to_string(), id.to_string(), u32::try_from(position).map_err(|_| StructureError::InvalidInput)?, current_timestamp()?],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Listet Zeitebenen stabil sortiert.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen gespeicherten Daten oder SQLite-Problemen.
    pub fn list_timelines(&self, project_id: ProjectId) -> Result<Vec<Timeline>, StructureError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, project_id, name, mode, position FROM timeline WHERE project_id = ?1 ORDER BY position, id"
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, u32>(4)?,
                ))
            })?
            .map(|row| parse_timeline(row?))
            .collect()
    }

    /// Erstellt eine relative oder absolute Zeitebene.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigem Namen, fehlendem Projekt oder SQLite-Problemen.
    pub fn create_timeline(
        &mut self,
        project_id: ProjectId,
        name: &str,
        mode: TimelineMode,
    ) -> Result<Timeline, StructureError> {
        validate_text(name, 120)?;
        let transaction = self.database.transaction()?;
        require_project(&transaction, project_id)?;
        let position: u32 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM timeline WHERE project_id = ?1",
            [project_id.to_string()],
            |row| row.get(0),
        )?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        transaction.execute(
            "INSERT INTO timeline (id, project_id, name, mode, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id.to_string(),
                project_id.to_string(),
                name.trim(),
                mode_name(mode),
                position,
                timestamp
            ],
        )?;
        transaction.commit()?;
        Ok(Timeline {
            id,
            project_id,
            name: name.trim().to_owned(),
            mode,
            position,
        })
    }

    /// Löscht eine Zeitebene samt Ereignissen und Links.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlender Zeitebene oder SQLite-Problemen.
    pub fn delete_timeline(
        &mut self,
        project_id: ProjectId,
        timeline_id: EntityId,
    ) -> Result<(), StructureError> {
        let changed = self.database.connection().execute(
            "DELETE FROM timeline WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), timeline_id.to_string()],
        )?;
        if changed == 0 {
            Err(StructureError::NotFound)
        } else {
            Ok(())
        }
    }

    /// Lädt eine Zeitebene samt Ereignissen und Links.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlender Zeitebene, ungültigen Daten oder SQLite-Problemen.
    pub fn load_timeline(
        &self,
        project_id: ProjectId,
        timeline_id: EntityId,
    ) -> Result<TimelineDetails, StructureError> {
        let timeline = self.database.connection().query_row(
            "SELECT id, project_id, name, mode, position FROM timeline WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), timeline_id.to_string()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, u32>(4)?))
        ).optional()?.map(parse_timeline).transpose()?.ok_or(StructureError::NotFound)?;
        let mut statement = self.database.connection().prepare(
            "SELECT id, title, description, time_value, position FROM event
             WHERE project_id = ?1 AND timeline_id = ?2 ORDER BY position, id",
        )?;
        let raw: Vec<(String, String, String, Option<String>, u32)> = statement
            .query_map(
                params![project_id.to_string(), timeline_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )?
            .collect::<Result<_, _>>()?;
        drop(statement);
        let events = raw
            .into_iter()
            .map(|(id, title, description, time_value, position)| {
                let id = EntityId::parse(&id).ok_or(StructureError::InvalidStoredData)?;
                validate_text(&title, 240).map_err(|_| StructureError::InvalidStoredData)?;
                if description.chars().count() > 8_000 {
                    return Err(StructureError::InvalidStoredData);
                }
                validate_event_time(timeline.mode, time_value.as_deref())
                    .map_err(|_| StructureError::InvalidStoredData)?;
                Ok(TimelineEvent {
                    id,
                    project_id,
                    timeline_id,
                    title,
                    description,
                    time_value,
                    position,
                    document_ids: self.link_ids(
                        "event_document_link",
                        "document_id",
                        project_id,
                        id,
                    )?,
                    entity_ids: self.link_ids("event_entity_link", "entity_id", project_id, id)?,
                })
            })
            .collect::<Result<_, StructureError>>()?;
        Ok(TimelineDetails { timeline, events })
    }

    /// Erstellt ein Ereignis am Ende einer Zeitebene.
    ///
    /// # Errors
    /// Liefert einen Fehler bei unpassendem Zeitwert, fehlender Zeitebene oder SQLite-Problemen.
    pub fn create_event(
        &mut self,
        project_id: ProjectId,
        timeline_id: EntityId,
        title: &str,
        description: &str,
        time_value: Option<&str>,
    ) -> Result<TimelineEvent, StructureError> {
        validate_text(title, 240)?;
        if description.chars().count() > 8_000 {
            return Err(StructureError::InvalidInput);
        }
        let transaction = self.database.transaction()?;
        let mode = require_timeline(&transaction, project_id, timeline_id)?;
        validate_event_time(mode, time_value)?;
        let position: u32 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM event WHERE project_id = ?1 AND timeline_id = ?2",
            params![project_id.to_string(), timeline_id.to_string()], |row| row.get(0)
        )?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        transaction.execute(
            "INSERT INTO event (id, project_id, timeline_id, title, description, time_value, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![id.to_string(), project_id.to_string(), timeline_id.to_string(), title.trim(), description, time_value.map(str::trim), position, timestamp],
        )?;
        transaction.commit()?;
        Ok(TimelineEvent {
            id,
            project_id,
            timeline_id,
            title: title.trim().to_owned(),
            description: description.to_owned(),
            time_value: time_value.map(str::trim).map(str::to_owned),
            position,
            document_ids: Vec::new(),
            entity_ids: Vec::new(),
        })
    }

    /// Löscht ein Ereignis samt Links.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlendem Ereignis oder SQLite-Problemen.
    pub fn delete_event(
        &mut self,
        project_id: ProjectId,
        event_id: EntityId,
    ) -> Result<(), StructureError> {
        let changed = self.database.connection().execute(
            "DELETE FROM event WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), event_id.to_string()],
        )?;
        if changed == 0 {
            Err(StructureError::NotFound)
        } else {
            Ok(())
        }
    }

    /// Speichert die vollständige Ereignisreihenfolge einer Zeitebene atomar.
    ///
    /// # Errors
    /// Liefert einen Fehler bei unvollständiger Reihenfolge oder SQLite-Problemen.
    pub fn reorder_events(
        &mut self,
        project_id: ProjectId,
        timeline_id: EntityId,
        ordered_ids: &[EntityId],
    ) -> Result<(), StructureError> {
        let transaction = self.database.transaction()?;
        require_exact_ids(
            &transaction,
            "event",
            "id",
            project_id,
            Some(timeline_id),
            ordered_ids,
        )?;
        for (position, id) in ordered_ids.iter().enumerate() {
            transaction.execute(
                "UPDATE event SET position = ?3, updated_at = ?4 WHERE project_id = ?1 AND id = ?2",
                params![
                    project_id.to_string(),
                    id.to_string(),
                    u32::try_from(position).map_err(|_| StructureError::InvalidInput)?,
                    current_timestamp()?
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Verknüpft ein Ereignis idempotent mit einer Szene oder Entität desselben Projekts.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlenden oder projektfremden Objekten sowie SQLite-Problemen.
    pub fn link_event(
        &mut self,
        project_id: ProjectId,
        event_id: EntityId,
        document_id: Option<EntityId>,
        entity_id: Option<EntityId>,
    ) -> Result<(), StructureError> {
        if document_id.is_some() == entity_id.is_some() {
            return Err(StructureError::InvalidInput);
        }
        let transaction = self.database.transaction()?;
        require_event(&transaction, project_id, event_id)?;
        let timestamp = current_timestamp()?;
        if let Some(id) = document_id {
            require_scene(&transaction, project_id, id)?;
            transaction.execute(
                "INSERT OR IGNORE INTO event_document_link (project_id, event_id, document_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![project_id.to_string(), event_id.to_string(), id.to_string(), timestamp],
            )?;
        }
        if let Some(id) = entity_id {
            require_any_entity(&transaction, project_id, id)?;
            transaction.execute(
                "INSERT OR IGNORE INTO event_entity_link (project_id, event_id, entity_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![project_id.to_string(), event_id.to_string(), id.to_string(), timestamp],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn list_storylines(&self, project_id: ProjectId) -> Result<Vec<Storyline>, StructureError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, name, color, position FROM storyline WHERE project_id = ?1 ORDER BY position, id"
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, u32>(3)?,
                ))
            })?
            .map(|row| {
                let (id, name, color, position) = row?;
                validate_text(&name, 120).map_err(|_| StructureError::InvalidStoredData)?;
                if !valid_color(&color) {
                    return Err(StructureError::InvalidStoredData);
                }
                Ok(Storyline {
                    id: EntityId::parse(&id).ok_or(StructureError::InvalidStoredData)?,
                    project_id,
                    name,
                    color,
                    position,
                })
            })
            .collect()
    }

    fn list_cards(&self, project_id: ProjectId) -> Result<Vec<PlotCard>, StructureError> {
        let mut statement = self.database.connection().prepare(
            "SELECT p.document_id, d.title, p.storyline_id, p.pov_entity_id, p.status, p.position
             FROM plot_card p JOIN document d ON d.id = p.document_id AND d.project_id = p.project_id
             WHERE p.project_id = ?1 ORDER BY p.position, p.document_id"
        )?;
        statement
            .query_map([project_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, u32>(5)?,
                ))
            })?
            .map(|row| {
                let (document, title, storyline, pov, status, position) = row?;
                Ok(PlotCard {
                    document_id: parse_id(&document)?,
                    title,
                    storyline_id: storyline.map(|id| parse_id(&id)).transpose()?,
                    pov_entity_id: pov.map(|id| parse_id(&id)).transpose()?,
                    status: parse_status(&status)?,
                    position,
                })
            })
            .collect()
    }

    fn link_ids(
        &self,
        table: &str,
        column: &str,
        project_id: ProjectId,
        event_id: EntityId,
    ) -> Result<Vec<EntityId>, StructureError> {
        let sql = format!(
            "SELECT {column} FROM {table} WHERE project_id = ?1 AND event_id = ?2 ORDER BY {column}"
        );
        let mut statement = self.database.connection().prepare(&sql)?;
        statement
            .query_map(
                params![project_id.to_string(), event_id.to_string()],
                |row| row.get::<_, String>(0),
            )?
            .map(|row| parse_id(&row?))
            .collect()
    }
}

fn require_exact_ids(
    transaction: &Transaction<'_>,
    table: &str,
    column: &str,
    project_id: ProjectId,
    timeline_id: Option<EntityId>,
    ids: &[EntityId],
) -> Result<(), StructureError> {
    let unique: HashSet<_> = ids.iter().copied().collect();
    if unique.len() != ids.len() {
        return Err(StructureError::InvalidInput);
    }
    let sql = if timeline_id.is_some() {
        format!("SELECT {column} FROM {table} WHERE project_id = ?1 AND timeline_id = ?2")
    } else {
        format!("SELECT {column} FROM {table} WHERE project_id = ?1")
    };
    let mut statement = transaction.prepare(&sql)?;
    let stored: HashSet<EntityId> = if let Some(timeline) = timeline_id {
        statement
            .query_map(
                params![project_id.to_string(), timeline.to_string()],
                |row| row.get::<_, String>(0),
            )?
            .map(|row| parse_id(&row?))
            .collect::<Result<_, _>>()?
    } else {
        statement
            .query_map([project_id.to_string()], |row| row.get::<_, String>(0))?
            .map(|row| parse_id(&row?))
            .collect::<Result<_, _>>()?
    };
    if unique == stored {
        Ok(())
    } else {
        Err(StructureError::InvalidInput)
    }
}

fn require_project(tx: &Transaction<'_>, project: ProjectId) -> Result<(), StructureError> {
    require_exists(tx, "project", project, None)
}
fn require_storyline(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<(), StructureError> {
    require_exists(tx, "storyline", project, Some(id))
}
fn require_event(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<(), StructureError> {
    require_exists(tx, "event", project, Some(id))
}
fn require_any_entity(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<(), StructureError> {
    require_exists(tx, "entity", project, Some(id))
}

fn require_exists(
    tx: &Transaction<'_>,
    table: &str,
    project: ProjectId,
    id: Option<EntityId>,
) -> Result<(), StructureError> {
    let sql = if id.is_some() {
        format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE project_id = ?1 AND id = ?2)")
    } else {
        format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)")
    };
    let exists = if let Some(id) = id {
        tx.query_row(&sql, params![project.to_string(), id.to_string()], |row| {
            row.get::<_, bool>(0)
        })?
    } else {
        tx.query_row(&sql, [project.to_string()], |row| row.get::<_, bool>(0))?
    };
    if exists {
        Ok(())
    } else {
        Err(StructureError::NotFound)
    }
}

fn require_character(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<(), StructureError> {
    let exists = tx.query_row("SELECT EXISTS(SELECT 1 FROM entity WHERE project_id = ?1 AND id = ?2 AND kind = 'character')", params![project.to_string(), id.to_string()], |row| row.get::<_, bool>(0))?;
    if exists {
        Ok(())
    } else {
        Err(StructureError::NotFound)
    }
}

fn require_scene(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<(), StructureError> {
    let exists = tx.query_row("SELECT EXISTS(SELECT 1 FROM document WHERE project_id = ?1 AND id = ?2 AND kind = 'scene')", params![project.to_string(), id.to_string()], |row| row.get::<_, bool>(0))?;
    if exists {
        Ok(())
    } else {
        Err(StructureError::NotFound)
    }
}

fn require_timeline(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: EntityId,
) -> Result<TimelineMode, StructureError> {
    tx.query_row(
        "SELECT mode FROM timeline WHERE project_id = ?1 AND id = ?2",
        params![project.to_string(), id.to_string()],
        |row| row.get::<_, String>(0),
    )
    .optional()?
    .ok_or(StructureError::NotFound)
    .and_then(|mode| parse_mode(&mode))
}

fn parse_timeline(raw: (String, String, String, String, u32)) -> Result<Timeline, StructureError> {
    validate_text(&raw.2, 120).map_err(|_| StructureError::InvalidStoredData)?;
    Ok(Timeline {
        id: parse_id(&raw.0)?,
        project_id: ProjectId::parse(&raw.1).ok_or(StructureError::InvalidStoredData)?,
        name: raw.2,
        mode: parse_mode(&raw.3)?,
        position: raw.4,
    })
}

fn parse_id(value: &str) -> Result<EntityId, StructureError> {
    EntityId::parse(value).ok_or(StructureError::InvalidStoredData)
}
fn parse_status(value: &str) -> Result<PlotStatus, StructureError> {
    match value {
        "planned" => Ok(PlotStatus::Planned),
        "draft" => Ok(PlotStatus::Draft),
        "revised" => Ok(PlotStatus::Revised),
        "done" => Ok(PlotStatus::Done),
        _ => Err(StructureError::InvalidStoredData),
    }
}
const fn status_name(value: PlotStatus) -> &'static str {
    match value {
        PlotStatus::Planned => "planned",
        PlotStatus::Draft => "draft",
        PlotStatus::Revised => "revised",
        PlotStatus::Done => "done",
    }
}
fn parse_mode(value: &str) -> Result<TimelineMode, StructureError> {
    match value {
        "relative" => Ok(TimelineMode::Relative),
        "absolute" => Ok(TimelineMode::Absolute),
        _ => Err(StructureError::InvalidStoredData),
    }
}
const fn mode_name(value: TimelineMode) -> &'static str {
    match value {
        TimelineMode::Relative => "relative",
        TimelineMode::Absolute => "absolute",
    }
}

fn validate_text(value: &str, maximum: usize) -> Result<(), StructureError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > maximum || value.chars().any(char::is_control) {
        Err(StructureError::InvalidInput)
    } else {
        Ok(())
    }
}
fn valid_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}
fn validate_event_time(mode: TimelineMode, value: Option<&str>) -> Result<(), StructureError> {
    match (mode, value.map(str::trim).filter(|value| !value.is_empty())) {
        (TimelineMode::Absolute, Some(value))
            if value.chars().count() <= 80 && !value.chars().any(char::is_control) =>
        {
            Ok(())
        }
        (TimelineMode::Relative, None) => Ok(()),
        _ => Err(StructureError::InvalidInput),
    }
}
fn current_timestamp() -> Result<String, StructureError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StructureError::InvalidSystemTime)?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kraqel_domain::{
        CURRENT_SCHEMA_VERSION, DocumentKind, EntityKind, PlotStatus, Project, ProjectId,
        TimelineMode,
    };
    use tempfile::tempdir;

    use super::{StructureError, StructureRepository};
    use crate::{
        document_repository::DocumentRepository, project_manager::ProjectManager,
        story_bible_repository::StoryBibleRepository,
    };

    #[test]
    fn plot_cards_keep_order_status_storyline_and_character_pov() {
        let (_root, project, mut opened) = fixture();
        let (first, second) = scenes(&mut opened, project.id);
        let character = StoryBibleRepository::new(opened.database_mut())
            .create_entity(
                project.id,
                EntityKind::Character,
                "Rhea",
                &BTreeMap::default(),
            )
            .expect("character should exist");
        let mut repository = StructureRepository::new(opened.database_mut());
        let board = repository
            .plot_board(project.id)
            .expect("board should load");
        assert_eq!(board.cards.len(), 2);
        let storyline = repository
            .create_storyline(project.id, "Nordroute", "#336699")
            .expect("storyline should exist");
        repository
            .update_card(
                project.id,
                first,
                PlotStatus::Draft,
                Some(storyline.id),
                Some(character.id),
            )
            .expect("card should update");
        repository
            .reorder_cards(project.id, &[second, first])
            .expect("cards should reorder");
        let board = repository
            .plot_board(project.id)
            .expect("board should reload");
        assert_eq!(board.cards[0].document_id, second);
        assert_eq!(board.cards[1].status, PlotStatus::Draft);
        assert_eq!(board.cards[1].pov_entity_id, Some(character.id));
    }

    #[test]
    fn parallel_timelines_enforce_time_model_and_keep_links() {
        let (_root, project, mut opened) = fixture();
        let (scene, _) = scenes(&mut opened, project.id);
        let entity = StoryBibleRepository::new(opened.database_mut())
            .create_entity(
                project.id,
                EntityKind::Location,
                "Hafen",
                &BTreeMap::default(),
            )
            .expect("entity should exist");
        let mut repository = StructureRepository::new(opened.database_mut());
        let relative = repository
            .create_timeline(project.id, "Traum", TimelineMode::Relative)
            .expect("relative timeline should exist");
        let absolute = repository
            .create_timeline(project.id, "Kalender", TimelineMode::Absolute)
            .expect("absolute timeline should exist");
        assert!(matches!(
            repository.create_event(project.id, relative.id, "Falsch", "", Some("Tag 1")),
            Err(StructureError::InvalidInput)
        ));
        assert!(matches!(
            repository.create_event(project.id, absolute.id, "Falsch", "", None),
            Err(StructureError::InvalidInput)
        ));
        let event = repository
            .create_event(project.id, absolute.id, "Ankunft", "", Some("2042-05-17"))
            .expect("event should exist");
        repository
            .link_event(project.id, event.id, Some(scene), None)
            .expect("scene should link");
        repository
            .link_event(project.id, event.id, None, Some(entity.id))
            .expect("entity should link");
        let loaded = repository
            .load_timeline(project.id, absolute.id)
            .expect("timeline should load");
        assert_eq!(loaded.events[0].document_ids, vec![scene]);
        assert_eq!(loaded.events[0].entity_ids, vec![entity.id]);
    }

    #[test]
    fn plotboard_and_timelines_survive_backup_and_recovery() {
        let root = tempdir().expect("root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = Project {
            id: ProjectId::new(),
            name: "Morgenrot".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let mut opened = manager
            .create_project("quelle", &project)
            .expect("project should exist");
        scenes(&mut opened, project.id);
        let timeline_id = {
            let mut repository = StructureRepository::new(opened.database_mut());
            repository
                .plot_board(project.id)
                .expect("board should load");
            repository
                .create_storyline(project.id, "Nordroute", "#336699")
                .expect("storyline should exist");
            let timeline = repository
                .create_timeline(project.id, "Kalender", TimelineMode::Absolute)
                .expect("timeline should exist");
            repository
                .create_event(project.id, timeline.id, "Ankunft", "", Some("2042-05-17"))
                .expect("event should exist");
            timeline.id
        };
        drop(opened);
        manager
            .backup_project("quelle", "m2")
            .expect("backup should work");
        let mut restored = manager
            .restore_backup("quelle", "m2", "wiederhergestellt")
            .expect("restore should work");
        let mut repository = StructureRepository::new(restored.database_mut());
        assert_eq!(
            repository
                .plot_board(project.id)
                .expect("board should load")
                .storylines
                .len(),
            1
        );
        assert_eq!(
            repository
                .load_timeline(project.id, timeline_id)
                .expect("timeline should load")
                .events
                .len(),
            1
        );
    }

    fn fixture() -> (
        tempfile::TempDir,
        Project,
        crate::project_manager::ManagedProject,
    ) {
        let root = tempdir().expect("root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = Project {
            id: ProjectId::new(),
            name: "Morgenrot".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let opened = manager
            .create_project("projekt", &project)
            .expect("project should exist");
        (root, project, opened)
    }

    fn scenes(
        opened: &mut crate::project_manager::ManagedProject,
        project: ProjectId,
    ) -> (kraqel_domain::EntityId, kraqel_domain::EntityId) {
        let mut repository = DocumentRepository::new(opened.database_mut());
        let chapter = repository
            .create(project, None, DocumentKind::Chapter, "Kapitel")
            .expect("chapter should exist");
        let first = repository
            .create(project, Some(chapter.id), DocumentKind::Scene, "Eins")
            .expect("scene should exist");
        let second = repository
            .create(project, Some(chapter.id), DocumentKind::Scene, "Zwei")
            .expect("scene should exist");
        (first.id, second.id)
    }
}
