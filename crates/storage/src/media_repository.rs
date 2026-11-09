//! Transaktionales Repository für lokale Medienmetadaten und fachliche Verknüpfungen.

use std::{
    error::Error,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{
    AssetLink, AssetTargetKind, EntityId, MediaAsset, MediaKind, MediaProvenance, ProjectId,
};
use rusqlite::{OptionalExtension, Row, Transaction, params};

use crate::{Database, StorageError};

const MAX_TAGS: usize = 30;
const MAX_TAG_CHARS: usize = 60;
const MAX_ROLE_CHARS: usize = 80;

#[derive(Debug)]
pub enum MediaError {
    InvalidInput,
    NotFound,
    Duplicate,
    InvalidStoredData,
    InvalidSystemTime,
    Storage(StorageError),
}

impl fmt::Display for MediaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die Mediendaten sind ungültig."),
            Self::NotFound => formatter.write_str("Das Medienasset wurde nicht gefunden."),
            Self::Duplicate => formatter.write_str("Das Medienasset ist bereits vorhanden."),
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte Mediendaten sind ungültig.")
            }
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
            Self::Storage(error) => error.fmt(formatter),
        }
    }
}

impl Error for MediaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MediaError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

impl From<StorageError> for MediaError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

pub struct MediaRepository<'database> {
    database: &'database mut Database,
}

impl<'database> MediaRepository<'database> {
    pub const fn new(database: &'database mut Database) -> Self {
        Self { database }
    }

