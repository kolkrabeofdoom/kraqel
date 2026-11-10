//! Transaktionales Repository für Kapitel, Szenen und versionierte Editorinhalte.

use std::{
    collections::BTreeSet,
    error::Error,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use kraqel_domain::{
    DocumentKind, DocumentNode, EDITOR_DOCUMENT_VERSION, EditorDocument, EntityId, ProjectId,
};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Value, json};

use crate::{Database, StorageError};

const MAX_EDITOR_DOCUMENT_BYTES: usize = 5 * 1024 * 1024;

/// Vollständiger gespeicherter Dokumentstand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDocument {
    /// Baum- und Revisionsmetadaten.
    pub node: DocumentNode,
    /// Versionierter Tiptap-Inhalt.
    pub content: EditorDocument,
    /// Für Suche und Wortzählung abgeleiteter Klartext.
    pub plain_text: String,
}

/// Fachlich unterscheidbare Fehler des Dokument-Repositorys.
#[derive(Debug)]
pub enum DocumentError {
    /// Titel oder Editorinhalt verletzt den öffentlichen Vertrag.
    InvalidInput,
    /// Ein Kapitel darf keinen Elternknoten besitzen.
    ChapterMustBeRoot,
    /// Eine Szene benötigt ein vorhandenes Kapitel desselben Projekts.
    InvalidParent,
    /// Projekt oder Dokument wurde nicht gefunden.
    NotFound,
    /// Der gespeicherte Stand wurde seit dem Laden verändert.
    RevisionConflict,
    /// Gespeicherte Daten verletzen den erwarteten internen Vertrag.
    InvalidStoredData,
    /// SQLite- oder Verbindungsfehler der Speicherschicht.
    Storage(StorageError),
    /// Die Systemzeit liegt vor der Unix-Epoche.
    InvalidSystemTime,
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die Dokumentdaten sind ungültig."),
            Self::ChapterMustBeRoot => {
                formatter.write_str("Kapitel müssen auf der Wurzelebene liegen.")
            }
            Self::InvalidParent => {
                formatter.write_str("Die Szene benötigt ein vorhandenes Kapitel.")
            }
            Self::NotFound => formatter.write_str("Das Dokument wurde nicht gefunden."),
            Self::RevisionConflict => {
                formatter.write_str("Das Dokument wurde zwischenzeitlich verändert.")
            }
            Self::InvalidStoredData => {
                formatter.write_str("Gespeicherte Dokumentdaten sind ungültig.")
            }
            Self::Storage(error) => error.fmt(formatter),
            Self::InvalidSystemTime => formatter.write_str("Die Systemzeit ist ungültig."),
        }
    }
}

impl Error for DocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::InvalidInput
            | Self::ChapterMustBeRoot
            | Self::InvalidParent
            | Self::NotFound
            | Self::RevisionConflict
            | Self::InvalidStoredData
            | Self::InvalidSystemTime => None,
        }
    }
}

impl From<rusqlite::Error> for DocumentError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}

impl From<StorageError> for DocumentError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Repository über einer bereits verwalteten Projektdatenbank.
pub struct DocumentRepository<'database> {
    database: &'database mut Database,
}

impl<'database> DocumentRepository<'database> {
    /// Bindet das Repository an eine Projektdatenbank.
    pub const fn new(database: &'database mut Database) -> Self {
        Self { database }
    }

    /// Listet alle Baumknoten mit stabiler Reihenfolge nach Elternknoten, Position und UUID.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei SQLite-Problemen oder ungültigen gespeicherten Werten.
    pub fn list(&self, project_id: ProjectId) -> Result<Vec<DocumentNode>, DocumentError> {
        let mut statement = self.database.connection().prepare(
            "SELECT id, project_id, parent_id, kind, title, position, revision
             FROM document
             WHERE project_id = ?1
             ORDER BY parent_id IS NOT NULL, parent_id, position, id",
        )?;
        let rows = statement.query_map([project_id.to_string()], |row| {
            Ok(RawDocumentNode {
                id: row.get(0)?,
                project_id: row.get(1)?,
                parent_id: row.get(2)?,
                kind: row.get(3)?,
                title: row.get(4)?,
                position: row.get(5)?,
                revision: row.get(6)?,
            })
        })?;

        rows.map(|row| {
            row.map_err(DocumentError::from)
                .and_then(RawDocumentNode::parse)
        })
        .collect()
    }

