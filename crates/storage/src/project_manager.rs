//! Sicherer Lebenszyklus lokaler Projektverzeichnisse.

use std::{
    error::Error,
    ffi::OsString,
    fmt, fs,
    io::{self, Write as _},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project, is_valid_project_name};
use rusqlite::{Connection, MAIN_DB, OpenFlags, params};
use sha2::{Digest as _, Sha256};

use crate::{CURRENT_DATABASE_VERSION, Database, StorageError};

/// Name der autoritativen Datenbank innerhalb eines Projektverzeichnisses.
pub const DATABASE_FILE_NAME: &str = "kraqel.sqlite3";
/// Verzeichnis für konsistente lokale Datenbanksicherungen.
pub const BACKUP_DIRECTORY_NAME: &str = "backups";
const BACKUP_ASSET_SUFFIX: &str = ".assets";

static TEMPORARY_NAME_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Kompakte Metadaten eines verwalteten Projekts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOverview {
    /// Stabile Projektkennung und zugleich sicherer Verzeichnisname.
    pub id: kraqel_domain::ProjectId,
    /// Anzeigename des Projekts.
    pub name: String,
}

/// Fehler des lokalen Projektlebenszyklus.
#[derive(Debug)]
pub enum ProjectError {
    /// Ein Pfad ist nicht relativ, leer oder enthält unsichere Komponenten.
    InvalidRelativePath,
    /// Eine Pfadkomponente ist ein symbolischer Link.
    SymlinkNotAllowed,
    /// Ein kanonischer Pfad liegt außerhalb der erlaubten Projektwurzel.
    OutsideProjectRoot,
    /// Das angeforderte Projekt existiert bereits.
    ProjectAlreadyExists,
    /// Das angeforderte Projektverzeichnis fehlt.
    ProjectMissing,
    /// Die erwartete Projektdatenbank fehlt.
    DatabaseMissing,
    /// Die angeforderte Sicherung existiert bereits.
    BackupAlreadyExists,
    /// Die angeforderte Sicherung fehlt.
    BackupMissing,
    /// Projektkennung, Name oder Schema-Version sind ungültig.
    InvalidProjectMetadata,
    /// Ein externer Projektstand konnte nicht sicher initialisiert werden.
    InvalidImportedData,
    /// Die Datenbank oder Sicherung ist beschädigt beziehungsweise nicht lesbar.
    InvalidDatabase,
    /// Die Systemzeit liegt vor der Unix-Epoche.
    InvalidSystemTime,
    /// Eine Dateisystemoperation ist fehlgeschlagen.
    Io {
        /// Datenarme Beschreibung der fehlgeschlagenen Operation.
        operation: &'static str,
        /// Technische Ursache ohne zusätzlich gespeicherten Projektpfad.
        source: io::Error,
    },
    /// Die SQLite-Speicherschicht hat die Operation abgelehnt.
    Storage(StorageError),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRelativePath => formatter.write_str("Der Projektpfad ist ungültig."),
            Self::SymlinkNotAllowed => {
                formatter.write_str("Symbolische Links sind in Projektpfaden nicht erlaubt.")
            }
            Self::OutsideProjectRoot => {
                formatter.write_str("Der Pfad liegt außerhalb der erlaubten Projektwurzel.")
            }
            Self::ProjectAlreadyExists => formatter.write_str("Das Projekt existiert bereits."),
            Self::ProjectMissing => {
                formatter.write_str("Das Projektverzeichnis wurde nicht gefunden.")
            }
            Self::DatabaseMissing => {
                formatter.write_str("Die Projektdatenbank wurde nicht gefunden.")
            }
            Self::BackupAlreadyExists => formatter.write_str("Die Sicherung existiert bereits."),
            Self::BackupMissing => formatter.write_str("Die Sicherung wurde nicht gefunden."),
            Self::InvalidProjectMetadata => formatter.write_str("Die Projektdaten sind ungültig."),
            Self::InvalidImportedData => {
                formatter.write_str("Die importierten Projektdaten sind ungültig.")
            }
            Self::InvalidDatabase => {
                formatter.write_str("Die Datenbank ist beschädigt oder nicht lesbar.")
            }
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
            Self::Io { operation, .. } => {
                write!(
                    formatter,
                    "Dateisystemoperation fehlgeschlagen: {operation}."
                )
            }
            Self::Storage(error) => error.fmt(formatter),
        }
    }
}

impl Error for ProjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Storage(error) => Some(error),
            Self::InvalidRelativePath
            | Self::SymlinkNotAllowed
            | Self::OutsideProjectRoot
            | Self::ProjectAlreadyExists
            | Self::ProjectMissing
            | Self::DatabaseMissing
            | Self::BackupAlreadyExists
            | Self::BackupMissing
            | Self::InvalidProjectMetadata
            | Self::InvalidImportedData
            | Self::InvalidDatabase
            | Self::InvalidSystemTime => None,
        }
    }
}

impl From<StorageError> for ProjectError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<rusqlite::Error> for ProjectError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

/// Geöffnetes lokales Projekt samt autoritativer Datenbank.
pub struct ManagedProject {
    directory: PathBuf,
    database: Database,
}

