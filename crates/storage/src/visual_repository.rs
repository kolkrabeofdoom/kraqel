//! Transaktionale Speicherung visueller Profile und projektweiter Stilvorlagen.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{EntityId, EntityKind, ProjectId, StylePreset, VisualProfile};
use rusqlite::{OptionalExtension, params};

use crate::{Database, StorageError};

const MAX_ENTRY_COUNT: usize = 32;
const MAX_KEY_CHARS: usize = 80;
const MAX_VALUE_CHARS: usize = 500;
const MAX_JSON_BYTES: usize = 64 * 1024;
const MAX_PRESET_NAME_CHARS: usize = 120;

/// Fachliche Fehler der visuellen Profilspeicherung.
#[derive(Debug)]
pub enum VisualError {
    InvalidInput,
    NotFound,
    Duplicate,
    InvalidStoredData,
    Storage(StorageError),
    InvalidSystemTime,
}

impl fmt::Display for VisualError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die visuellen Profildaten sind ungültig."),
            Self::NotFound => {
                formatter.write_str("Das visuelle Profilobjekt wurde nicht gefunden.")
            }
            Self::Duplicate => formatter.write_str("Die Stilvorlage ist bereits vorhanden."),
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte visuelle Profildaten sind ungültig.")
            }
            Self::Storage(error) => error.fmt(formatter),
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
        }
    }
}

impl Error for VisualError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::InvalidInput
            | Self::NotFound
            | Self::Duplicate
            | Self::InvalidStoredData
            | Self::InvalidSystemTime => None,
        }
    }
}

impl From<rusqlite::Error> for VisualError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

impl From<StorageError> for VisualError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Repository über einer verwalteten Projektdatenbank.
pub struct VisualRepository<'database> {
    database: &'database mut Database,
}

impl<'database> VisualRepository<'database> {
    pub const fn new(database: &'database mut Database) -> Self {
        Self { database }
    }