    /// Lädt einen vollständigen Kapitel- oder Szenenstand.
    ///
    /// # Errors
    ///
    /// Liefert `NotFound`, wenn der Knoten nicht zum Projekt gehört, sowie einen Fehler bei
    /// ungültigen gespeicherten Daten.
    pub fn load(
        &self,
        project_id: ProjectId,
        document_id: EntityId,
    ) -> Result<StoredDocument, DocumentError> {
        let raw = self
            .database
            .connection()
            .query_row(
                "SELECT id, project_id, parent_id, kind, title, position, revision,
                        content_json, plain_text
                 FROM document WHERE project_id = ?1 AND id = ?2",
                params![project_id.to_string(), document_id.to_string()],
                |row| {
                    Ok((
                        RawDocumentNode {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            parent_id: row.get(2)?,
                            kind: row.get(3)?,
                            title: row.get(4)?,
                            position: row.get(5)?,
                            revision: row.get(6)?,
                        },
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or(DocumentError::NotFound)?;
        let node = raw.0.parse()?;
        let content: EditorDocument =
            serde_json::from_str(&raw.1).map_err(|_| DocumentError::InvalidStoredData)?;
        validate_content(&content).map_err(|_| DocumentError::InvalidStoredData)?;
        Ok(StoredDocument {
            node,
            content,
            plain_text: raw.2,
        })
    }

    /// Erstellt ein Kapitel oder eine Szene an der letzten stabilen Geschwisterposition.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler bei ungültigem Titel, unzulässigem Elternknoten, fehlendem Projekt oder
    /// fehlgeschlagener Transaktion.
    pub fn create(
        &mut self,
        project_id: ProjectId,
        parent_id: Option<EntityId>,
        kind: DocumentKind,
        title: &str,
    ) -> Result<DocumentNode, DocumentError> {
        validate_title(title)?;
        let transaction = self.database.transaction()?;
        validate_project_and_parent(&transaction, project_id, parent_id, kind)?;

        let position: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM document
             WHERE project_id = ?1 AND parent_id IS ?2",
            params![project_id.to_string(), parent_id.map(|id| id.to_string())],
            |row| row.get(0),
        )?;
        let id = EntityId::new();
        let timestamp = current_timestamp()?;
        let content = empty_editor_document();
        let content_json =
            serde_json::to_string(&content).map_err(|_| DocumentError::InvalidInput)?;
        transaction.execute(
            "INSERT INTO document (
                id, project_id, parent_id, kind, title, position, revision,
                content_json, plain_text, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, '', ?8, ?8)",
            params![
                id.to_string(),
                project_id.to_string(),
                parent_id.map(|parent| parent.to_string()),
                kind_name(kind),
                title.trim(),
                position,
                content_json,
                timestamp
            ],
        )?;
        transaction.commit()?;

        Ok(DocumentNode {
            id,
            project_id,
            parent_id,
            kind,
            title: title.trim().to_owned(),
            position: u32::try_from(position).map_err(|_| DocumentError::InvalidStoredData)?,
            revision: 0,
        })
    }

    /// Speichert eine Szene nur dann, wenn ihre erwartete Revision noch aktuell ist.
    ///
    /// Der Klartext wird serverseitig aus dem Tiptap-JSON abgeleitet und zusammen mit Inhalt,
    /// Revision und FTS-Triggern in einer Transaktion geschrieben.
    ///
    /// # Errors
    ///
    /// Liefert einen Validierungs-, Nicht-gefunden- oder Revisionskonflikt sowie SQLite-Fehler.
    pub fn save_scene(
        &mut self,
        project_id: ProjectId,
        document_id: EntityId,
        expected_revision: u64,
        content: &EditorDocument,
    ) -> Result<u64, DocumentError> {
        validate_content(content)?;
        let content_json =
            serde_json::to_string(content).map_err(|_| DocumentError::InvalidInput)?;
        if content_json.len() > MAX_EDITOR_DOCUMENT_BYTES {
            return Err(DocumentError::InvalidInput);
        }
        let plain_text = plain_text(content);
        let next_revision = expected_revision
            .checked_add(1)
            .ok_or(DocumentError::InvalidInput)?;
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        let changed = transaction.execute(
            "UPDATE document
             SET content_json = ?1, plain_text = ?2, revision = ?3, updated_at = ?4
             WHERE project_id = ?5 AND id = ?6 AND kind = 'scene' AND revision = ?7",
            params![
                content_json,
                plain_text,
                i64::try_from(next_revision).map_err(|_| DocumentError::InvalidInput)?,
                timestamp,
                project_id.to_string(),
                document_id.to_string(),
                i64::try_from(expected_revision).map_err(|_| DocumentError::InvalidInput)?
            ],
        )?;
        if changed == 0 {
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM document
                    WHERE project_id = ?1 AND id = ?2 AND kind = 'scene'
                 )",
                params![project_id.to_string(), document_id.to_string()],
                |row| row.get(0),
            )?;
            return Err(if exists {
                DocumentError::RevisionConflict
            } else {
                DocumentError::NotFound
            });
        }
        transaction.commit()?;
        Ok(next_revision)
    }

