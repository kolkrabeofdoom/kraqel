//! Autoritative lokale Speicherung und vorwärtsgerichtete Migrationen.

use std::{error::Error, fmt, path::Path, time::Duration};

use kraqel_domain::{CURRENT_SCHEMA_VERSION, SchemaVersion};
use rusqlite::{Connection, Transaction, TransactionBehavior};

pub mod document_repository;
pub mod project_manager;

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");
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

        if found > CURRENT_SCHEMA_VERSION {
            return Err(StorageError::FutureSchema {
                found,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }
        if found == CURRENT_SCHEMA_VERSION {
            return Ok(());
        }
        if found != SchemaVersion(0) || CURRENT_SCHEMA_VERSION != SchemaVersion(1) {
            return Err(StorageError::MissingMigration {
                found,
                target: CURRENT_SCHEMA_VERSION,
            });
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(INITIAL_SCHEMA)?;
        transaction.execute_batch("PRAGMA user_version = 1;")?;
        transaction.commit()?;
        Ok(())
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

    use super::{Database, StorageError};
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
            CURRENT_SCHEMA_VERSION
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
                                'document_fts', 'document_fts_after_insert',
                                'document_fts_after_update', 'document_fts_after_delete')",
                [],
                |row| row.get(0),
            )
            .expect("schema objects should be readable");
        assert_eq!(object_count, 9);
    }

    #[test]
    fn future_schema_is_rejected_without_modification() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("future.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection
            .pragma_update(None, "user_version", 2)
            .expect("future version should be set");
        drop(connection);

        let Err(error) = Database::open(&path) else {
            panic!("future schema should be rejected");
        };
        assert!(matches!(
            error,
            StorageError::FutureSchema {
                found: SchemaVersion(2),
                supported: SchemaVersion(1)
            }
        ));

        let connection = Connection::open(path).expect("database should reopen");
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("version should remain readable");
        assert_eq!(version, 2);
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
            CURRENT_SCHEMA_VERSION
        );
        assert!(
            fs::metadata(path)
                .expect("database file should exist")
                .len()
                > 0
        );
    }
}