    /// Listet Assets mit stabil sortierten Links.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen gespeicherten Daten oder SQLite-Problemen.
    pub fn list(&self, project_id: ProjectId) -> Result<Vec<MediaAsset>, MediaError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                    mime_type, checksum, byte_count, width, height, provenance, tags_json
             FROM media_asset WHERE project_id = ?1 ORDER BY title COLLATE NOCASE, id",
        )?;
        let raw = statement
            .query_map([project_id.to_string()], raw_asset)?
            .collect::<Result<Vec<_>, _>>()?;
        raw.into_iter().map(|row| self.parse_asset(row)).collect()
    }

    /// Lädt genau ein Asset.
    ///
    /// # Errors
    /// Liefert einen fehlenden Datensatz oder einen Speicherfehler.
    pub fn load(
        &self,
        project_id: ProjectId,
        asset_id: EntityId,
    ) -> Result<MediaAsset, MediaError> {
        let raw = self
            .database
            .connection()
            .query_row(
                "SELECT id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                    mime_type, checksum, byte_count, width, height, provenance, tags_json
             FROM media_asset WHERE project_id = ?1 AND id = ?2",
                params![project_id.to_string(), asset_id.to_string()],
                raw_asset,
            )
            .optional()?
            .ok_or(MediaError::NotFound)?;
        self.parse_asset(raw)
    }

    /// Sucht ein Asset anhand seines projektweit eindeutigen Inhalts-Hashes.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen Daten oder SQLite-Problemen.
    pub fn find_by_checksum(
        &self,
        project_id: ProjectId,
        checksum: &str,
    ) -> Result<Option<MediaAsset>, MediaError> {
        let id = self
            .database
            .connection()
            .query_row(
                "SELECT id FROM media_asset WHERE project_id = ?1 AND checksum = ?2",
                params![project_id.to_string(), checksum],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        id.map(|value| {
            let id = EntityId::parse(&value).ok_or(MediaError::InvalidStoredData)?;
            self.load(project_id, id)
        })
        .transpose()
    }

    /// Persistiert ein bereits validiertes und veröffentlichtes Bild.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen Metadaten, Duplikaten oder SQLite-Problemen.
    #[allow(
        clippy::too_many_arguments,
        reason = "Die Parameter bilden den expliziten Assetvertrag ab"
    )]
    pub fn insert(
        &mut self,
        project_id: ProjectId,
        id: EntityId,
        relative_path: &str,
        thumbnail_path: &str,
        original_name: &str,
        title: &str,
        mime_type: &str,
        checksum: &str,
        byte_count: u64,
        width: u32,
        height: u32,
        tags: &[String],
    ) -> Result<MediaAsset, MediaError> {
        let tags_json = validate_metadata(title, tags)?;
        validate_file_contract(
            relative_path,
            thumbnail_path,
            original_name,
            mime_type,
            checksum,
            byte_count,
            width,
            height,
        )?;
        let timestamp = current_timestamp()?;
        let result = self.database.connection().execute(
            "INSERT INTO media_asset (
                id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                mime_type, checksum, byte_count, width, height, provenance, tags_json,
                created_at, updated_at
             ) VALUES (?1, ?2, 'image', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                       'imported', ?12, ?13, ?13)",
            params![
                id.to_string(),
                project_id.to_string(),
                relative_path,
                thumbnail_path,
                original_name,
                title.trim(),
                mime_type,
                checksum,
                i64::try_from(byte_count).map_err(|_| MediaError::InvalidInput)?,
                width,
                height,
                tags_json,
                timestamp
            ],
        );
        match result {
            Ok(_) => self.load(project_id, id),
            Err(error)
                if error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation) =>
            {
                Err(MediaError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Ändert ausschließlich redaktionelle Metadaten.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen Daten, fehlendem Asset oder SQLite-Problemen.
    pub fn update_metadata(
        &mut self,
        project_id: ProjectId,
        asset_id: EntityId,
        title: &str,
        tags: &[String],
    ) -> Result<MediaAsset, MediaError> {
        let tags_json = validate_metadata(title, tags)?;
        let changed = self.database.connection().execute(
            "UPDATE media_asset SET title = ?3, tags_json = ?4, updated_at = ?5
             WHERE project_id = ?1 AND id = ?2",
            params![
                project_id.to_string(),
                asset_id.to_string(),
                title.trim(),
                tags_json,
                current_timestamp()?
            ],
        )?;
        if changed == 0 {
            return Err(MediaError::NotFound);
        }
        self.load(project_id, asset_id)
    }

    /// Schaltet ein Asset nach erfolgreicher Dateiveröffentlichung auf neuen Inhalt um.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen Daten, Duplikaten oder SQLite-Problemen.
    #[allow(
        clippy::too_many_arguments,
        reason = "Die Parameter bilden den expliziten Dateivertrag ab"
    )]
    pub fn replace_content(
        &mut self,
        project_id: ProjectId,
        asset_id: EntityId,
        relative_path: &str,
        thumbnail_path: &str,
        original_name: &str,
        mime_type: &str,
        checksum: &str,
        byte_count: u64,
        width: u32,
        height: u32,
    ) -> Result<MediaAsset, MediaError> {
        validate_file_contract(
            relative_path,
            thumbnail_path,
            original_name,
            mime_type,
            checksum,
            byte_count,
            width,
            height,
        )?;
        let result = self.database.connection().execute(
            "UPDATE media_asset SET relative_path = ?3, thumbnail_path = ?4, original_name = ?5,
                    mime_type = ?6, checksum = ?7, byte_count = ?8, width = ?9, height = ?10,
                    updated_at = ?11 WHERE project_id = ?1 AND id = ?2",
            params![
                project_id.to_string(),
                asset_id.to_string(),
                relative_path,
                thumbnail_path,
                original_name,
                mime_type,
                checksum,
                i64::try_from(byte_count).map_err(|_| MediaError::InvalidInput)?,
                width,
                height,
                current_timestamp()?
            ],
        );
        match result {
            Ok(0) => Err(MediaError::NotFound),
            Ok(_) => self.load(project_id, asset_id),
            Err(error)
                if error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation) =>
            {
                Err(MediaError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Löscht Assetmetadaten und alle Links; Dateien entfernt der aufrufende Dienst danach.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlendem Asset oder SQLite-Problemen.
    pub fn delete(
        &mut self,
        project_id: ProjectId,
        asset_id: EntityId,
    ) -> Result<MediaAsset, MediaError> {
        let asset = self.load(project_id, asset_id)?;
        let changed = self.database.connection().execute(
            "DELETE FROM media_asset WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), asset_id.to_string()],
        )?;
        if changed == 0 {
            return Err(MediaError::NotFound);
        }
        Ok(asset)
    }

    /// Verknüpft ein Asset idempotent mit einem existierenden Projektobjekt.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültiger Rolle, fehlendem Ziel oder SQLite-Problemen.
    pub fn link(
        &mut self,
        project_id: ProjectId,
        asset_id: EntityId,
        target_kind: AssetTargetKind,
        target_id: EntityId,
        role: &str,
    ) -> Result<MediaAsset, MediaError> {
        validate_role(role)?;
        let transaction = self.database.transaction()?;
        require_exists(&transaction, "media_asset", project_id, asset_id)?;
        require_target(&transaction, project_id, target_kind, target_id)?;
        transaction.execute(
            "INSERT OR IGNORE INTO asset_link (
                project_id, asset_id, target_type, target_id, role, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project_id.to_string(),
                asset_id.to_string(),
                target_name(target_kind),
                target_id.to_string(),
                role.trim(),
                current_timestamp()?
            ],
        )?;
        transaction.commit()?;
        self.load(project_id, asset_id)
    }

    /// Entfernt genau eine Assetverknüpfung idempotent.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültiger Rolle, fehlendem Asset oder SQLite-Problemen.
    pub fn unlink(
        &mut self,
        project_id: ProjectId,
        asset_id: EntityId,
        target_kind: AssetTargetKind,
        target_id: EntityId,
        role: &str,
    ) -> Result<MediaAsset, MediaError> {
        validate_role(role)?;
        require_asset(self.database.connection(), project_id, asset_id)?;
        self.database.connection().execute(
            "DELETE FROM asset_link WHERE project_id = ?1 AND asset_id = ?2
             AND target_type = ?3 AND target_id = ?4 AND role = ?5",
            params![
                project_id.to_string(),
                asset_id.to_string(),
                target_name(target_kind),
                target_id.to_string(),
                role.trim()
            ],
        )?;
        self.load(project_id, asset_id)
    }

    fn parse_asset(&self, raw: RawAsset) -> Result<MediaAsset, MediaError> {
        let id = EntityId::parse(&raw.id).ok_or(MediaError::InvalidStoredData)?;
        let project_id = ProjectId::parse(&raw.project_id).ok_or(MediaError::InvalidStoredData)?;
        let tags: Vec<String> =
            serde_json::from_str(&raw.tags_json).map_err(|_| MediaError::InvalidStoredData)?;
        validate_metadata(&raw.title, &tags).map_err(|_| MediaError::InvalidStoredData)?;
        validate_file_contract(
            &raw.relative_path,
            &raw.thumbnail_path,
            &raw.original_name,
            &raw.mime_type,
            &raw.checksum,
            u64::try_from(raw.byte_count).map_err(|_| MediaError::InvalidStoredData)?,
            raw.width,
            raw.height,
        )
        .map_err(|_| MediaError::InvalidStoredData)?;
        Ok(MediaAsset {
            id,
            project_id,
            kind: if raw.kind == "image" {
                MediaKind::Image
            } else {
                return Err(MediaError::InvalidStoredData);
            },
            relative_path: raw.relative_path,
            thumbnail_path: raw.thumbnail_path,
            original_name: raw.original_name,
            title: raw.title,
            mime_type: raw.mime_type,
            checksum: raw.checksum,
            byte_count: u64::try_from(raw.byte_count).map_err(|_| MediaError::InvalidStoredData)?,
            width: raw.width,
            height: raw.height,
            provenance: if raw.provenance == "imported" {
                MediaProvenance::Imported
            } else {
                return Err(MediaError::InvalidStoredData);
            },
            tags,
            links: self.links(project_id, id)?,
        })
    }

    fn links(
        &self,
        project_id: ProjectId,
        asset_id: EntityId,
    ) -> Result<Vec<AssetLink>, MediaError> {
        let mut statement = self.database.connection().prepare(
            "SELECT target_type, target_id, role FROM asset_link
             WHERE project_id = ?1 AND asset_id = ?2 ORDER BY target_type, target_id, role",
        )?;
        statement
            .query_map(
                params![project_id.to_string(), asset_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )?
            .map(|row| {
                let (kind, id, role) = row?;
                Ok(AssetLink {
                    target_kind: parse_target(&kind)?,
                    target_id: EntityId::parse(&id).ok_or(MediaError::InvalidStoredData)?,
                    role,
                })
            })
            .collect()
    }
}

struct RawAsset {
    id: String,
    project_id: String,
    kind: String,
    relative_path: String,
    thumbnail_path: String,
    original_name: String,
    title: String,
    mime_type: String,
    checksum: String,
    byte_count: i64,
    width: u32,
    height: u32,
    provenance: String,
    tags_json: String,
}

fn raw_asset(row: &Row<'_>) -> rusqlite::Result<RawAsset> {
    Ok(RawAsset {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind: row.get(2)?,
        relative_path: row.get(3)?,
        thumbnail_path: row.get(4)?,
        original_name: row.get(5)?,
        title: row.get(6)?,
        mime_type: row.get(7)?,
        checksum: row.get(8)?,
        byte_count: row.get(9)?,
        width: row.get(10)?,
        height: row.get(11)?,
        provenance: row.get(12)?,
        tags_json: row.get(13)?,
    })
}

fn validate_metadata(title: &str, tags: &[String]) -> Result<String, MediaError> {
    if title.trim() != title
        || title.is_empty()
        || title.chars().count() > 240
        || title.chars().any(char::is_control)
        || tags.len() > MAX_TAGS
    {
        return Err(MediaError::InvalidInput);
    }
    let mut normalized = Vec::with_capacity(tags.len());
    for tag in tags {
        let value = tag.trim();
        if value.is_empty()
            || value.chars().count() > MAX_TAG_CHARS
            || value.chars().any(char::is_control)
            || normalized
                .iter()
                .any(|stored: &String| stored.eq_ignore_ascii_case(value))
        {
            return Err(MediaError::InvalidInput);
        }
        normalized.push(value.to_owned());
    }
    serde_json::to_string(&normalized).map_err(|_| MediaError::InvalidInput)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Der explizite Dateivertrag wird gemeinsam validiert"
)]
fn validate_file_contract(
    relative: &str,
    thumbnail: &str,
    original: &str,
    mime: &str,
    checksum: &str,
    byte_count: u64,
    width: u32,
    height: u32,
) -> Result<(), MediaError> {
    if !relative.starts_with("assets/")
        || relative.starts_with("assets/thumbnails/")
        || !thumbnail.starts_with("assets/thumbnails/")
        || relative.contains(['\\', '\0'])
        || thumbnail.contains(['\\', '\0'])
        || relative
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
        || thumbnail
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
        || original.trim() != original
        || original.is_empty()
        || original.chars().count() > 240
        || !matches!(mime, "image/png" | "image/jpeg" | "image/webp")
        || checksum.len() != 64
        || !checksum
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || !(1..=26_214_400).contains(&byte_count)
        || width == 0
        || height == 0
        || width > 16_384
        || height > 16_384
    {
        return Err(MediaError::InvalidInput);
    }
    Ok(())
}

fn validate_role(role: &str) -> Result<(), MediaError> {
    if role.trim() != role
        || role.is_empty()
        || role.chars().count() > MAX_ROLE_CHARS
        || role.chars().any(char::is_control)
    {
        Err(MediaError::InvalidInput)
    } else {
        Ok(())
    }
}

const fn target_name(kind: AssetTargetKind) -> &'static str {
    match kind {
        AssetTargetKind::Document => "document",
        AssetTargetKind::Entity => "entity",
        AssetTargetKind::Event => "event",
    }
}