    /// Benennt ein Kapitel oder eine Szene nur vom zuletzt angezeigten Titel aus um.
    ///
    /// Die Inhaltsrevision bleibt unverändert, damit eine reine Metadatenänderung keinen laufenden
    /// Autosave verfälscht. Der erwartete Titel bildet dafür einen eigenen Compare-and-swap-Schutz.
    ///
    /// # Errors
    ///
    /// Liefert `RevisionConflict`, wenn der Titel zwischenzeitlich geändert wurde.
    pub fn rename(
        &mut self,
        project_id: ProjectId,
        document_id: EntityId,
        expected_title: &str,
        title: &str,
    ) -> Result<DocumentNode, DocumentError> {
        validate_title(expected_title)?;
        validate_title(title)?;
        let title = title.trim();
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        let mut node = load_node(&transaction, project_id, document_id)?;
        if node.title != expected_title.trim() {
            return Err(DocumentError::RevisionConflict);
        }
        let changed = transaction.execute(
            "UPDATE document SET title = ?1, updated_at = ?2
             WHERE project_id = ?3 AND id = ?4 AND title = ?5",
            params![
                title,
                timestamp,
                project_id.to_string(),
                document_id.to_string(),
                expected_title.trim()
            ],
        )?;
        if changed != 1 {
            return Err(DocumentError::RevisionConflict);
        }
        transaction.commit()?;
        title.clone_into(&mut node.title);
        Ok(node)
    }

