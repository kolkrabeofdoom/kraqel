//! Autoritative lokale Speicherung und vorwärtsgerichtete Migrationen.

use std::{error::Error, fmt, path::Path, time::Duration};

use kraqel_domain::SchemaVersion;
use rusqlite::{Connection, Transaction, TransactionBehavior};

pub mod continuity_repository;
pub mod document_repository;
pub mod media_repository;
pub mod project_manager;
pub mod prompt_repository;
pub mod story_bible_repository;
pub mod structure_repository;
pub mod visual_repository;

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");
const STRUCTURE_SCHEMA: &str = include_str!("../migrations/0002_plotboard_timeline.sql");
const MEDIA_SCHEMA: &str = include_str!("../migrations/0003_media_library.sql");
const VISUAL_SCHEMA: &str = include_str!("../migrations/0004_visual_profiles.sql");
const PROMPT_SCHEMA: &str = include_str!("../migrations/0005_image_prompt_drafts.sql");
/// Aktueller interner SQLite-Migrationsstand.
pub const CURRENT_DATABASE_VERSION: SchemaVersion = SchemaVersion(6);
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Von der Speicherschicht gemeldete Fehler.
#[derive(Debug)]
pub enum StorageError {
    /// SQLite konnte die angeforderte Operation nicht ausführen.
    Sqlite(rusqlite::Error),
    /// Die Datenbank stammt aus einer neueren, nicht unterstützten KRAQEL-Version.
    FutureSchema {
        /// In der Datenbank gespeicherte Version.
        found: SchemaVersion,
        /// Höchste von dieser Anwendung unterstützte Version.
        supported: SchemaVersion,
    },
    /// Für eine ältere Datenbank fehlt eine vorwärtsgerichtete Migration.
    MissingMigration {
        /// In der Datenbank gespeicherte Version.
        found: SchemaVersion,
        /// Von dieser Anwendung erwartete Version.
        target: SchemaVersion,
    },
    /// SQLite hat die verbindliche Fremdschlüsselprüfung nicht aktiviert.
    ForeignKeysDisabled,
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(formatter, "SQLite-Fehler: {error}"),
            Self::FutureSchema { found, supported } => write!(
                formatter,
                "Datenbankschema {} ist neuer als die unterstützte Version {}",
                found.0, supported.0
            ),
            Self::MissingMigration { found, target } => write!(
                formatter,
                "keine Migration von Datenbankschema {} auf {} vorhanden",
                found.0, target.0
            ),
            Self::ForeignKeysDisabled => {
                formatter.write_str("SQLite-Fremdschlüsselprüfung ist deaktiviert")
            }
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::FutureSchema { .. }
            | Self::MissingMigration { .. }
            | Self::ForeignKeysDisabled => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

/// Verwaltete Verbindung zur autoritativen Projektdatenbank.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Öffnet oder erzeugt eine dateibasierte Datenbank und migriert sie auf den aktuellen Stand.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn SQLite nicht geöffnet oder konfiguriert werden kann, eine
    /// Migration fehlschlägt oder die Datei aus einer neueren KRAQEL-Version stammt.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        Self::initialize(connection, true)
    }

    /// Erzeugt eine flüchtige Datenbank mit identischem Schema für Tests und Werkzeuge.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn SQLite nicht initialisiert oder migriert werden kann.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        Self::initialize(connection, false)
    }

    fn initialize(connection: Connection, use_wal: bool) -> Result<Self, StorageError> {
        connection.busy_timeout(BUSY_TIMEOUT)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;")?;

        if use_wal {
            connection.execute_batch("PRAGMA journal_mode = WAL;")?;
        }

        let foreign_keys: bool =
            connection.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
        if !foreign_keys {
            return Err(StorageError::ForeignKeysDisabled);
        }

        let mut database = Self { connection };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&mut self) -> Result<(), StorageError> {
        let found = self.schema_version()?;

        if found > CURRENT_DATABASE_VERSION {
            return Err(StorageError::FutureSchema {
                found,
                supported: CURRENT_DATABASE_VERSION,
            });
        }
        if found == CURRENT_DATABASE_VERSION {
            return Ok(());
        }
        let mut version = found;
        while version < CURRENT_DATABASE_VERSION {
            if version == SchemaVersion(5) {
                self.migrate_imported_titles()?;
                version = SchemaVersion(6);
                continue;
            }
            let transaction = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            match version {
                SchemaVersion(0) => {
                    transaction.execute_batch(INITIAL_SCHEMA)?;
                    transaction.execute_batch("PRAGMA user_version = 1;")?;
                    version = SchemaVersion(1);
                }
                SchemaVersion(1) => {
                    transaction.execute_batch(STRUCTURE_SCHEMA)?;
                    transaction.execute_batch("PRAGMA user_version = 2;")?;
                    version = SchemaVersion(2);
                }
                SchemaVersion(2) => {
                    transaction.execute_batch(MEDIA_SCHEMA)?;
                    transaction.execute_batch("PRAGMA user_version = 3;")?;
                    version = SchemaVersion(3);
                }
                SchemaVersion(3) => {
                    transaction.execute_batch(VISUAL_SCHEMA)?;
                    transaction.execute_batch("PRAGMA user_version = 4;")?;
                    version = SchemaVersion(4);
                }
                SchemaVersion(4) => {
                    transaction.execute_batch(PROMPT_SCHEMA)?;
                    transaction.execute_batch("PRAGMA user_version = 5;")?;
                    version = SchemaVersion(5);
                }
                _ => {
                    return Err(StorageError::MissingMigration {
                        found: version,
                        target: CURRENT_DATABASE_VERSION,
                    });
                }
            }
            transaction.commit()?;
        }
        Ok(())
    }

    fn migrate_imported_titles(&mut self) -> Result<(), StorageError> {
        // SQLite requires this outside a transaction for its documented table-rebuild procedure.
        self.connection
            .execute_batch("PRAGMA foreign_keys = OFF;")?;
        let result = (|| -> Result<(), StorageError> {
            let transaction = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(include_str!(
                "../migrations/0006_imported_document_titles.sql"
            ))?;
            let violations: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
                [],
                |row| row.get(0),
            )?;
            if violations {
                return Err(StorageError::Sqlite(rusqlite::Error::InvalidQuery));
            }
            transaction.commit()?;
            Ok(())
        })();
        self.connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        result
    }

    /// Liefert die gespeicherte Schema-Version.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn SQLite die Versionsinformation nicht lesen kann.
    pub fn schema_version(&self) -> Result<SchemaVersion, StorageError> {
        let version = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(SchemaVersion(version))
    }

    /// Beginnt eine sofortige Transaktion für zusammengehörige Schreiboperationen.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn SQLite die Transaktion nicht beginnen kann.
    pub fn transaction(&mut self) -> Result<Transaction<'_>, StorageError> {
        self.connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StorageError::from)
    }

    /// Gewährt lesenden Zugriff für Abfragen der Repository-Schicht.
    #[must_use]
    pub const fn connection(&self) -> &Connection {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::{Connection, params};
    use tempfile::tempdir;

    use super::{
        CURRENT_DATABASE_VERSION, Database, INITIAL_SCHEMA, MEDIA_SCHEMA, STRUCTURE_SCHEMA,
        StorageError, VISUAL_SCHEMA,
    };
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, SchemaVersion};

    const PROJECT_ID: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2";
    const DOCUMENT_ID: &str = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3";
    const TIMESTAMP: &str = "2026-11-09T12:00:00Z";

    fn insert_project(connection: &Connection) {
        connection
            .execute(
                "INSERT INTO project (id, name, schema_version, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![PROJECT_ID, "Morgenrot", CURRENT_SCHEMA_VERSION.0, TIMESTAMP],
            )
            .expect("project should be inserted");
    }

    fn insert_document(connection: &Connection, text: &str) {
        connection
            .execute(
                "INSERT INTO document (
                    id, project_id, kind, title, position, plain_text, created_at, updated_at
                 ) VALUES (?1, ?2, 'scene', 'Ankunft', 0, ?3, ?4, ?4)",
                params![DOCUMENT_ID, PROJECT_ID, text, TIMESTAMP],
            )
            .expect("document should be inserted");
    }

    fn fts_matches(connection: &Connection, query: &str) -> i64 {
        connection
            .query_row(
                "SELECT count(*) FROM document_fts WHERE document_fts MATCH ?1",
                [query],
                |row| row.get(0),
            )
            .expect("FTS query should succeed")
    }

    #[test]
    fn empty_database_is_migrated_atomically() {
        let database = Database::open_in_memory().expect("database should open");

        assert_eq!(
            database
                .schema_version()
                .expect("version should be readable"),
            CURRENT_DATABASE_VERSION
        );
        let foreign_keys: bool = database
            .connection()
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("foreign key setting should be readable");
        assert!(foreign_keys);

        let object_count: i64 = database
            .connection()
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
            .expect("schema objects should be readable");
        assert_eq!(object_count, 32);
    }

    #[test]
    fn future_schema_is_rejected_without_modification() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("future.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection
            .pragma_update(None, "user_version", 7)
            .expect("future version should be set");
        drop(connection);

        let Err(error) = Database::open(&path) else {
            panic!("future schema should be rejected");
        };
        assert!(matches!(
            error,
            StorageError::FutureSchema {
                found: SchemaVersion(7),
                supported: SchemaVersion(6)
            }
        ));

        let connection = Connection::open(path).expect("database should reopen");
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("version should remain readable");
        assert_eq!(version, 7);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let database = Database::open_in_memory().expect("database should open");
        let result = database.connection().execute(
            "INSERT INTO document (
                id, project_id, kind, title, position, created_at, updated_at
             ) VALUES (?1, ?2, 'scene', 'Verwaist', 0, ?3, ?3)",
            params![DOCUMENT_ID, PROJECT_ID, TIMESTAMP],
        );

        assert!(result.is_err());
    }

    #[test]
    fn fts_index_tracks_insert_update_and_delete() {
        let database = Database::open_in_memory().expect("database should open");
        insert_project(database.connection());
        insert_document(database.connection(), "Die Libelle landet am Fluss.");
        assert_eq!(fts_matches(database.connection(), "libelle"), 1);

        database
            .connection()
            .execute(
                "UPDATE document SET plain_text = 'Der Rabe fliegt weiter.' WHERE id = ?1",
                [DOCUMENT_ID],
            )
            .expect("document should be updated");
        assert_eq!(fts_matches(database.connection(), "libelle"), 0);
        assert_eq!(fts_matches(database.connection(), "rabe"), 1);

        database
            .connection()
            .execute("DELETE FROM document WHERE id = ?1", [DOCUMENT_ID])
            .expect("document should be deleted");
        assert_eq!(fts_matches(database.connection(), "rabe"), 0);
    }

    #[test]
    fn populated_database_survives_reopen_without_remigration() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("project.sqlite3");

        {
            let database = Database::open(&path).expect("database should open");
            insert_project(database.connection());
            insert_document(database.connection(), "Ein beharrlicher Text.");
        }

        let database = Database::open(&path).expect("database should reopen");
        let title: String = database
            .connection()
            .query_row(
                "SELECT title FROM document WHERE id = ?1",
                [DOCUMENT_ID],
                |row| row.get(0),
            )
            .expect("document should remain stored");
        assert_eq!(title, "Ankunft");
        assert_eq!(
            database
                .schema_version()
                .expect("version should be readable"),
            CURRENT_DATABASE_VERSION
        );
        assert!(
            fs::metadata(path)
                .expect("database file should exist")
                .len()
                > 0
        );
    }

    #[test]
    fn populated_v1_database_migrates_without_losing_existing_data() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("v1.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection
            .execute_batch(INITIAL_SCHEMA)
            .expect("v1 schema should install");
        connection
            .pragma_update(None, "user_version", 1)
            .expect("v1 should be marked");
        insert_project(&connection);
        insert_document(&connection, "Bleibt erhalten.");
        drop(connection);

        let database = Database::open(&path).expect("v1 database should migrate");
        assert_eq!(
            database.schema_version().expect("version should load"),
            CURRENT_DATABASE_VERSION
        );
        let text: String = database
            .connection()
            .query_row(
                "SELECT plain_text FROM document WHERE id = ?1",
                [DOCUMENT_ID],
                |row| row.get(0),
            )
            .expect("document should remain");
        assert_eq!(text, "Bleibt erhalten.");
        let structure_tables: i64 = database.connection().query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('storyline', 'plot_card', 'timeline', 'event')",
            [], |row| row.get(0)
        ).expect("new tables should exist");
        assert_eq!(structure_tables, 4);
    }

    #[test]
    fn populated_v2_database_adds_media_tables_without_losing_documents() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("v2.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection.execute_batch(INITIAL_SCHEMA).unwrap();
        connection.execute_batch(STRUCTURE_SCHEMA).unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();
        insert_project(&connection);
        insert_document(&connection, "Bleibt auch bei Medien erhalten.");
        drop(connection);

        let database = Database::open(&path).expect("v2 database should migrate");
        assert_eq!(database.schema_version().unwrap(), CURRENT_DATABASE_VERSION);
        let text: String = database
            .connection()
            .query_row(
                "SELECT plain_text FROM document WHERE id = ?1",
                [DOCUMENT_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "Bleibt auch bei Medien erhalten.");
        let media_tables: i64 = database
            .connection()
            .query_row(
                "SELECT count(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('media_asset', 'asset_link')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(media_tables, 2);
    }

    #[test]
    fn populated_v3_database_adds_visual_tables_without_losing_media() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("v3.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection.execute_batch(INITIAL_SCHEMA).unwrap();
        connection.execute_batch(STRUCTURE_SCHEMA).unwrap();
        connection.execute_batch(MEDIA_SCHEMA).unwrap();
        connection.pragma_update(None, "user_version", 3).unwrap();
        insert_project(&connection);
        insert_document(&connection, "Bleibt bei visuellen Profilen erhalten.");
        drop(connection);

        let database = Database::open(&path).expect("v3 database should migrate");
        assert_eq!(database.schema_version().unwrap(), CURRENT_DATABASE_VERSION);
        let text: String = database
            .connection()
            .query_row(
                "SELECT plain_text FROM document WHERE id = ?1",
                [DOCUMENT_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "Bleibt bei visuellen Profilen erhalten.");
        let visual_tables: i64 = database
            .connection()
            .query_row(
                "SELECT count(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('visual_profile', 'style_preset')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(visual_tables, 2);
    }

    #[test]
    fn v5_title_migration_preserves_references_fts_and_delete_triggers() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("v5.sqlite3");
        let connection = Connection::open(&path).unwrap();
        for schema in [
            INITIAL_SCHEMA,
            STRUCTURE_SCHEMA,
            MEDIA_SCHEMA,
            VISUAL_SCHEMA,
            super::PROMPT_SCHEMA,
        ] {
            connection.execute_batch(schema).unwrap();
        }
        connection.pragma_update(None, "user_version", 5).unwrap();
        insert_project(&connection);
        insert_document(&connection, "Suchanker bleibt erhalten");
        connection.execute("INSERT INTO plot_card (project_id,document_id,position,updated_at) VALUES (?1,?2,0,?3)", params![PROJECT_ID,DOCUMENT_ID,TIMESTAMP]).unwrap();
        connection.execute("INSERT INTO image_prompt_draft (id,project_id,name,source_document_id,source_document_revision,prompt_text,created_at,updated_at) VALUES (?1,?2,'Probe',?3,0,'Text',?4,?4)", params!["018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4",PROJECT_ID,DOCUMENT_ID,TIMESTAMP]).unwrap();
        drop(connection);
        let database = Database::open(&path).unwrap();
        let db = database.connection();
        assert_eq!(fts_matches(db, "Suchanker"), 1);
        for table in ["plot_card", "image_prompt_draft"] {
            assert_eq!(
                db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1
            );
        }
        assert_eq!(
            db.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
            0
        );
        assert!(
            db.pragma_query_value::<bool, _>(None, "foreign_keys", |r| r.get(0))
                .unwrap()
        );
        db.execute("UPDATE document SET title='' WHERE id=?1", [DOCUMENT_ID])
            .unwrap();
        assert_eq!(fts_matches(db, "Suchanker"), 1);
        db.execute("DELETE FROM document WHERE id=?1", [DOCUMENT_ID])
            .unwrap();
        assert_eq!(fts_matches(db, "Suchanker"), 0);
        assert_eq!(
            db.query_row("SELECT count(*) FROM image_prompt_draft", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            db.query_row("SELECT count(*) FROM plot_card", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn populated_v4_database_adds_prompt_tables_without_losing_profiles() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("v4.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection.execute_batch(INITIAL_SCHEMA).unwrap();
        connection.execute_batch(STRUCTURE_SCHEMA).unwrap();
        connection.execute_batch(MEDIA_SCHEMA).unwrap();
        connection.execute_batch(VISUAL_SCHEMA).unwrap();
        connection.pragma_update(None, "user_version", 4).unwrap();
        insert_project(&connection);
        insert_document(&connection, "Bleibt bei Prompt-Entwürfen erhalten.");
        drop(connection);

        let database = Database::open(&path).expect("v4 database should migrate");
        assert_eq!(database.schema_version().unwrap(), CURRENT_DATABASE_VERSION);
        let prompt_tables: i64 = database
            .connection()
            .query_row(
                "SELECT count(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('image_prompt_draft', 'image_prompt_draft_profile')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(prompt_tables, 2);
        let text: String = database
            .connection()
            .query_row(
                "SELECT plain_text FROM document WHERE id = ?1",
                [DOCUMENT_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "Bleibt bei Prompt-Entwürfen erhalten.");
    }
}