impl ManagedProject {
    /// Liefert den kanonischen Projektordner.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Liefert die verwaltete Datenbank.
    #[must_use]
    pub const fn database(&self) -> &Database {
        &self.database
    }

    /// Liefert die verwaltete Datenbank veränderlich für transaktionale Schreibvorgänge.
    #[must_use]
    pub const fn database_mut(&mut self) -> &mut Database {
        &mut self.database
    }
}

/// Verwaltet Projekte ausschließlich unter einer kanonischen lokalen Wurzel.
pub struct ProjectManager {
    root: PathBuf,
}

impl ProjectManager {
    /// Bindet den Manager an eine bereits vorhandene, echte Verzeichniswurzel.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn die Wurzel fehlt, kein Verzeichnis oder selbst ein Symlink ist.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let root = root.as_ref();
        let metadata = fs::symlink_metadata(root).map_err(|source| ProjectError::Io {
            operation: "Projektwurzel lesen",
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ProjectError::SymlinkNotAllowed);
        }
        if !metadata.is_dir() {
            return Err(ProjectError::InvalidRelativePath);
        }

        let root = fs::canonicalize(root).map_err(|source| ProjectError::Io {
            operation: "Projektwurzel kanonisieren",
            source,
        })?;
        Ok(Self { root })
    }

    /// Liefert die kanonische Projektwurzel.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Listet alle unmittelbar verwalteten Projekte stabil nach Name und UUID.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn ein als Projekt benanntes Verzeichnis unsicher, beschädigt oder
    /// nicht lesbar ist.
    pub fn list_projects(&self) -> Result<Vec<ProjectOverview>, ProjectError> {
        let entries = fs::read_dir(&self.root).map_err(|source| ProjectError::Io {
            operation: "Projektwurzel auflisten",
            source,
        })?;
        let mut projects = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|source| ProjectError::Io {
                operation: "Projekteintrag lesen",
                source,
            })?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Some(directory_id) = kraqel_domain::ProjectId::parse(file_name) else {
                continue;
            };
            let opened = self.open_project(Path::new(file_name))?;
            let (stored_id, name): (String, String) = opened.database().connection().query_row(
                "SELECT id, name FROM project",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let stored_id =
                kraqel_domain::ProjectId::parse(&stored_id).ok_or(ProjectError::InvalidDatabase)?;
            if stored_id != directory_id {
                return Err(ProjectError::InvalidDatabase);
            }
            projects.push(ProjectOverview {
                id: stored_id,
                name,
            });
        }

        projects.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.to_string().cmp(&right.id.to_string()))
        });
        Ok(projects)
    }

    /// Erstellt ein Projekt vollständig in einem temporären Geschwisterverzeichnis und
    /// veröffentlicht es anschließend atomar.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei unsicheren Pfaden, ungültigen Projektdaten, vorhandenen Zielen oder
    /// fehlgeschlagenen Dateisystem- beziehungsweise SQLite-Operationen.
    pub fn create_project(
        &self,
        relative_path: impl AsRef<Path>,
        project: &Project,
    ) -> Result<ManagedProject, ProjectError> {
        self.create_project_with_initializer(relative_path, project, |_, _| Ok(()))
    }

    /// Erstellt und befüllt ein Projekt vollständig im temporären Geschwisterverzeichnis.
    ///
    /// Der Initialisierer erhält die bereits migrierte Datenbank und den noch unveröffentlichten
    /// Projektordner. Schlägt er fehl, entfernt der Manager den gesamten Zwischenstand; das Ziel
    /// wird erst nach erfolgreicher Initialisierung atomar sichtbar.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei ungültigen Projektdaten, unsicheren Pfaden, fehlgeschlagener
    /// Initialisierung oder Veröffentlichung.
    pub fn create_project_with_initializer<F>(
        &self,
        relative_path: impl AsRef<Path>,
        project: &Project,
        initializer: F,
    ) -> Result<ManagedProject, ProjectError>
    where
        F: FnOnce(&mut Database, &Path) -> Result<(), ProjectError>,
    {
        if !is_valid_project_name(&project.name) || project.schema_version != CURRENT_SCHEMA_VERSION
        {
            return Err(ProjectError::InvalidProjectMetadata);
        }

        let (parent, destination) = self.prepare_new_destination(relative_path.as_ref())?;
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(ProjectError::ProjectAlreadyExists);
        }

        let mut staging = TemporaryDirectory::create(&parent)?;
        fs::create_dir(staging.path().join(BACKUP_DIRECTORY_NAME)).map_err(|source| {
            ProjectError::Io {
                operation: "Sicherungsverzeichnis anlegen",
                source,
            }
        })?;

        let database_path = staging.path().join(DATABASE_FILE_NAME);
        let mut database = Database::open(&database_path)?;
        let timestamp = current_timestamp()?;
        let transaction = database.transaction()?;
        transaction.execute(
            "INSERT INTO project (id, name, schema_version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![
                project.id.to_string(),
                project.name,
                project.schema_version.0,
                timestamp
            ],
        )?;
        transaction.commit()?;
        initializer(&mut database, staging.path())?;
        drop(database);

        fs::rename(staging.path(), &destination).map_err(|source| ProjectError::Io {
            operation: "Projekt atomar veröffentlichen",
            source,
        })?;
        staging.keep();
        self.open_project(relative_path)
    }

    /// Öffnet ein vorhandenes Projekt, ohne fehlende Datenbanken still anzulegen.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei unsicheren Pfaden, fehlenden Dateien oder ungültigen Datenbanken.
    pub fn open_project(
        &self,
        relative_path: impl AsRef<Path>,
    ) -> Result<ManagedProject, ProjectError> {
        let directory =
            self.resolve_existing_directory(relative_path.as_ref(), ProjectError::ProjectMissing)?;
        let database_path = self.resolve_regular_child(
            &directory,
            DATABASE_FILE_NAME,
            ProjectError::DatabaseMissing,
        )?;
        let database = Database::open(&database_path).map_err(map_open_error)?;
        verify_connection(database.connection())?;
        Ok(ManagedProject {
            directory,
            database,
        })
    }

    /// Löscht genau ein über seine UUID adressiertes, geprüftes Projektverzeichnis.
    ///
    /// Vor der destruktiven Operation müssen Verzeichnisname und gespeicherte Projektkennung
    /// übereinstimmen. Symbolische Links und Pfade außerhalb der verwalteten Wurzel werden durch
    /// denselben Auflösungspfad wie beim Öffnen abgelehnt.
    ///
    /// # Errors
    /// Liefert einen Fehler, wenn das Projekt fehlt, unsicher oder beschädigt ist oder das
    /// Verzeichnis nicht vollständig entfernt werden kann.
    pub fn delete_project(&self, project_id: kraqel_domain::ProjectId) -> Result<(), ProjectError> {
        let opened = self.open_project(project_id.to_string())?;
        let stored_id: String =
            opened
                .database()
                .connection()
                .query_row("SELECT id FROM project", [], |row| row.get(0))?;
        if kraqel_domain::ProjectId::parse(&stored_id) != Some(project_id) {
            return Err(ProjectError::InvalidProjectMetadata);
        }
        let directory = opened.directory().to_path_buf();
        drop(opened);
        fs::remove_dir_all(directory).map_err(|source| ProjectError::Io {
            operation: "Projekt löschen",
            source,
        })
    }

    /// Erzeugt eine konsistente Sicherung und veröffentlicht sie atomar unter `backups/`.
    ///
    /// `backup_name` ist ein einzelner Dateiname ohne Erweiterung. Eine vorhandene Sicherung wird
    /// niemals überschrieben.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei unsicheren Namen, fehlenden Projekten, vorhandenen Zielen oder
    /// fehlgeschlagenen SQLite- und Dateisystemoperationen.
    pub fn backup_project(
        &self,
        relative_path: impl AsRef<Path>,
        backup_name: &str,
    ) -> Result<PathBuf, ProjectError> {
        validate_single_name(backup_name)?;
        let project = self.open_project(relative_path)?;
        let backup_directory = self.resolve_existing_directory_from(
            project.directory(),
            Path::new(BACKUP_DIRECTORY_NAME),
            ProjectError::BackupMissing,
        )?;
        let destination = backup_directory.join(format!("{backup_name}.sqlite3"));
        let asset_destination =
            backup_directory.join(format!("{backup_name}{BACKUP_ASSET_SUFFIX}"));
        if fs::symlink_metadata(&destination).is_ok()
            || fs::symlink_metadata(&asset_destination).is_ok()
        {
            return Err(ProjectError::BackupAlreadyExists);
        }

        let temporary = TemporaryFile::reserve(&backup_directory)?;
        project
            .database()
            .connection()
            .backup(MAIN_DB, temporary.path(), None)?;
        verify_database(temporary.path())?;
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(temporary.path())
            .and_then(|file| file.sync_all())
            .map_err(|source| ProjectError::Io {
                operation: "Sicherung synchronisieren",
                source,
            })?;

        let mut asset_staging = TemporaryDirectory::create(&backup_directory)?;
        copy_assets_from_project(temporary.path(), project.directory(), asset_staging.path())?;
        fs::rename(asset_staging.path(), &asset_destination).map_err(|source| {
            ProjectError::Io {
                operation: "Asset-Sicherung atomar veröffentlichen",
                source,
            }
        })?;
        asset_staging.keep();

        if let Err(source) = fs::hard_link(temporary.path(), &destination) {
            let _ = fs::remove_dir_all(&asset_destination);
            return Err(if source.kind() == io::ErrorKind::AlreadyExists {
                ProjectError::BackupAlreadyExists
            } else {
                ProjectError::Io {
                    operation: "Sicherung atomar veröffentlichen",
                    source,
                }
            });
        }
        Ok(destination)
    }

    /// Stellt eine Sicherung in einem neuen Projektverzeichnis wieder her.
    ///
    /// Quellprojekt und Sicherungsdatei werden ausschließlich lesend geöffnet. Das Ziel darf noch
    /// nicht existieren und wird erst nach erfolgreicher Integritätsprüfung atomar veröffentlicht.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei fehlenden oder beschädigten Sicherungen, unsicheren Pfaden,
    /// vorhandenen Zielen oder fehlgeschlagenen SQLite- und Dateisystemoperationen.
    pub fn restore_backup(
        &self,
        source_project: impl AsRef<Path>,
        backup_name: &str,
        destination_project: impl AsRef<Path>,
    ) -> Result<ManagedProject, ProjectError> {
        validate_single_name(backup_name)?;
        let source_directory =
            self.resolve_existing_directory(source_project.as_ref(), ProjectError::ProjectMissing)?;
        let backup_directory = self.resolve_existing_directory_from(
            &source_directory,
            Path::new(BACKUP_DIRECTORY_NAME),
            ProjectError::BackupMissing,
        )?;
        let backup_path = self.resolve_regular_child(
            &backup_directory,
            &format!("{backup_name}.sqlite3"),
            ProjectError::BackupMissing,
        )?;
        verify_restore_source(&backup_path)?;

        let (parent, destination) = self.prepare_new_destination(destination_project.as_ref())?;
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(ProjectError::ProjectAlreadyExists);
        }

        let mut staging = TemporaryDirectory::create(&parent)?;
        fs::create_dir(staging.path().join(BACKUP_DIRECTORY_NAME)).map_err(|source| {
            ProjectError::Io {
                operation: "Sicherungsverzeichnis anlegen",
                source,
            }
        })?;
        let restored_database = staging.path().join(DATABASE_FILE_NAME);
        let source = open_read_only(&backup_path)?;
        source.backup(MAIN_DB, &restored_database, None)?;
        drop(source);
        {
            let migrated = Database::open(&restored_database).map_err(map_open_error)?;
            migrated
                .connection()
                .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        verify_database(&restored_database)?;
        let backup_assets = backup_directory.join(format!("{backup_name}{BACKUP_ASSET_SUFFIX}"));
        restore_assets_into_project(&restored_database, &backup_assets, staging.path())?;

        fs::rename(staging.path(), &destination).map_err(|source| ProjectError::Io {
            operation: "Wiederherstellung atomar veröffentlichen",
            source,
        })?;
        staging.keep();
        self.open_project(destination_project)
    }

    fn prepare_new_destination(&self, relative: &Path) -> Result<(PathBuf, PathBuf), ProjectError> {
        let components = relative_components(relative)?;
        let (name, parents) = components
            .split_last()
            .ok_or(ProjectError::InvalidRelativePath)?;
        let mut parent = self.root.clone();

        for component in parents {
            let candidate = parent.join(component);
            match fs::symlink_metadata(&candidate) {
                Ok(metadata) => validate_directory_metadata(&metadata)?,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir(&candidate).map_err(|source| ProjectError::Io {
                        operation: "Projektpfad anlegen",
                        source,
                    })?;
                    let metadata =
                        fs::symlink_metadata(&candidate).map_err(|source| ProjectError::Io {
                            operation: "Projektpfad prüfen",
                            source,
                        })?;
                    validate_directory_metadata(&metadata)?;
                }
                Err(source) => {
                    return Err(ProjectError::Io {
                        operation: "Projektpfad prüfen",
                        source,
                    });
                }
            }
            parent = canonical_within(&self.root, &candidate)?;
        }

        Ok((parent.clone(), parent.join(name)))
    }

    fn resolve_existing_directory(
        &self,
        relative: &Path,
        missing: ProjectError,
    ) -> Result<PathBuf, ProjectError> {
        self.resolve_existing_directory_from(&self.root, relative, missing)
    }

    fn resolve_existing_directory_from(
        &self,
        base: &Path,
        relative: &Path,
        missing: ProjectError,
    ) -> Result<PathBuf, ProjectError> {
        let components = relative_components(relative)?;
        let mut current = base.to_path_buf();
        for component in components {
            let candidate = current.join(component);
            let metadata = match fs::symlink_metadata(&candidate) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(missing),
                Err(source) => {
                    return Err(ProjectError::Io {
                        operation: "Projektpfad prüfen",
                        source,
                    });
                }
            };
            validate_directory_metadata(&metadata)?;
            current = canonical_within(&self.root, &candidate)?;
        }
        Ok(current)
    }

    fn resolve_regular_child(
        &self,
        directory: &Path,
        name: &str,
        missing: ProjectError,
    ) -> Result<PathBuf, ProjectError> {
        validate_single_name(name)?;
        let candidate = directory.join(name);
        let metadata = match fs::symlink_metadata(&candidate) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(missing),
            Err(source) => {
                return Err(ProjectError::Io {
                    operation: "Projektdatei prüfen",
                    source,
                });
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(ProjectError::SymlinkNotAllowed);
        }
        if !metadata.is_file() {
            return Err(ProjectError::InvalidDatabase);
        }
        canonical_within(&self.root, &candidate)
    }
}