    /// Ordnet genau eine vollständige Geschwisterliste atomar neu.
    ///
    /// # Errors
    ///
    /// Liefert `RevisionConflict`, wenn sich Reihenfolge oder Menge seit der Anzeige geändert haben,
    /// sowie `InvalidInput` bei Duplikaten oder fremden Kennungen.
    pub fn reorder(
        &mut self,
        project_id: ProjectId,
        parent_id: Option<EntityId>,
        expected_order: &[EntityId],
        ordered_ids: &[EntityId],
    ) -> Result<(), DocumentError> {
        if expected_order.len() != ordered_ids.len()
            || expected_order
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != expected_order.len()
            || ordered_ids.iter().copied().collect::<BTreeSet<_>>().len() != ordered_ids.len()
            || expected_order.iter().copied().collect::<BTreeSet<_>>()
                != ordered_ids.iter().copied().collect::<BTreeSet<_>>()
        {
            return Err(DocumentError::InvalidInput);
        }
        let timestamp = current_timestamp()?;
        let transaction = self.database.transaction()?;
        validate_reorder_parent(&transaction, project_id, parent_id)?;
        let current = sibling_ids(&transaction, project_id, parent_id)?;
        if current != expected_order {
            return Err(DocumentError::RevisionConflict);
        }
        for (position, id) in ordered_ids.iter().enumerate() {
            transaction.execute(
                "UPDATE document SET position = ?1, updated_at = ?2
                 WHERE project_id = ?3 AND id = ?4 AND parent_id IS ?5",
                params![
                    i64::try_from(position).map_err(|_| DocumentError::InvalidInput)?,
                    timestamp,
                    project_id.to_string(),
                    id.to_string(),
                    parent_id.map(|value| value.to_string())
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Löscht den exakt angezeigten Dokumentstand samt deklarierter Kindliste.
    ///
    /// Kapitelkinder und fachliche Referenzen werden durch die dokumentierten SQLite-Regeln
    /// kaskadiert. Eine neue Revision, Umbenennung oder abweichende Kindliste verhindert das
    /// Löschen.
    ///
    /// # Errors
    ///
    /// Liefert `RevisionConflict` bei einem veralteten Stand und `InvalidInput`, wenn für eine Szene
    /// Kinder angegeben werden.
    pub fn delete(
        &mut self,
        project_id: ProjectId,
        document_id: EntityId,
        expected_title: &str,
        expected_revision: u64,
        expected_child_ids: &[EntityId],
    ) -> Result<(), DocumentError> {
        validate_title(expected_title)?;
        if expected_child_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != expected_child_ids.len()
        {
            return Err(DocumentError::InvalidInput);
        }
        let transaction = self.database.transaction()?;
        let node = load_node(&transaction, project_id, document_id)?;
        if node.title != expected_title.trim() || node.revision != expected_revision {
            return Err(DocumentError::RevisionConflict);
        }
        let children = sibling_ids(&transaction, project_id, Some(document_id))?;
        if node.kind == DocumentKind::Scene && !expected_child_ids.is_empty() {
            return Err(DocumentError::InvalidInput);
        }
        if children != expected_child_ids {
            return Err(DocumentError::RevisionConflict);
        }
        let changed = transaction.execute(
            "DELETE FROM document
             WHERE project_id = ?1 AND id = ?2 AND title = ?3 AND revision = ?4",
            params![
                project_id.to_string(),
                document_id.to_string(),
                expected_title.trim(),
                i64::try_from(expected_revision).map_err(|_| DocumentError::InvalidInput)?
            ],
        )?;
        if changed != 1 {
            return Err(DocumentError::RevisionConflict);
        }
        compact_siblings(&transaction, project_id, node.parent_id)?;
        transaction.commit()?;
        Ok(())
    }
}

struct RawDocumentNode {
    id: String,
    project_id: String,
    parent_id: Option<String>,
    kind: String,
    title: String,
    position: i64,
    revision: i64,
}

impl RawDocumentNode {
    fn parse(self) -> Result<DocumentNode, DocumentError> {
        Ok(DocumentNode {
            id: EntityId::parse(&self.id).ok_or(DocumentError::InvalidStoredData)?,
            project_id: ProjectId::parse(&self.project_id)
                .ok_or(DocumentError::InvalidStoredData)?,
            parent_id: match self.parent_id {
                Some(value) => {
                    Some(EntityId::parse(&value).ok_or(DocumentError::InvalidStoredData)?)
                }
                None => None,
            },
            kind: parse_kind(&self.kind)?,
            title: self.title,
            position: u32::try_from(self.position).map_err(|_| DocumentError::InvalidStoredData)?,
            revision: u64::try_from(self.revision).map_err(|_| DocumentError::InvalidStoredData)?,
        })
    }
}

fn load_node(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    document_id: EntityId,
) -> Result<DocumentNode, DocumentError> {
    transaction
        .query_row(
            "SELECT id, project_id, parent_id, kind, title, position, revision
             FROM document WHERE project_id = ?1 AND id = ?2",
            params![project_id.to_string(), document_id.to_string()],
            |row| {
                Ok(RawDocumentNode {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    kind: row.get(3)?,
                    title: row.get(4)?,
                    position: row.get(5)?,
                    revision: row.get(6)?,
                })
            },
        )
        .optional()?
        .ok_or(DocumentError::NotFound)?
        .parse()
}

fn sibling_ids(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    parent_id: Option<EntityId>,
) -> Result<Vec<EntityId>, DocumentError> {
    let mut statement = transaction.prepare(
        "SELECT id FROM document
         WHERE project_id = ?1 AND parent_id IS ?2
         ORDER BY position, id",
    )?;
    let rows = statement.query_map(
        params![
            project_id.to_string(),
            parent_id.map(|value| value.to_string())
        ],
        |row| row.get::<_, String>(0),
    )?;
    rows.map(|row| {
        row.map_err(DocumentError::from)
            .and_then(|value| EntityId::parse(&value).ok_or(DocumentError::InvalidStoredData))
    })
    .collect()
}

fn validate_reorder_parent(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    parent_id: Option<EntityId>,
) -> Result<(), DocumentError> {
    if let Some(parent_id) = parent_id {
        let parent = load_node(transaction, project_id, parent_id)?;
        if parent.kind != DocumentKind::Chapter || parent.parent_id.is_some() {
            return Err(DocumentError::InvalidParent);
        }
    } else {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1)",
            [project_id.to_string()],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(DocumentError::NotFound);
        }
    }
    Ok(())
}

fn compact_siblings(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    parent_id: Option<EntityId>,
) -> Result<(), DocumentError> {
    let siblings = sibling_ids(transaction, project_id, parent_id)?;
    for (position, id) in siblings.iter().enumerate() {
        transaction.execute(
            "UPDATE document SET position = ?1 WHERE project_id = ?2 AND id = ?3",
            params![
                i64::try_from(position).map_err(|_| DocumentError::InvalidStoredData)?,
                project_id.to_string(),
                id.to_string()
            ],
        )?;
    }
    Ok(())
}

fn validate_project_and_parent(
    transaction: &Transaction<'_>,
    project_id: ProjectId,
    parent_id: Option<EntityId>,
    kind: DocumentKind,
) -> Result<(), DocumentError> {
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1)",
        [project_id.to_string()],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(DocumentError::NotFound);
    }