    /// Listet alle visuellen Profile eines Projekts stabil nach Entitätskennung.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen gespeicherten Daten oder SQLite-Problemen.
    pub fn list_profiles(&self, project_id: ProjectId) -> Result<Vec<VisualProfile>, VisualError> {
        let mut statement = self.database.connection().prepare(
            "SELECT entity_id, project_id, traits_json, canonical_asset_id, exclusions_json
             FROM visual_profile WHERE project_id = ?1 ORDER BY entity_id",
        )?;
        let rows = statement.query_map([project_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        rows.map(|row| {
            row.map_err(VisualError::from)
                .and_then(|raw| parse_profile(&raw))
        })
        .collect()
    }

    /// Erstellt oder ersetzt das genau eine Profil eines Story-Bibel-Eintrags.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen Eingaben, fehlenden Referenzen oder SQLite-Problemen.
    pub fn upsert_profile(
        &mut self,
        project_id: ProjectId,
        entity_id: EntityId,
        traits: &BTreeMap<String, String>,
        canonical_asset_id: Option<EntityId>,
        exclusions: &[String],
    ) -> Result<VisualProfile, VisualError> {
        let traits = normalize_map(traits)?;
        let exclusions = normalize_list(exclusions)?;
        let traits_json = serialize_limited(&traits)?;
        let exclusions_json = serialize_limited(&exclusions)?;
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        require_entity(&transaction, project_id, entity_id)?;
        if let Some(asset_id) = canonical_asset_id {
            require_asset(&transaction, project_id, asset_id)?;
        }
        transaction.execute(
            "INSERT INTO visual_profile (
                entity_id, project_id, traits_json, canonical_asset_id, exclusions_json,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(entity_id) DO UPDATE SET
                traits_json = excluded.traits_json,
                canonical_asset_id = excluded.canonical_asset_id,
                exclusions_json = excluded.exclusions_json,
                updated_at = excluded.updated_at",
            params![
                entity_id.to_string(),
                project_id.to_string(),
                traits_json,
                canonical_asset_id.map(|id| id.to_string()),
                exclusions_json,
                timestamp
            ],
        )?;
        transaction.commit()?;
        Ok(VisualProfile {
            entity_id,
            project_id,
            traits,
            canonical_asset_id,
            exclusions,
        })
    }

    /// Löscht ein Profil, ohne Entität oder Asset zu verändern.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlendem Profil oder SQLite-Problemen.
    pub fn delete_profile(
        &mut self,
        project_id: ProjectId,
        entity_id: EntityId,
    ) -> Result<(), VisualError> {
        let changed = self.database.connection().execute(
            "DELETE FROM visual_profile WHERE project_id = ?1 AND entity_id = ?2",
            params![project_id.to_string(), entity_id.to_string()],
        )?;
        if changed == 0 {
            return Err(VisualError::NotFound);
        }
        Ok(())
    }

    /// Listet Stilvorlagen stabil nach Name und Kennung.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen gespeicherten Daten oder SQLite-Problemen.
    pub fn list_presets(&self, project_id: ProjectId) -> Result<Vec<StylePreset>, VisualError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, project_id, name, visual_rules_json, applicable_kinds_json
             FROM style_preset WHERE project_id = ?1 ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([project_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        rows.map(|row| {
            row.map_err(VisualError::from)
                .and_then(|raw| parse_preset(&raw))
        })
        .collect()
    }

    /// Erstellt eine projektweite Stilvorlage.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen oder doppelten Eingaben und SQLite-Problemen.
    pub fn create_preset(
        &mut self,
        project_id: ProjectId,
        name: &str,
        visual_rules: &BTreeMap<String, String>,
        applicable_kinds: &[EntityKind],
    ) -> Result<StylePreset, VisualError> {
        let id = EntityId::new();
        self.write_preset(id, project_id, name, visual_rules, applicable_kinds, true)
    }

    /// Aktualisiert eine vorhandene Stilvorlage.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültigen oder doppelten Eingaben, fehlender Vorlage oder
    /// SQLite-Problemen.
    pub fn update_preset(
        &mut self,
        project_id: ProjectId,
        preset_id: EntityId,
        name: &str,
        visual_rules: &BTreeMap<String, String>,
        applicable_kinds: &[EntityKind],
    ) -> Result<StylePreset, VisualError> {
        self.write_preset(
            preset_id,
            project_id,
            name,
            visual_rules,
            applicable_kinds,
            false,
        )
    }

    /// Löscht eine Stilvorlage.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlender Vorlage oder SQLite-Problemen.
    pub fn delete_preset(
        &mut self,
        project_id: ProjectId,
        preset_id: EntityId,
    ) -> Result<(), VisualError> {
        let changed = self.database.connection().execute(
            "DELETE FROM style_preset WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), preset_id.to_string()],
        )?;
        if changed == 0 {
            return Err(VisualError::NotFound);
        }
        Ok(())
    }

    fn write_preset(
        &mut self,
        id: EntityId,
        project_id: ProjectId,
        name: &str,
        visual_rules: &BTreeMap<String, String>,
        applicable_kinds: &[EntityKind],
        create: bool,
    ) -> Result<StylePreset, VisualError> {
        let name = normalize_name(name)?;
        let visual_rules = normalize_map(visual_rules)?;
        if visual_rules.is_empty() {
            return Err(VisualError::InvalidInput);
        }
        let applicable_kinds = normalize_kinds(applicable_kinds)?;
        let rules_json = serialize_limited(&visual_rules)?;
        let kinds_json = serde_json::to_string(
            &applicable_kinds
                .iter()
                .copied()
                .map(kind_name)
                .collect::<Vec<_>>(),
        )
        .map_err(|_| VisualError::InvalidInput)?;
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        require_project(&transaction, project_id)?;
        let duplicate: bool = transaction
            .query_row(
                "SELECT 1 FROM style_preset
                 WHERE project_id = ?1 AND name = ?2 COLLATE NOCASE AND id <> ?3",
                params![project_id.to_string(), name, id.to_string()],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if duplicate {
            return Err(VisualError::Duplicate);
        }
        if create {
            transaction.execute(
                "INSERT INTO style_preset (
                    id, project_id, name, visual_rules_json, applicable_kinds_json,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    id.to_string(),
                    project_id.to_string(),
                    name,
                    rules_json,
                    kinds_json,
                    timestamp
                ],
            )?;
        } else {
            let changed = transaction.execute(
                "UPDATE style_preset SET name = ?3, visual_rules_json = ?4,
                    applicable_kinds_json = ?5, updated_at = ?6
                 WHERE project_id = ?1 AND id = ?2",
                params![
                    project_id.to_string(),
                    id.to_string(),
                    name,
                    rules_json,
                    kinds_json,
                    timestamp
                ],
            )?;
            if changed == 0 {
                return Err(VisualError::NotFound);
            }
        }
        transaction.commit()?;
        Ok(StylePreset {
            id,
            project_id,
            name,
            visual_rules,
            applicable_kinds,
        })
    }
}

fn normalize_name(value: &str) -> Result<String, VisualError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_PRESET_NAME_CHARS {
        return Err(VisualError::InvalidInput);
    }
    Ok(trimmed.to_owned())
}

fn normalize_map(
    values: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, VisualError> {
    if values.len() > MAX_ENTRY_COUNT {
        return Err(VisualError::InvalidInput);
    }
    let mut normalized = BTreeMap::new();
    for (key, value) in values {
        let key = key.trim();
        let value = value.trim();
        if key.is_empty()
            || value.is_empty()
            || key.chars().count() > MAX_KEY_CHARS
            || value.chars().count() > MAX_VALUE_CHARS
            || normalized
                .insert(key.to_owned(), value.to_owned())
                .is_some()
        {
            return Err(VisualError::InvalidInput);
        }
    }
    Ok(normalized)
}

fn normalize_list(values: &[String]) -> Result<Vec<String>, VisualError> {
    if values.len() > MAX_ENTRY_COUNT {
        return Err(VisualError::InvalidInput);
    }
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > MAX_VALUE_CHARS
            || !seen.insert(value.to_owned())
        {
            return Err(VisualError::InvalidInput);
        }
        normalized.push(value.to_owned());
    }
    Ok(normalized)
}

fn normalize_kinds(values: &[EntityKind]) -> Result<Vec<EntityKind>, VisualError> {
    let mut result = Vec::new();
    for candidate in [
        EntityKind::Character,
        EntityKind::Location,
        EntityKind::Faction,
        EntityKind::Item,
    ] {
        if values.contains(&candidate) {
            result.push(candidate);
        }
    }
    if result.is_empty() || result.len() != values.len() {
        return Err(VisualError::InvalidInput);
    }
    Ok(result)
}

fn serialize_limited<T: serde::Serialize>(value: &T) -> Result<String, VisualError> {
    let json = serde_json::to_string(value).map_err(|_| VisualError::InvalidInput)?;
    if json.len() > MAX_JSON_BYTES {
        return Err(VisualError::InvalidInput);
    }
    Ok(json)
}

fn parse_profile(
    raw: &(String, String, String, Option<String>, String),
) -> Result<VisualProfile, VisualError> {
    let traits: BTreeMap<String, String> =
        serde_json::from_str(&raw.2).map_err(|_| VisualError::InvalidStoredData)?;
    let exclusions: Vec<String> =
        serde_json::from_str(&raw.4).map_err(|_| VisualError::InvalidStoredData)?;
    Ok(VisualProfile {
        entity_id: parse_entity_id(&raw.0)?,
        project_id: parse_project_id(&raw.1)?,
        traits: normalize_map(&traits).map_err(|_| VisualError::InvalidStoredData)?,
        canonical_asset_id: raw.3.as_deref().map(parse_entity_id).transpose()?,
        exclusions: normalize_list(&exclusions).map_err(|_| VisualError::InvalidStoredData)?,
    })
}

fn parse_preset(
    raw: &(String, String, String, String, String),
) -> Result<StylePreset, VisualError> {
    let rules: BTreeMap<String, String> =
        serde_json::from_str(&raw.3).map_err(|_| VisualError::InvalidStoredData)?;
    let stored_kinds: Vec<String> =
        serde_json::from_str(&raw.4).map_err(|_| VisualError::InvalidStoredData)?;
    let kinds = stored_kinds
        .iter()
        .map(|value| parse_kind(value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StylePreset {
        id: parse_entity_id(&raw.0)?,
        project_id: parse_project_id(&raw.1)?,
        name: normalize_name(&raw.2).map_err(|_| VisualError::InvalidStoredData)?,
        visual_rules: normalize_map(&rules).map_err(|_| VisualError::InvalidStoredData)?,
        applicable_kinds: normalize_kinds(&kinds).map_err(|_| VisualError::InvalidStoredData)?,
    })
}

fn parse_entity_id(value: &str) -> Result<EntityId, VisualError> {
    EntityId::parse(value).ok_or(VisualError::InvalidStoredData)
}

fn parse_project_id(value: &str) -> Result<ProjectId, VisualError> {
    ProjectId::parse(value).ok_or(VisualError::InvalidStoredData)
}

const fn kind_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "character",
        EntityKind::Location => "location",
        EntityKind::Faction => "faction",
        EntityKind::Item => "item",
    }
}

fn parse_kind(value: &str) -> Result<EntityKind, VisualError> {
    match value {
        "character" => Ok(EntityKind::Character),
        "location" => Ok(EntityKind::Location),
        "faction" => Ok(EntityKind::Faction),
        "item" => Ok(EntityKind::Item),
        _ => Err(VisualError::InvalidStoredData),
    }
}

fn require_project(
    transaction: &rusqlite::Transaction<'_>,
    project_id: ProjectId,
) -> Result<(), VisualError> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM project WHERE id = ?1",
            [project_id.to_string()],
            |_| Ok(()),
        )
        .optional()?;
    exists.ok_or(VisualError::NotFound)
}