fn map_open_error(error: StorageError) -> ProjectError {
    match error {
        StorageError::Sqlite(_) => ProjectError::InvalidDatabase,
        other => ProjectError::Storage(other),
    }
}

fn current_timestamp() -> Result<String, ProjectError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ProjectError::InvalidSystemTime)?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}

fn relative_components(path: &Path) -> Result<Vec<OsString>, ProjectError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(ProjectError::InvalidRelativePath);
    }

    path.components()
        .map(|component| match component {
            Component::Normal(value) => Ok(value.to_os_string()),
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => Err(ProjectError::InvalidRelativePath),
        })
        .collect()
}

fn validate_single_name(name: &str) -> Result<(), ProjectError> {
    let components = relative_components(Path::new(name))?;
    if components.len() == 1 {
        Ok(())
    } else {
        Err(ProjectError::InvalidRelativePath)
    }
}

fn validate_directory_metadata(metadata: &fs::Metadata) -> Result<(), ProjectError> {
    if metadata.file_type().is_symlink() {
        return Err(ProjectError::SymlinkNotAllowed);
    }
    if metadata.is_dir() {
        Ok(())
    } else {
        Err(ProjectError::InvalidRelativePath)
    }
}

fn canonical_within(root: &Path, candidate: &Path) -> Result<PathBuf, ProjectError> {
    let canonical = fs::canonicalize(candidate).map_err(|source| ProjectError::Io {
        operation: "Projektpfad kanonisieren",
        source,
    })?;
    if canonical.starts_with(root) {
        Ok(canonical)
    } else {
        Err(ProjectError::OutsideProjectRoot)
    }
}