    match (kind, parent_id) {
        (DocumentKind::Chapter, None) => Ok(()),
        (DocumentKind::Chapter, Some(_)) => Err(DocumentError::ChapterMustBeRoot),
        (DocumentKind::Scene, None) => Err(DocumentError::InvalidParent),
        (DocumentKind::Scene, Some(parent)) => {
            let valid: bool = transaction.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM document
                    WHERE project_id = ?1 AND id = ?2 AND kind = 'chapter'
                 )",
                params![project_id.to_string(), parent.to_string()],
                |row| row.get(0),
            )?;
            if valid {
                Ok(())
            } else {
                Err(DocumentError::InvalidParent)
            }
        }
    }
}

fn validate_title(title: &str) -> Result<(), DocumentError> {
    let length = title.trim().chars().count();
    if (1..=240).contains(&length) {
        Ok(())
    } else {
        Err(DocumentError::InvalidInput)
    }
}

fn validate_content(content: &EditorDocument) -> Result<(), DocumentError> {
    if content.version != EDITOR_DOCUMENT_VERSION
        || content.document.get("type").and_then(Value::as_str) != Some("doc")
        || content
            .document
            .get("content")
            .is_some_and(|value| !value.is_array())
    {
        return Err(DocumentError::InvalidInput);
    }
    Ok(())
}

fn empty_editor_document() -> EditorDocument {
    EditorDocument {
        version: EDITOR_DOCUMENT_VERSION,
        document: json!({"type": "doc", "content": [{"type": "paragraph"}]}),
    }
}

fn plain_text(content: &EditorDocument) -> String {
    let mut fragments = Vec::new();
    collect_text(&content.document, &mut fragments);
    fragments
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_text(value: &Value, fragments: &mut Vec<String>) {
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        fragments.push(text.to_owned());
    }
    if let Some(children) = value.get("content").and_then(Value::as_array) {
        for child in children {
            collect_text(child, fragments);
        }
    }
}

const fn kind_name(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Chapter => "chapter",
        DocumentKind::Scene => "scene",
    }
}

fn parse_kind(value: &str) -> Result<DocumentKind, DocumentError> {
    match value {
        "chapter" => Ok(DocumentKind::Chapter),
        "scene" => Ok(DocumentKind::Scene),
        _ => Err(DocumentError::InvalidStoredData),
    }
}