fn parse_target(value: &str) -> Result<AssetTargetKind, MediaError> {
    match value {
        "document" => Ok(AssetTargetKind::Document),
        "entity" => Ok(AssetTargetKind::Entity),
        "event" => Ok(AssetTargetKind::Event),
        _ => Err(MediaError::InvalidStoredData),
    }
}

fn require_asset(
    connection: &rusqlite::Connection,
    project: ProjectId,
    id: EntityId,
) -> Result<(), MediaError> {
    connection
        .query_row(
            "SELECT 1 FROM media_asset WHERE project_id = ?1 AND id = ?2",
            params![project.to_string(), id.to_string()],
            |_| Ok(()),
        )
        .optional()?
        .ok_or(MediaError::NotFound)
}

fn require_exists(
    transaction: &Transaction<'_>,
    table: &str,
    project: ProjectId,
    id: EntityId,
) -> Result<(), MediaError> {
    let sql = format!("SELECT 1 FROM {table} WHERE project_id = ?1 AND id = ?2");
    transaction
        .query_row(&sql, params![project.to_string(), id.to_string()], |_| {
            Ok(())
        })
        .optional()?
        .ok_or(MediaError::NotFound)
}

fn require_target(
    transaction: &Transaction<'_>,
    project: ProjectId,
    kind: AssetTargetKind,
    id: EntityId,
) -> Result<(), MediaError> {
    if kind == AssetTargetKind::Document {
        transaction
            .query_row(
                "SELECT 1 FROM document WHERE project_id = ?1 AND id = ?2 AND kind = 'scene'",
                params![project.to_string(), id.to_string()],
                |_| Ok(()),
            )
            .optional()?
            .ok_or(MediaError::NotFound)
    } else {
        require_exists(transaction, target_name(kind), project, id)
    }
}