fn open_read_only(path: &Path) -> Result<Connection, ProjectError> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| ProjectError::InvalidDatabase)
}

fn copy_assets_from_project(
    database_path: &Path,
    project_directory: &Path,
    backup_root: &Path,
) -> Result<(), ProjectError> {
    copy_referenced_assets(database_path, project_directory, backup_root)
}

fn restore_assets_into_project(
    database_path: &Path,
    backup_root: &Path,
    project_directory: &Path,
) -> Result<(), ProjectError> {
    copy_referenced_assets(database_path, backup_root, project_directory)
}

fn copy_referenced_assets(
    database_path: &Path,
    source_root: &Path,
    destination_root: &Path,
) -> Result<(), ProjectError> {
    let connection = open_read_only(database_path)?;
    let mut statement = connection.prepare(
        "SELECT relative_path, thumbnail_path, checksum FROM media_asset
         ORDER BY relative_path",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Ok(());
    }
    validate_directory_metadata(
        &fs::symlink_metadata(source_root).map_err(|_| ProjectError::InvalidDatabase)?,
    )?;
    for (original, thumbnail, checksum) in rows {
        copy_asset_file(source_root, destination_root, &original, Some(&checksum))?;
        copy_asset_file(source_root, destination_root, &thumbnail, None)?;
    }
    Ok(())
}