fn current_timestamp() -> Result<String, DocumentError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| DocumentError::InvalidSystemTime)?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}

#[cfg(test)]
mod tests {
    use kraqel_domain::{
        CURRENT_SCHEMA_VERSION, DocumentKind, EDITOR_DOCUMENT_VERSION, EditorDocument, Project,
        ProjectId,
    };
    use rusqlite::params;
    use serde_json::json;
    use tempfile::tempdir;

    use super::{DocumentError, DocumentRepository};
    use crate::project_manager::ProjectManager;

    #[test]
    fn creates_stably_sorted_tree_and_rejects_invalid_parents() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("projekt", &project)
            .expect("project should be created");
        let mut repository = DocumentRepository::new(opened.database_mut());
        let chapter = repository
            .create(project.id, None, DocumentKind::Chapter, "Kapitel 1")
            .expect("chapter should be created");
        let first = repository
            .create(project.id, Some(chapter.id), DocumentKind::Scene, "Szene 1")
            .expect("scene should be created");
        let second = repository
            .create(project.id, Some(chapter.id), DocumentKind::Scene, "Szene 2")
            .expect("scene should be created");

        let nodes = repository.list(project.id).expect("tree should load");
        assert_eq!(nodes, vec![chapter, first, second]);
        assert!(matches!(
            repository.create(project.id, None, DocumentKind::Scene, "Verwaist"),
            Err(DocumentError::InvalidParent)
        ));
    }

    #[test]
    fn saves_with_optimistic_revision_and_server_derived_plain_text() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("projekt", &project)
            .expect("project should be created");
        let content = content("Die Stadt erwacht.");
        let scene = {
            let mut repository = DocumentRepository::new(opened.database_mut());
            let chapter = repository
                .create(project.id, None, DocumentKind::Chapter, "Kapitel")
                .expect("chapter should be created");
            let scene = repository
                .create(project.id, Some(chapter.id), DocumentKind::Scene, "Szene")
                .expect("scene should be created");

            assert_eq!(
                repository
                    .save_scene(project.id, scene.id, 0, &content)
                    .expect("scene should save"),
                1
            );
            let loaded = repository
                .load(project.id, scene.id)
                .expect("scene should load");
            assert_eq!(loaded.plain_text, "Die Stadt erwacht.");
            assert_eq!(loaded.content, content);
            assert!(matches!(
                repository.save_scene(project.id, scene.id, 0, &content),
                Err(DocumentError::RevisionConflict)
            ));
            scene
        };

        opened
            .database()
            .connection()
            .execute(
                "UPDATE document SET content_json = ?1 WHERE id = ?2",
                params![
                    r#"{"version":1,"document":{"type":"not-a-doc"}}"#,
                    scene.id.to_string()
                ],
            )
            .expect("malformed contract should be stored for the corruption test");
        let repository = DocumentRepository::new(opened.database_mut());
        assert!(matches!(
            repository.load(project.id, scene.id),
            Err(DocumentError::InvalidStoredData)
        ));
    }

    #[test]
    fn renames_reorders_and_deletes_only_the_displayed_tree_state() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("projekt", &project)
            .expect("project should be created");
        let mut repository = DocumentRepository::new(opened.database_mut());
        let first_chapter = repository
            .create(project.id, None, DocumentKind::Chapter, "Kapitel 1")
            .expect("chapter should be created");
        let second_chapter = repository
            .create(project.id, None, DocumentKind::Chapter, "Kapitel 2")
            .expect("chapter should be created");
        let first_scene = repository
            .create(
                project.id,
                Some(first_chapter.id),
                DocumentKind::Scene,
                "Szene 1",
            )
            .expect("scene should be created");
        let second_scene = repository
            .create(
                project.id,
                Some(first_chapter.id),
                DocumentKind::Scene,
                "Szene 2",
            )
            .expect("scene should be created");

        let renamed = repository
            .rename(project.id, first_scene.id, "Szene 1", "Ankunft")
            .expect("current title should rename");
        assert_eq!(renamed.title, "Ankunft");
        assert_eq!(renamed.revision, first_scene.revision);
        assert!(matches!(
            repository.rename(project.id, first_scene.id, "Szene 1", "Veraltet"),
            Err(DocumentError::RevisionConflict)
        ));

        repository
            .reorder(
                project.id,
                None,
                &[first_chapter.id, second_chapter.id],
                &[second_chapter.id, first_chapter.id],
            )
            .expect("chapters should reorder");
        assert!(matches!(
            repository.reorder(
                project.id,
                None,
                &[first_chapter.id, second_chapter.id],
                &[second_chapter.id, first_chapter.id],
            ),
            Err(DocumentError::RevisionConflict)
        ));
        repository
            .reorder(
                project.id,
                Some(first_chapter.id),
                &[first_scene.id, second_scene.id],
                &[second_scene.id, first_scene.id],
            )
            .expect("scenes should reorder");

        assert!(matches!(
            repository.delete(
                project.id,
                first_chapter.id,
                "Kapitel 1",
                0,
                &[first_scene.id, second_scene.id],
            ),
            Err(DocumentError::RevisionConflict)
        ));
        repository
            .delete(
                project.id,
                first_chapter.id,
                "Kapitel 1",
                0,
                &[second_scene.id, first_scene.id],
            )
            .expect("exactly displayed chapter should delete");

        let remaining = repository.list(project.id).expect("tree should load");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, second_chapter.id);
        assert_eq!(remaining[0].position, 0);
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Die Integrationsfixture belegt alle dokumentbezogenen Kaskaden gemeinsam"
    )]
    fn scene_delete_cascades_all_document_references() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("projekt", &project)
            .expect("project should be created");
        let (scene, entity, event, asset, draft) = {
            let mut repository = DocumentRepository::new(opened.database_mut());
            let chapter = repository
                .create(project.id, None, DocumentKind::Chapter, "Kapitel")
                .expect("chapter should be created");
            let scene = repository
                .create(project.id, Some(chapter.id), DocumentKind::Scene, "Szene")
                .expect("scene should be created");
            (
                scene,
                kraqel_domain::EntityId::new(),
                kraqel_domain::EntityId::new(),
                kraqel_domain::EntityId::new(),
                kraqel_domain::EntityId::new(),
            )
        };
        let timeline = kraqel_domain::EntityId::new();
        let connection = opened.database().connection();
        connection
            .execute(
                "INSERT INTO entity (
                    id, project_id, kind, name, properties_json, created_at, updated_at
                 ) VALUES (?1, ?2, 'character', 'Rhea', '{}', 't', 't')",
                params![entity.to_string(), project.id.to_string()],
            )
            .expect("entity fixture");
        connection
            .execute(
                "INSERT INTO link (id, project_id, document_id, entity_id, role, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'reference', 't')",
                params![
                    kraqel_domain::EntityId::new().to_string(),
                    project.id.to_string(),
                    scene.id.to_string(),
                    entity.to_string()
                ],
            )
            .expect("story link fixture");
        connection
            .execute(
                "INSERT INTO plot_card (
                    project_id, document_id, status, position, updated_at
                 ) VALUES (?1, ?2, 'planned', 0, 't')",
                params![project.id.to_string(), scene.id.to_string()],
            )
            .expect("plot card fixture");
        connection
            .execute(
                "INSERT INTO timeline (
                    id, project_id, name, mode, position, created_at, updated_at
                 ) VALUES (?1, ?2, 'Zeit', 'relative', 0, 't', 't')",
                params![timeline.to_string(), project.id.to_string()],
            )
            .expect("timeline fixture");
        connection
            .execute(
                "INSERT INTO event (
                    id, project_id, timeline_id, title, description, position, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, 'Ereignis', '', 0, 't', 't')",
                params![
                    event.to_string(),
                    project.id.to_string(),
                    timeline.to_string()
                ],
            )
            .expect("event fixture");
        connection
            .execute(
                "INSERT INTO event_document_link (project_id, event_id, document_id, created_at)
                 VALUES (?1, ?2, ?3, 't')",
                params![
                    project.id.to_string(),
                    event.to_string(),
                    scene.id.to_string()
                ],
            )
            .expect("timeline link fixture");
        connection
            .execute(
                "INSERT INTO media_asset (
                    id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                    mime_type, checksum, byte_count, width, height, provenance, tags_json,
                    created_at, updated_at
                 ) VALUES (
                    ?1, ?2, 'image', ?3, ?4, 'rabe.png', 'Rabe', 'image/png', ?5,
                    1, 1, 1, 'imported', '[]', 't', 't'
                 )",
                params![
                    asset.to_string(),
                    project.id.to_string(),
                    format!("assets/{asset}.png"),
                    format!("assets/thumbnails/{asset}.png"),
                    "a".repeat(64)
                ],
            )
            .expect("asset fixture");
        connection
            .execute(
                "INSERT INTO asset_link (
                    project_id, asset_id, target_type, target_id, role, created_at
                 ) VALUES (?1, ?2, 'document', ?3, 'reference', 't')",
                params![
                    project.id.to_string(),
                    asset.to_string(),
                    scene.id.to_string()
                ],
            )
            .expect("asset link fixture");
        connection
            .execute(
                "INSERT INTO visual_profile (
                    entity_id, project_id, traits_json, exclusions_json, created_at, updated_at
                 ) VALUES (?1, ?2, '{}', '[]', 't', 't')",
                params![entity.to_string(), project.id.to_string()],
            )
            .expect("profile fixture");
        connection
            .execute(
                "INSERT INTO image_prompt_draft (
                    id, project_id, name, source_document_id, source_document_revision,
                    prompt_text, created_at, updated_at
                 ) VALUES (?1, ?2, 'Prompt', ?3, 0, 'Rabe', 't', 't')",
                params![
                    draft.to_string(),
                    project.id.to_string(),
                    scene.id.to_string()
                ],
            )
            .expect("prompt fixture");
        connection
            .execute(
                "INSERT INTO image_prompt_draft_profile (
                    draft_id, project_id, entity_id, position
                 ) VALUES (?1, ?2, ?3, 0)",
                params![
                    draft.to_string(),
                    project.id.to_string(),
                    entity.to_string()
                ],
            )
            .expect("prompt profile fixture");

        DocumentRepository::new(opened.database_mut())
            .delete(project.id, scene.id, "Szene", 0, &[])
            .expect("current scene should delete");

        for table in [
            "link",
            "plot_card",
            "event_document_link",
            "asset_link",
            "image_prompt_draft",
        ] {
            let count: i64 = opened
                .database()
                .connection()
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("cascade count");
            assert_eq!(count, 0, "{table} should cascade");
        }
        let surviving: i64 = opened
            .database()
            .connection()
            .query_row(
                "SELECT
                    (SELECT count(*) FROM entity) +
                    (SELECT count(*) FROM event) +
                    (SELECT count(*) FROM media_asset)",
                [],
                |row| row.get(0),
            )
            .expect("surviving records");
        assert_eq!(surviving, 3);
    }

    #[test]
    fn one_hundred_scenes_survive_backup_and_recovery() {
        let root = tempdir().expect("temporary root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let project = project();
        let mut opened = manager
            .create_project("quelle", &project)
            .expect("project should be created");
        {
            let mut repository = DocumentRepository::new(opened.database_mut());
            let chapter = repository
                .create(project.id, None, DocumentKind::Chapter, "Kapitel")
                .expect("chapter should be created");
            for index in 0..100 {
                repository
                    .create(
                        project.id,
                        Some(chapter.id),
                        DocumentKind::Scene,
                        &format!("Szene {index:03}"),
                    )
                    .expect("scene should be created");
            }
        }
        drop(opened);

        manager
            .backup_project("quelle", "m1")
            .expect("backup should be created");
        let mut restored = manager
            .restore_backup("quelle", "m1", "wiederhergestellt")
            .expect("backup should restore");
        let repository = DocumentRepository::new(restored.database_mut());
        assert_eq!(
            repository.list(project.id).expect("tree should load").len(),
            101
        );
    }

    fn project() -> Project {
        Project {
            id: ProjectId::new(),
            name: "Morgenrot".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    fn content(text: &str) -> EditorDocument {
        EditorDocument {
            version: EDITOR_DOCUMENT_VERSION,
            document: json!({
                "type": "doc",
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": text}]}]
            }),
        }
    }
}