fn require_entity(
    transaction: &rusqlite::Transaction<'_>,
    project_id: ProjectId,
    entity_id: EntityId,
) -> Result<(), VisualError> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM entity WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), entity_id.to_string()],
            |_| Ok(()),
        )
        .optional()?;
    exists.ok_or(VisualError::NotFound)
}

fn require_asset(
    transaction: &rusqlite::Transaction<'_>,
    project_id: ProjectId,
    asset_id: EntityId,
) -> Result<(), VisualError> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM media_asset WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), asset_id.to_string()],
            |_| Ok(()),
        )
        .optional()?;
    exists.ok_or(VisualError::NotFound)
}

fn current_timestamp() -> Result<String, VisualError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| VisualError::InvalidSystemTime)?
        .as_secs();
    Ok(seconds.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kraqel_domain::{EntityId, EntityKind, ProjectId};
    use rusqlite::params;

    use super::{VisualError, VisualRepository};
    use crate::Database;

    const PROJECT: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2";
    const OTHER_PROJECT: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3";
    const ENTITY: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4";
    const ASSET: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a5";

    fn database() -> Database {
        let database = Database::open_in_memory().unwrap();
        for (id, name) in [(PROJECT, "Morgenrot"), (OTHER_PROJECT, "Abendrot")] {
            database
                .connection()
                .execute(
                    "INSERT INTO project (id, name, schema_version, created_at, updated_at)
                 VALUES (?1, ?2, 1, '0', '0')",
                    params![id, name],
                )
                .unwrap();
        }
        database
            .connection()
            .execute(
                "INSERT INTO entity (id, project_id, kind, name, created_at, updated_at)
             VALUES (?1, ?2, 'character', 'Rhea', '0', '0')",
                params![ENTITY, PROJECT],
            )
            .unwrap();
        database
            .connection()
            .execute(
                "INSERT INTO media_asset (
                id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                mime_type, checksum, byte_count, width, height, provenance, created_at, updated_at
             ) VALUES (?1, ?2, 'image', 'assets/rhea.png', 'assets/thumbnails/rhea.png',
                'rhea.png', 'Rhea', 'image/png', ?3, 1, 1, 1, 'imported', '0', '0')",
                params![ASSET, PROJECT, "a".repeat(64)],
            )
            .unwrap();
        database
    }

    #[test]
    fn profile_and_preset_crud_are_stable_and_project_bound() {
        let mut database = database();
        let project = ProjectId::parse(PROJECT).unwrap();
        let entity = EntityId::parse(ENTITY).unwrap();
        let asset = EntityId::parse(ASSET).unwrap();
        let mut traits = BTreeMap::new();
        traits.insert("Augen".to_owned(), "bernsteinfarben".to_owned());
        let profile = VisualRepository::new(&mut database)
            .upsert_profile(
                project,
                entity,
                &traits,
                Some(asset),
                &["blaue Augen".to_owned()],
            )
            .unwrap();
        assert_eq!(profile.canonical_asset_id, Some(asset));

        let mut rules = BTreeMap::new();
        rules.insert("Palette".to_owned(), "Ocker und Nachtblau".to_owned());
        let preset = VisualRepository::new(&mut database)
            .create_preset(project, "Graphic Novel", &rules, &[EntityKind::Character])
            .unwrap();
        assert_eq!(
            VisualRepository::new(&mut database)
                .list_profiles(project)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            VisualRepository::new(&mut database)
                .list_presets(project)
                .unwrap(),
            vec![preset.clone()]
        );

        VisualRepository::new(&mut database)
            .delete_preset(project, preset.id)
            .unwrap();
        VisualRepository::new(&mut database)
            .delete_profile(project, entity)
            .unwrap();
        assert_eq!(
            VisualRepository::new(&mut database)
                .list_profiles(project)
                .unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn canonical_asset_is_cleared_and_entity_delete_cascades() {
        let mut database = database();
        let project = ProjectId::parse(PROJECT).unwrap();
        let entity = EntityId::parse(ENTITY).unwrap();
        let asset = EntityId::parse(ASSET).unwrap();
        VisualRepository::new(&mut database)
            .upsert_profile(project, entity, &BTreeMap::new(), Some(asset), &[])
            .unwrap();
        database
            .connection()
            .execute("DELETE FROM media_asset WHERE id = ?1", [ASSET])
            .unwrap();
        let profiles = VisualRepository::new(&mut database)
            .list_profiles(project)
            .unwrap();
        assert_eq!(profiles[0].canonical_asset_id, None);
        database
            .connection()
            .execute("DELETE FROM entity WHERE id = ?1", [ENTITY])
            .unwrap();
        assert_eq!(
            VisualRepository::new(&mut database)
                .list_profiles(project)
                .unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn rejects_invalid_and_duplicate_values() {
        let mut database = database();
        let project = ProjectId::parse(PROJECT).unwrap();
        let entity = EntityId::parse(ENTITY).unwrap();
        let missing_asset = EntityId::new();
        assert!(matches!(
            VisualRepository::new(&mut database).upsert_profile(
                project,
                entity,
                &BTreeMap::new(),
                Some(missing_asset),
                &[]
            ),
            Err(VisualError::NotFound)
        ));
        assert!(matches!(
            VisualRepository::new(&mut database).create_preset(
                project,
                "Leer",
                &BTreeMap::new(),
                &[EntityKind::Character]
            ),
            Err(VisualError::InvalidInput)
        ));
        let rules = BTreeMap::from([("Palette".to_owned(), "Monochrom".to_owned())]);
        VisualRepository::new(&mut database)
            .create_preset(project, "Noir", &rules, &[EntityKind::Character])
            .unwrap();
        assert!(matches!(
            VisualRepository::new(&mut database).create_preset(
                project,
                " noir ",
                &rules,
                &[EntityKind::Location]
            ),
            Err(VisualError::Duplicate)
        ));
    }
}