fn copy_asset_file(
    source_root: &Path,
    destination_root: &Path,
    relative: &str,
    expected_checksum: Option<&str>,
) -> Result<(), ProjectError> {
    let components = relative_components(Path::new(relative))?;
    if components.first().and_then(|part| part.to_str()) != Some("assets") {
        return Err(ProjectError::InvalidDatabase);
    }
    let mut source = source_root.to_path_buf();
    for component in &components {
        source.push(component);
        let metadata = fs::symlink_metadata(&source).map_err(|_| ProjectError::InvalidDatabase)?;
        if metadata.file_type().is_symlink() {
            return Err(ProjectError::SymlinkNotAllowed);
        }
    }
    if !source.is_file() {
        return Err(ProjectError::InvalidDatabase);
    }
    let bytes = fs::read(&source).map_err(|source| ProjectError::Io {
        operation: "Asset für Sicherung lesen",
        source,
    })?;
    if let Some(expected) = expected_checksum
        && format!("{:x}", Sha256::digest(&bytes)) != expected
    {
        return Err(ProjectError::InvalidDatabase);
    }
    let destination = components
        .iter()
        .fold(destination_root.to_path_buf(), |path, part| path.join(part));
    let parent = destination
        .parent()
        .ok_or(ProjectError::InvalidRelativePath)?;
    fs::create_dir_all(parent).map_err(|source| ProjectError::Io {
        operation: "Asset-Sicherungsverzeichnis anlegen",
        source,
    })?;
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&destination)
        .map_err(|source| ProjectError::Io {
            operation: "Asset-Sicherung schreiben",
            source,
        })?;
    output
        .write_all(&bytes)
        .map_err(|source| ProjectError::Io {
            operation: "Asset-Sicherung schreiben",
            source,
        })?;
    output.sync_all().map_err(|source| ProjectError::Io {
        operation: "Asset-Sicherung synchronisieren",
        source,
    })
}

fn verify_database(path: &Path) -> Result<(), ProjectError> {
    let connection = open_read_only(path)?;
    verify_connection(&connection)
}

fn verify_restore_source(path: &Path) -> Result<(), ProjectError> {
    let connection = open_read_only(path)?;
    let integrity: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if integrity != "ok" {
        return Err(ProjectError::InvalidDatabase);
    }
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if !(1..=CURRENT_DATABASE_VERSION.0).contains(&version) {
        return Err(ProjectError::InvalidDatabase);
    }
    let required: u32 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master
             WHERE name IN ('project', 'document', 'entity', 'relation', 'link',
                            'storyline', 'plot_card', 'timeline', 'event',
                            'event_document_link', 'event_entity_link',
                            'media_asset', 'asset_link',
                            'asset_link_document_after_delete',
                            'asset_link_entity_after_delete',
                            'asset_link_event_after_delete',
                            'visual_profile', 'style_preset',
                            'style_preset_project_name_idx',
                            'visual_profile_asset_before_insert',
                            'visual_profile_asset_before_update',
                            'visual_profile_project_entity_idx',
                            'image_prompt_draft', 'image_prompt_draft_profile',
                            'image_prompt_draft_project_idx',
                            'image_prompt_profile_after_delete',
                            'image_prompt_style_before_insert',
                            'image_prompt_style_before_update',
                            'document_fts', 'document_fts_after_insert',
                            'document_fts_after_update', 'document_fts_after_delete')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| ProjectError::InvalidDatabase)?;
    let expected = match version {
        1 => 9,
        2 => 15,
        3 => 20,
        4 => 25,
        5 => 32,
        _ => return Err(ProjectError::InvalidDatabase),
    };
    if required == expected {
        Ok(())
    } else {
        Err(ProjectError::InvalidDatabase)
    }
}