fn current_timestamp() -> Result<String, MediaError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MediaError::InvalidSystemTime)?;
    Ok(format!(
        "{}.{:09}Z",
        duration.as_secs(),
        duration.subsec_nanos()
    ))
}

#[cfg(test)]
mod tests {
    use kraqel_domain::{AssetTargetKind, CURRENT_SCHEMA_VERSION, EntityId, ProjectId};
    use rusqlite::params;

    use super::MediaRepository;
    use crate::Database;

    #[test]
    fn stores_metadata_and_idempotent_links_with_stable_backing_ids() {
        let mut database = Database::open_in_memory().unwrap();
        let project = ProjectId::new();
        let document = EntityId::new();
        database
            .connection()
            .execute(
                "INSERT INTO project (id, name, schema_version, created_at, updated_at)
             VALUES (?1, 'Test', ?2, 't', 't')",
                params![project.to_string(), CURRENT_SCHEMA_VERSION.0],
            )
            .unwrap();
        database.connection().execute(
            "INSERT INTO document (id, project_id, kind, title, position, created_at, updated_at)
             VALUES (?1, ?2, 'scene', 'Szene', 0, 't', 't')",
            params![document.to_string(), project.to_string()],
        ).unwrap();
        let id = EntityId::new();
        let checksum = "a".repeat(64);
        {
            let mut repository = MediaRepository::new(&mut database);
            repository
                .insert(
                    project,
                    id,
                    &format!("assets/{id}-a.png"),
                    &format!("assets/thumbnails/{id}-a.png"),
                    "bild.png",
                    "Bild",
                    "image/png",
                    &checksum,
                    12,
                    8,
                    6,
                    &["Nacht".to_owned()],
                )
                .unwrap();
            repository
                .link(
                    project,
                    id,
                    AssetTargetKind::Document,
                    document,
                    "reference",
                )
                .unwrap();
            let linked = repository
                .link(
                    project,
                    id,
                    AssetTargetKind::Document,
                    document,
                    "reference",
                )
                .unwrap();
            assert_eq!(linked.links.len(), 1);
            assert_eq!(linked.links[0].target_id, document);
        }
        database
            .connection()
            .execute(
                "DELETE FROM document WHERE project_id = ?1 AND id = ?2",
                params![project.to_string(), document.to_string()],
            )
            .unwrap();
        let link_count: i64 = database
            .connection()
            .query_row("SELECT count(*) FROM asset_link", [], |row| row.get(0))
            .unwrap();
        assert_eq!(link_count, 0);
    }
}