fn verify_connection(connection: &Connection) -> Result<(), ProjectError> {
    let integrity: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if integrity != "ok" {
        return Err(ProjectError::InvalidDatabase);
    }
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if version != CURRENT_DATABASE_VERSION.0 {
        return Err(ProjectError::InvalidDatabase);
    }

    let object_count: u32 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master
             WHERE name IN ('project', 'document', 'entity', 'relation', 'link',
                            'storyline', 'plot_card', 'timeline', 'event',
                            'event_document_link', 'event_entity_link',
                            'media_asset', 'asset_link',
                            'asset_link_document_after_delete',
                            'asset_link_entity_after_delete',
                            'asset_link_event_after_delete',
                            'visual_profile', 'style_preset',
                            'style_preset_project_name_idx',
                            'visual_profile_asset_before_insert',
                            'visual_profile_asset_before_update',
                            'visual_profile_project_entity_idx',
                            'image_prompt_draft', 'image_prompt_draft_profile',
                            'image_prompt_draft_project_idx',
                            'image_prompt_profile_after_delete',
                            'image_prompt_style_before_insert',
                            'image_prompt_style_before_update',
                            'document_fts', 'document_fts_after_insert',
                            'document_fts_after_update', 'document_fts_after_delete')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if object_count != 32 {
        return Err(ProjectError::InvalidDatabase);
    }

    let project_count: u32 = connection
        .query_row("SELECT count(*) FROM project", [], |row| row.get(0))
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if project_count != 1 {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(())
}

struct TemporaryDirectory {
    path: PathBuf,
    remove_on_drop: bool,
}

impl TemporaryDirectory {
    fn create(parent: &Path) -> Result<Self, ProjectError> {
        for _ in 0..100 {
            let sequence = TEMPORARY_NAME_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".kraqel-stage-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        remove_on_drop: true,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => {
                    return Err(ProjectError::Io {
                        operation: "temporäres Projektverzeichnis anlegen",
                        source,
                    });
                }
            }
        }
        Err(ProjectError::Io {
            operation: "eindeutiges temporäres Projektverzeichnis anlegen",
            source: io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collision"),
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn keep(&mut self) {
        self.remove_on_drop = false;
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if self.remove_on_drop {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn reserve(parent: &Path) -> Result<Self, ProjectError> {
        for _ in 0..100 {
            let sequence = TEMPORARY_NAME_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(
                ".kraqel-backup-{}-{sequence}.tmp",
                std::process::id()
            ));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => {
                    return Err(ProjectError::Io {
                        operation: "temporäre Sicherung anlegen",
                        source,
                    });
                }
            }
        }
        Err(ProjectError::Io {
            operation: "eindeutige temporäre Sicherung anlegen",
            source: io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collision"),
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId, Project, ProjectId};
    use rusqlite::params;
    use sha2::{Digest as _, Sha256};
    use tempfile::tempdir;

    use super::{
        BACKUP_DIRECTORY_NAME, CURRENT_DATABASE_VERSION, DATABASE_FILE_NAME, ProjectError,
        ProjectManager,
    };

    fn project(name: &str) -> Project {
        Project {
            id: ProjectId::new(),
            name: name.to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    #[test]
    fn creates_and_opens_project_with_windows_friendly_path() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let relative = Path::new("Welten mit Leerzeichen").join("Äther-Projekt");

        let created = manager
            .create_project(&relative, &project("Äther & Archive"))
            .expect("project should be created");
        assert!(created.directory().ends_with(&relative));
        assert!(created.directory().join(DATABASE_FILE_NAME).is_file());
        assert!(created.directory().join(BACKUP_DIRECTORY_NAME).is_dir());
        drop(created);

        manager
            .open_project(&relative)
            .expect("project should reopen");
    }

    #[test]
    fn deletes_only_a_project_with_matching_directory_and_stored_id() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let doomed = project("Vergängliche Republik");
        let directory = root.path().join(doomed.id.to_string());
        drop(
            manager
                .create_project(doomed.id.to_string(), &doomed)
                .expect("project should be created"),
        );

        manager
            .delete_project(doomed.id)
            .expect("matching project should be deleted");

        assert!(!directory.exists());
        assert_eq!(manager.list_projects().unwrap(), Vec::new());

        let mismatched_directory_id = ProjectId::new();
        let mismatched = project("Nicht die erwartete Kennung");
        drop(
            manager
                .create_project(mismatched_directory_id.to_string(), &mismatched)
                .expect("mismatched fixture should be created"),
        );
        assert!(matches!(
            manager.delete_project(mismatched_directory_id),
            Err(ProjectError::InvalidProjectMetadata)
        ));
        assert!(
            root.path()
                .join(mismatched_directory_id.to_string())
                .is_dir()
        );
    }

    #[test]
    fn failed_initializer_never_publishes_a_partial_project() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project("Unvollständiger Import");

        let result = manager.create_project_with_initializer(
            project.id.to_string(),
            &project,
            |_database, directory| {
                fs::write(directory.join("halbfertig.txt"), b"nicht veroeffentlichen").unwrap();
                Err(ProjectError::InvalidImportedData)
            },
        );

        assert!(matches!(result, Err(ProjectError::InvalidImportedData)));
        assert!(!root.path().join(project.id.to_string()).exists());
        assert_eq!(manager.list_projects().unwrap().len(), 0);
        assert!(fs::read_dir(root.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".kraqel-stage-")
        }));
    }

    #[test]
    fn supports_long_project_directory_names() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let long_name = format!("Roman-{}", "x".repeat(180));

        let opened = manager
            .create_project(&long_name, &project("Ein langer Weg"))
            .expect("long project path should work");
        assert!(opened.directory().ends_with(long_name));
    }

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");

        assert!(matches!(
            manager.create_project(Path::new("..").join("escape"), &project("Flucht")),
            Err(ProjectError::InvalidRelativePath)
        ));
        assert!(matches!(
            manager.open_project(root.path()),
            Err(ProjectError::InvalidRelativePath)
        ));
    }

    #[test]
    fn reports_missing_and_corrupt_databases() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");

        assert!(matches!(
            manager.open_project("fehlt"),
            Err(ProjectError::ProjectMissing)
        ));

        let broken = root.path().join("kaputt");
        fs::create_dir(&broken).expect("broken project directory should be created");
        fs::write(broken.join(DATABASE_FILE_NAME), b"keine Datenbank")
            .expect("broken database should be written");
        assert!(matches!(
            manager.open_project("kaputt"),
            Err(ProjectError::InvalidDatabase)
        ));

        let incomplete = root.path().join("unvollstaendig");
        fs::create_dir(&incomplete).expect("incomplete project directory should be created");
        let connection = rusqlite::Connection::open(incomplete.join(DATABASE_FILE_NAME))
            .expect("sqlite database should be created");
        connection
            .pragma_update(None, "user_version", CURRENT_DATABASE_VERSION.0)
            .expect("claimed version should be stored");
        drop(connection);
        assert!(matches!(
            manager.open_project("unvollstaendig"),
            Err(ProjectError::InvalidDatabase)
        ));
    }

    #[test]
    fn backup_is_consistent_and_never_overwritten() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let opened = manager
            .create_project("quelle", &project("Quelle"))
            .expect("project should be created");
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO entity (
                    id, project_id, kind, name, created_at, updated_at
                 ) SELECT '018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3', id, 'figure',
                          'Ada', 'test', 'test' FROM project",
                [],
            )
            .expect("entity should be stored");
        drop(opened);

        let backup = manager
            .backup_project("quelle", "stand-1")
            .expect("backup should be created");
        assert!(backup.is_file());
        assert!(matches!(
            manager.backup_project("quelle", "stand-1"),
            Err(ProjectError::BackupAlreadyExists)
        ));
    }

    #[test]
    fn backup_and_restore_preserve_hash_verified_assets() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project("Medienprojekt");
        let project_id = project.id;
        let opened = manager
            .create_project("quelle", &project)
            .expect("project should be created");
        let original = b"deterministic image bytes";
        let thumbnail = b"thumbnail bytes";
        fs::create_dir_all(opened.directory().join("assets/thumbnails")).unwrap();
        fs::write(opened.directory().join("assets/image.png"), original).unwrap();
        fs::write(
            opened.directory().join("assets/thumbnails/image.png"),
            thumbnail,
        )
        .unwrap();
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO media_asset (
                id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                mime_type, checksum, byte_count, width, height, provenance, created_at, updated_at
             ) VALUES (?1, ?2, 'image', 'assets/image.png', 'assets/thumbnails/image.png',
                       'image.png', 'Bild', 'image/png', ?3, ?4, 1, 1, 'imported', 't', 't')",
                params![
                    EntityId::new().to_string(),
                    project_id.to_string(),
                    format!("{:x}", Sha256::digest(original)),
                    i64::try_from(original.len()).unwrap()
                ],
            )
            .unwrap();
        drop(opened);

        manager.backup_project("quelle", "mit-assets").unwrap();
        let restored = manager
            .restore_backup("quelle", "mit-assets", "wiederhergestellt")
            .unwrap();
        assert_eq!(
            fs::read(restored.directory().join("assets/image.png")).unwrap(),
            original
        );
        assert_eq!(
            fs::read(restored.directory().join("assets/thumbnails/image.png")).unwrap(),
            thumbnail
        );
    }

    #[test]
    fn backup_and_restore_preserve_visual_profiles_and_presets() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project("Visuelles Projekt");
        let project_id = project.id;
        let entity_id = EntityId::new();
        let opened = manager
            .create_project("quelle", &project)
            .expect("project should be created");
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO entity (id, project_id, kind, name, created_at, updated_at)
             VALUES (?1, ?2, 'character', 'Rhea', '0', '0')",
                params![entity_id.to_string(), project_id.to_string()],
            )
            .unwrap();
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO visual_profile (
                entity_id, project_id, traits_json, exclusions_json, created_at, updated_at
             ) VALUES (?1, ?2, '{\"Augen\":\"bernsteinfarben\"}', '[\"blaue Augen\"]', '0', '0')",
                params![entity_id.to_string(), project_id.to_string()],
            )
            .unwrap();
        opened.database().connection().execute(
            "INSERT INTO style_preset (
                id, project_id, name, visual_rules_json, applicable_kinds_json, created_at, updated_at
             ) VALUES (?1, ?2, 'Noir', '{\"Palette\":\"monochrom\"}', '[\"character\"]', '0', '0')",
            params![EntityId::new().to_string(), project_id.to_string()],
        ).unwrap();
        drop(opened);

        manager.backup_project("quelle", "visueller-stand").unwrap();
        let restored = manager
            .restore_backup("quelle", "visueller-stand", "wiederhergestellt")
            .unwrap();
        let profile_count: i64 = restored
            .database()
            .connection()
            .query_row(
                "SELECT count(*) FROM visual_profile WHERE entity_id = ?1",
                [entity_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        let preset_name: String = restored
            .database()
            .connection()
            .query_row(
                "SELECT name FROM style_preset WHERE project_id = ?1",
                [project_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(profile_count, 1);
        assert_eq!(preset_name, "Noir");
    }

    #[test]
    fn restore_creates_new_project_without_changing_source_backup() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        manager
            .create_project("quelle", &project("Quelle"))
            .expect("project should be created");
        let backup = manager
            .backup_project("quelle", "stand-1")
            .expect("backup should be created");
        let bytes_before = fs::read(&backup).expect("backup should be readable");

        let restored = manager
            .restore_backup("quelle", "stand-1", "wiederhergestellt")
            .expect("backup should restore");
        let project_count: i64 = restored
            .database()
            .connection()
            .query_row("SELECT count(*) FROM project", [], |row| row.get(0))
            .expect("restored project should be readable");
        assert_eq!(project_count, 1);
        assert_eq!(
            fs::read(&backup).expect("backup should remain readable"),
            bytes_before
        );
        assert!(matches!(
            manager.restore_backup("quelle", "stand-1", "wiederhergestellt"),
            Err(ProjectError::ProjectAlreadyExists)
        ));
    }

    #[test]
    fn restore_migrates_v1_backup_without_changing_source() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project("Quelle");
        manager
            .create_project("quelle", &project)
            .expect("project should be created");
        let backup = root
            .path()
            .join("quelle")
            .join(BACKUP_DIRECTORY_NAME)
            .join("alt.sqlite3");
        let connection = rusqlite::Connection::open(&backup).expect("v1 backup should open");
        connection
            .execute_batch(crate::INITIAL_SCHEMA)
            .expect("v1 schema should install");
        connection
            .pragma_update(None, "user_version", 1)
            .expect("v1 should be marked");
        connection
            .execute(
                "INSERT INTO project (id, name, schema_version, created_at, updated_at)
                 VALUES (?1, 'Quelle', 1, 'test', 'test')",
                [project.id.to_string()],
            )
            .expect("project should be inserted");
        drop(connection);

        let restored = manager
            .restore_backup("quelle", "alt", "wiederhergestellt")
            .expect("v1 backup should restore");
        assert_eq!(
            restored
                .database()
                .schema_version()
                .expect("version should load"),
            CURRENT_DATABASE_VERSION
        );
        let source = rusqlite::Connection::open(&backup).expect("source should reopen");
        let source_version: u32 = source
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("source version should load");
        assert_eq!(source_version, 1);
    }

    #[test]
    fn rejects_corrupt_backup_without_creating_destination() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let source = manager
            .create_project("quelle", &project("Quelle"))
            .expect("project should be created");
        fs::write(
            source
                .directory()
                .join(BACKUP_DIRECTORY_NAME)
                .join("kaputt.sqlite3"),
            b"zerbrochen",
        )
        .expect("corrupt backup should be written");
        drop(source);

        assert!(matches!(
            manager.restore_backup("quelle", "kaputt", "ziel"),
            Err(ProjectError::InvalidDatabase)
        ));
        assert!(!root.path().join("ziel").exists());
    }

    #[test]
    fn rejects_symlink_components() {
        let root = tempdir().expect("temporary root should exist");
        let outside = tempdir().expect("outside directory should exist");
        let link = root.path().join("verknuepfung");
        if create_directory_symlink(outside.path(), &link).is_err() {
            return;
        }
        let manager = ProjectManager::new(root.path()).expect("manager should open");

        assert!(matches!(
            manager.create_project(Path::new("verknuepfung").join("flucht"), &project("Flucht")),
            Err(ProjectError::SymlinkNotAllowed)
        ));
        assert!(!outside.path().join("flucht").exists());
    }

    #[cfg(unix)]
    fn create_directory_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, target)
    }

    #[cfg(windows)]
    fn create_directory_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
