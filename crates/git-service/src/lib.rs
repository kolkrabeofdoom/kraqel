//! Deterministische Exportsnapshots und kontrollierte Git-Operationen.

use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use kraqel_domain::ProjectId;
use kraqel_storage::Database;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

const SNAPSHOT_DIRECTORY: &str = "snapshot";
const MANUSCRIPT_DIRECTORY: &str = "manuscript";
const MAX_GIT_OUTPUT_BYTES: usize = 1024 * 1024;
const SNAPSHOT_FILES_V1: [&str; 7] = [
    "manifest.json",
    "schema.json",
    "documents.json",
    "story-bible.json",
    "plotboard.json",
    "timelines.json",
    "README.md",
];
const SNAPSHOT_FILES_V2: [&str; 8] = [
    "manifest.json",
    "schema.json",
    "documents.json",
    "story-bible.json",
    "plotboard.json",
    "timelines.json",
    "visual-profiles.json",
    "README.md",
];
const GITIGNORE: &str = concat!(
    "# KRAQEL: SQLite ist autoritativ; Git erfasst nur portable Snapshots.\n",
    "*\n",
    "!/.gitignore\n",
    "!/snapshot/\n",
    "!/snapshot/**\n",
);

/// Ergebnis einer Snapshot-Erzeugung oder -Prüfung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    /// Version des portablen Snapshotformats.
    pub format_version: u16,
    /// Projektkennung aus dem Manifest.
    pub project_id: ProjectId,
    /// Anzahl portabler Dateien einschließlich Markdown-Dokumenten.
    pub file_count: u32,
    /// Gesamte Bytegröße der portablen Dateien.
    pub byte_count: u64,
}

/// Git-Zustand ausschließlich für freigegebene Snapshotpfade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionControlStatus {
    /// Ob ein lokales Repository initialisiert wurde.
    pub initialized: bool,
    /// Ob keine freigegebenen Snapshotdateien geändert sind.
    pub clean: bool,
    /// Geänderte relative Pfade aus dem eingeschränkten Git-Status.
    pub changed_files: Vec<String>,
    /// Aktueller Commit, sofern bereits einer existiert.
    pub head: Option<String>,
}

/// Erfolgreich erzeugter lokaler Commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResult {
    /// Vollständige Commitkennung.
    pub commit_id: String,
}

/// Fehler der Snapshot- und Git-Schicht.
#[derive(Debug)]
pub enum GitServiceError {
    /// Projektpfad, Projektkennung oder Commitnachricht ist ungültig.
    InvalidInput,
    /// Gespeicherte Daten verletzen das Snapshotformat.
    InvalidSnapshot,
    /// Git ist nicht installiert oder die Operation ist fehlgeschlagen.
    GitFailed,
    /// Es gibt keine freigegebene Änderung für einen Commit.
    NothingToCommit,
    /// SQLite konnte die Snapshotabfrage nicht ausführen.
    Database(rusqlite::Error),
    /// Eine lokale Dateisystemoperation ist fehlgeschlagen.
    Io(io::Error),
    /// JSON konnte nicht stabil serialisiert oder geprüft werden.
    Json(serde_json::Error),
}

impl fmt::Display for GitServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("Die Versionsanfrage ist ungültig."),
            Self::InvalidSnapshot => formatter.write_str("Der portable Snapshot ist ungültig."),
            Self::GitFailed => formatter.write_str("Die lokale Git-Operation ist fehlgeschlagen."),
            Self::NothingToCommit => formatter.write_str("Es gibt keine Snapshotänderung."),
            Self::Database(_) => {
                formatter.write_str("Die Projektdaten konnten nicht gelesen werden.")
            }
            Self::Io(_) => {
                formatter.write_str("Die Snapshotdateien konnten nicht verarbeitet werden.")
            }
            Self::Json(_) => {
                formatter.write_str("Das Snapshotformat konnte nicht verarbeitet werden.")
            }
        }
    }
}

impl Error for GitServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for GitServiceError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
impl From<io::Error> for GitServiceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for GitServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// Bindet Snapshot- und Git-Operationen an genau einen verwalteten Projektordner.
pub struct GitService<'project> {
    project_directory: &'project Path,
}

impl<'project> GitService<'project> {
    /// Erstellt einen Dienst für einen bereits durch den Projektmanager geprüften Ordner.
    #[must_use]
    pub const fn new(project_directory: &'project Path) -> Self {
        Self { project_directory }
    }

    /// Erzeugt und veröffentlicht einen deterministischen Snapshot und staged nur erlaubte Pfade.
    ///
    /// # Errors
    /// Liefert einen Fehler bei inkonsistenten Daten, Dateisystem- oder Git-Problemen.
    pub fn create_snapshot(
        &self,
        database: &Database,
        project_id: ProjectId,
    ) -> Result<SnapshotSummary, GitServiceError> {
        self.require_safe_project_directory()?;
        let staging = self
            .project_directory
            .join(format!(".kraqel-snapshot-new-{}", Uuid::new_v4()));
        fs::create_dir(&staging)?;
        if let Err(error) = Self::write_snapshot(
            database.connection(),
            project_id,
            &staging,
            self.project_directory,
        ) {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        self.publish_snapshot(&staging)?;
        self.write_gitignore()?;
        self.ensure_repository()?;
        self.run_git(&["add", "-A", "--", ".gitignore", SNAPSHOT_DIRECTORY])?;
        self.validate_snapshot(project_id)
    }

    /// Prüft Manifest, feste JSON-Dateien und UUID-basierte Manuskriptdateien ausschließlich lesend.
    ///
    /// # Errors
    /// Liefert einen Fehler, wenn Dateien fehlen oder Inhalte nicht dem Formatvertrag entsprechen.
    pub fn validate_snapshot(
        &self,
        expected_project_id: ProjectId,
    ) -> Result<SnapshotSummary, GitServiceError> {
        let root = self.project_directory.join(SNAPSHOT_DIRECTORY);
        let manifest: Manifest = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)
            .map_err(|_| GitServiceError::InvalidSnapshot)?;
        let expected_files = expected_manifest_files(manifest.format_version)?;
        if manifest.format != "kraqel-project-snapshot"
            || manifest.project.id != expected_project_id
            || manifest.files != expected_files
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        for name in &expected_files {
            let path = root.join(name);
            if !path.is_file() {
                return Err(GitServiceError::InvalidSnapshot);
            }
            if Path::new(name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                let value: Value = serde_json::from_slice(&fs::read(path)?)
                    .map_err(|_| GitServiceError::InvalidSnapshot)?;
                validate_json_file(name, &value)?;
            }
        }
        validate_snapshot_assets(&root, &manifest.assets)?;
        if manifest.format_version == 2 {
            validate_visual_references(&root, &manifest.assets)?;
        }
        let manuscript = root.join(MANUSCRIPT_DIRECTORY);
        if !manuscript.is_dir() {
            return Err(GitServiceError::InvalidSnapshot);
        }
        for entry in fs::read_dir(&manuscript)? {
            let entry = entry?;
            if entry.file_type()?.is_symlink() || !entry.file_type()?.is_file() {
                return Err(GitServiceError::InvalidSnapshot);
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return Err(GitServiceError::InvalidSnapshot);
            };
            let Some(id) = name.strip_suffix(".md") else {
                return Err(GitServiceError::InvalidSnapshot);
            };
            if kraqel_domain::EntityId::parse(id).is_none() {
                return Err(GitServiceError::InvalidSnapshot);
            }
        }
        summarize(&root, manifest.format_version, manifest.project.id)
    }

    /// Liefert den eingeschränkten Status der freigegebenen Snapshotdateien.
    ///
    /// # Errors
    /// Liefert einen Fehler bei unlesbarem Projektordner oder fehlgeschlagenem Git-Aufruf.
    pub fn status(&self) -> Result<VersionControlStatus, GitServiceError> {
        self.require_safe_project_directory()?;
        if !self.project_directory.join(".git").is_dir() {
            return Ok(VersionControlStatus {
                initialized: false,
                clean: true,
                changed_files: Vec::new(),
                head: None,
            });
        }
        let output = self.run_git(&[
            "status",
            "--porcelain=v1",
            "--",
            ".gitignore",
            SNAPSHOT_DIRECTORY,
        ])?;
        let mut changed_files: Vec<String> = output_text(&output)?
            .lines()
            .filter_map(|line| line.get(3..))
            .map(|path| path.trim_matches('"').replace('\\', "/"))
            .collect();
        changed_files.sort();
        changed_files.dedup();
        let head = self
            .run_git_optional(&["rev-parse", "HEAD"])?
            .map(|output| output_text(&output).map(|text| text.trim().to_owned()))
            .transpose()?;
        Ok(VersionControlStatus {
            initialized: true,
            clean: changed_files.is_empty(),
            changed_files,
            head,
        })
    }

    /// Liefert einen begrenzten Unified Diff ausschließlich für gestagte Snapshotpfade.
    ///
    /// # Errors
    /// Liefert einen Fehler bei fehlgeschlagenem Git-Aufruf.
    pub fn diff(&self) -> Result<String, GitServiceError> {
        if !self.project_directory.join(".git").is_dir() {
            return Ok(String::new());
        }
        let output = self.run_git(&[
            "diff",
            "--cached",
            "--no-ext-diff",
            "--no-color",
            "--",
            ".gitignore",
            SNAPSHOT_DIRECTORY,
        ])?;
        output_text(&output)
    }

    /// Erstellt einen Commit, der ausschließlich die erlaubten Snapshotpfade enthält.
    ///
    /// # Errors
    /// Liefert einen Fehler bei ungültiger Nachricht, leerem Index oder Git-Problemen.
    pub fn commit(&self, message: &str) -> Result<CommitResult, GitServiceError> {
        let message = message.trim();
        if message.is_empty() || message.chars().count() > 200 || message.contains(['\r', '\n']) {
            return Err(GitServiceError::InvalidInput);
        }
        if self.status()?.clean {
            return Err(GitServiceError::NothingToCommit);
        }
        let output = self.run_git_optional(&[
            "-c",
            "core.hooksPath=.kraqel-disabled-hooks",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=KRAQEL",
            "-c",
            "user.email=kraqel@localhost",
            "commit",
            "--only",
            "-m",
            message,
            "--",
            ".gitignore",
            SNAPSHOT_DIRECTORY,
        ])?;
        if output.is_none() {
            return Err(GitServiceError::NothingToCommit);
        }
        Ok(CommitResult {
            commit_id: output_text(&self.run_git(&["rev-parse", "HEAD"])?)?
                .trim()
                .to_owned(),
        })
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Die lineare Abbildung aller Snapshotdateien hält Sortierung und Vollständigkeit gemeinsam prüfbar"
    )]
    fn write_snapshot(
        connection: &Connection,
        project_id: ProjectId,
        root: &Path,
        project_directory: &Path,
    ) -> Result<(), GitServiceError> {
        let project_id_text = project_id.to_string();
        let (stored_id, name, schema_version, settings): (String, String, u32, String) = connection
            .query_row(
                "SELECT id, name, schema_version, settings_json FROM project WHERE id = ?1",
                [&project_id_text],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;
        if stored_id != project_id_text {
            return Err(GitServiceError::InvalidInput);
        }
        let documents = query_rows(
            connection,
            "SELECT id, parent_id, kind, title, position, revision, content_json, plain_text FROM document WHERE project_id = ?1 ORDER BY parent_id, position, id",
            &project_id_text,
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?, "parentId": row.get::<_, Option<String>>(1)?,
                    "kind": row.get::<_, String>(2)?, "title": row.get::<_, String>(3)?,
                    "position": row.get::<_, u32>(4)?, "revision": nonnegative_u64(row.get::<_, i64>(5)?)?,
                "content": parse_json(&row.get::<_, String>(6)?)?, "plainText": row.get::<_, String>(7)?,
                }))
            },
        )?;
        let entities = query_json_rows(
            connection,
            "SELECT id, kind, name, properties_json FROM entity WHERE project_id = ?1 ORDER BY kind, name, id",
            &project_id_text,
            &["id", "kind", "name", "properties"],
        )?;
        let relations = query_json_rows(
            connection,
            "SELECT id, source_entity_id, target_entity_id, kind, properties_json FROM relation WHERE project_id = ?1 ORDER BY source_entity_id, target_entity_id, kind, id",
            &project_id_text,
            &[
                "id",
                "sourceEntityId",
                "targetEntityId",
                "kind",
                "properties",
            ],
        )?;
        let links = query_text_rows(
            connection,
            "SELECT document_id, entity_id, role FROM link WHERE project_id = ?1 ORDER BY document_id, entity_id, role",
            &project_id_text,
            &["documentId", "entityId", "role"],
        )?;
        let storylines = query_mixed_rows(
            connection,
            "SELECT id, name, color, position FROM storyline WHERE project_id = ?1 ORDER BY position, id",
            &project_id_text,
            &["id", "name", "color", "position"],
        )?;
        let cards = query_mixed_rows(
            connection,
            "SELECT document_id, storyline_id, pov_entity_id, status, position FROM plot_card WHERE project_id = ?1 ORDER BY position, document_id",
            &project_id_text,
            &[
                "documentId",
                "storylineId",
                "povEntityId",
                "status",
                "position",
            ],
        )?;
        let timelines = query_mixed_rows(
            connection,
            "SELECT id, name, mode, position FROM timeline WHERE project_id = ?1 ORDER BY position, id",
            &project_id_text,
            &["id", "name", "mode", "position"],
        )?;
        let events = query_mixed_rows(
            connection,
            "SELECT id, timeline_id, title, description, time_value, position FROM event WHERE project_id = ?1 ORDER BY timeline_id, position, id",
            &project_id_text,
            &[
                "id",
                "timelineId",
                "title",
                "description",
                "timeValue",
                "position",
            ],
        )?;
        let event_documents = query_text_rows(
            connection,
            "SELECT event_id, document_id FROM event_document_link WHERE project_id = ?1 ORDER BY event_id, document_id",
            &project_id_text,
            &["eventId", "documentId"],
        )?;
        let event_entities = query_text_rows(
            connection,
            "SELECT event_id, entity_id FROM event_entity_link WHERE project_id = ?1 ORDER BY event_id, entity_id",
            &project_id_text,
            &["eventId", "entityId"],
        )?;
        let visual_profiles = query_rows(
            connection,
            "SELECT entity_id, traits_json, canonical_asset_id, exclusions_json
             FROM visual_profile WHERE project_id = ?1 ORDER BY entity_id",
            &project_id_text,
            |row| {
                Ok(json!({
                    "entityId": row.get::<_, String>(0)?,
                    "traits": parse_json(&row.get::<_, String>(1)?)?,
                    "canonicalAssetId": row.get::<_, Option<String>>(2)?,
                    "exclusions": parse_json(&row.get::<_, String>(3)?)?,
                }))
            },
        )?;
        let style_presets = query_rows(
            connection,
            "SELECT id, name, visual_rules_json, applicable_kinds_json
             FROM style_preset WHERE project_id = ?1 ORDER BY name COLLATE NOCASE, id",
            &project_id_text,
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "visualRules": parse_json(&row.get::<_, String>(2)?)?,
                    "applicableKinds": parse_json(&row.get::<_, String>(3)?)?,
                }))
            },
        )?;

        fs::create_dir(root.join(MANUSCRIPT_DIRECTORY))?;
        write_json(
            root.join("documents.json"),
            &json!({"kind": "documents", "documents": documents}),
        )?;
        write_json(
            root.join("story-bible.json"),
            &json!({"kind": "storyBible", "entities": entities, "relations": relations, "links": links}),
        )?;
        write_json(
            root.join("plotboard.json"),
            &json!({"kind": "plotboard", "storylines": storylines, "cards": cards}),
        )?;
        write_json(
            root.join("timelines.json"),
            &json!({"kind": "timelines", "timelines": timelines, "events": events, "documentLinks": event_documents, "entityLinks": event_entities}),
        )?;
        write_json(
            root.join("visual-profiles.json"),
            &json!({"kind": "visualProfiles", "profiles": visual_profiles, "stylePresets": style_presets}),
        )?;
        fs::write(
            root.join("schema.json"),
            include_bytes!("../../../docs/data-format/snapshot-v2.schema.json"),
        )?;
        let assets = write_snapshot_assets(connection, &project_id_text, project_directory, root)?;
        let manifest = Manifest {
            format: "kraqel-project-snapshot".to_owned(),
            format_version: 2,
            project: ManifestProject {
                id: project_id,
                name,
                schema_version,
                settings: serde_json::from_str(&settings)?,
            },
            files: expected_manifest_files(2)?,
            assets,
        };
        write_json(root.join("manifest.json"), &manifest)?;
        fs::write(
            root.join("README.md"),
            "# KRAQEL-Projektsnapshot\n\nDieser Ordner ist ein portabler, deterministischer Export. SQLite bleibt die lokale Autorität.\n",
        )?;
        if let Some(items) = documents.as_array() {
            for item in items {
                let id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(GitServiceError::InvalidSnapshot)?;
                let title = item
                    .get("title")
                    .and_then(Value::as_str)
                    .ok_or(GitServiceError::InvalidSnapshot)?;
                let kind = item
                    .get("kind")
                    .and_then(Value::as_str)
                    .ok_or(GitServiceError::InvalidSnapshot)?;
                let text = item
                    .get("plainText")
                    .and_then(Value::as_str)
                    .ok_or(GitServiceError::InvalidSnapshot)?;
                let level = if kind == "chapter" { "#" } else { "##" };
                fs::write(
                    root.join(MANUSCRIPT_DIRECTORY).join(format!("{id}.md")),
                    format!(
                        "{level} {}\n\n{}\n",
                        normalize_inline(title),
                        normalize_text(text)
                    ),
                )?;
            }
        }
        Ok(())
    }

    fn publish_snapshot(&self, staging: &Path) -> Result<(), GitServiceError> {
        let destination = self.project_directory.join(SNAPSHOT_DIRECTORY);
        if !destination.exists() {
            fs::rename(staging, destination)?;
            return Ok(());
        }
        if fs::symlink_metadata(&destination)?.file_type().is_symlink() {
            return Err(GitServiceError::InvalidInput);
        }
        let old = self
            .project_directory
            .join(format!(".kraqel-snapshot-old-{}", Uuid::new_v4()));
        fs::rename(&destination, &old)?;
        if let Err(error) = fs::rename(staging, &destination) {
            let _ = fs::rename(&old, &destination);
            return Err(error.into());
        }
        fs::remove_dir_all(old)?;
        Ok(())
    }

    fn write_gitignore(&self) -> Result<(), GitServiceError> {
        let temporary = self
            .project_directory
            .join(format!(".gitignore-{}.tmp", Uuid::new_v4()));
        fs::write(&temporary, GITIGNORE)?;
        let destination = self.project_directory.join(".gitignore");
        if destination.exists() {
            fs::remove_file(&destination)?;
        }
        fs::rename(temporary, destination)?;
        Ok(())
    }

    fn ensure_repository(&self) -> Result<(), GitServiceError> {
        if !self.project_directory.join(".git").exists() {
            self.run_git(&["init", "--quiet"])?;
        }
        if !self.project_directory.join(".git").is_dir() {
            return Err(GitServiceError::InvalidInput);
        }
        Ok(())
    }

    fn require_safe_project_directory(&self) -> Result<(), GitServiceError> {
        let metadata = fs::symlink_metadata(self.project_directory)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(GitServiceError::InvalidInput);
        }
        Ok(())
    }

    fn run_git(&self, arguments: &[&str]) -> Result<Output, GitServiceError> {
        self.run_git_optional(arguments)?
            .ok_or(GitServiceError::GitFailed)
    }

    fn run_git_optional(&self, arguments: &[&str]) -> Result<Option<Output>, GitServiceError> {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(self.project_directory)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", null_device())
            .output()
            .map_err(|_| GitServiceError::GitFailed)?;
        if output.stdout.len() > MAX_GIT_OUTPUT_BYTES || output.stderr.len() > MAX_GIT_OUTPUT_BYTES
        {
            return Err(GitServiceError::GitFailed);
        }
        Ok(output.status.success().then_some(output))
    }
}

fn write_snapshot_assets(
    connection: &Connection,
    project_id: &str,
    project_directory: &Path,
    snapshot_root: &Path,
) -> Result<Vec<Value>, GitServiceError> {
    let mut statement = connection.prepare(
        "SELECT id, original_name, title, mime_type, checksum, byte_count, width, height,
                provenance, tags_json, relative_path
         FROM media_asset WHERE project_id = ?1 ORDER BY id",
    )?;
    let rows = statement
        .query_map([project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, u32>(6)?,
                row.get::<_, u32>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    fs::create_dir(snapshot_root.join("assets"))?;
    let mut result = Vec::with_capacity(rows.len());
    for (
        id,
        original_name,
        title,
        mime_type,
        checksum,
        byte_count,
        width,
        height,
        provenance,
        tags_json,
        relative_path,
    ) in rows
    {
        let extension = snapshot_extension(&mime_type)?;
        let snapshot_path = format!("assets/{checksum}.{extension}");
        let source = resolve_project_asset(project_directory, &relative_path)?;
        let bytes = fs::read(source)?;
        if byte_count < 0
            || u64::try_from(byte_count).ok() != Some(bytes.len() as u64)
            || format!("{:x}", Sha256::digest(&bytes)) != checksum
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        fs::write(snapshot_root.join(&snapshot_path), bytes)?;
        let tags: Value =
            serde_json::from_str(&tags_json).map_err(|_| GitServiceError::InvalidSnapshot)?;
        if !tags.is_array() {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let mut link_statement = connection.prepare(
            "SELECT target_type, target_id, role FROM asset_link
             WHERE project_id = ?1 AND asset_id = ?2 ORDER BY target_type, target_id, role",
        )?;
        let links = link_statement
            .query_map(params![project_id, id], |row| {
                Ok(json!({
                    "targetKind": row.get::<_, String>(0)?,
                    "targetId": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        result.push(json!({
            "id": id,
            "kind": "image",
            "path": snapshot_path,
            "originalName": original_name,
            "title": title,
            "mimeType": mime_type,
            "checksum": checksum,
            "byteCount": byte_count,
            "width": width,
            "height": height,
            "provenance": provenance,
            "tags": tags,
            "links": links,
        }));
    }
    Ok(result)
}

fn validate_snapshot_assets(root: &Path, assets: &[Value]) -> Result<(), GitServiceError> {
    let mut expected = BTreeSet::new();
    for asset in assets {
        let path = asset
            .get("path")
            .and_then(Value::as_str)
            .ok_or(GitServiceError::InvalidSnapshot)?;
        let checksum = asset
            .get("checksum")
            .and_then(Value::as_str)
            .ok_or(GitServiceError::InvalidSnapshot)?;
        let mime_type = asset
            .get("mimeType")
            .and_then(Value::as_str)
            .ok_or(GitServiceError::InvalidSnapshot)?;
        let extension = snapshot_extension(mime_type)?;
        if path != format!("assets/{checksum}.{extension}")
            || checksum.len() != 64
            || !checksum
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || !expected.insert(path.to_owned())
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let file = root.join(path);
        let metadata = fs::symlink_metadata(&file)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let bytes = fs::read(file)?;
        if format!("{:x}", Sha256::digest(&bytes)) != checksum
            || asset.get("byteCount").and_then(Value::as_u64) != Some(bytes.len() as u64)
            || asset.get("links").is_none_or(|value| !value.is_array())
            || asset.get("tags").is_none_or(|value| !value.is_array())
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    let asset_directory = root.join("assets");
    if assets.is_empty() {
        if asset_directory.exists() {
            return Err(GitServiceError::InvalidSnapshot);
        }
        return Ok(());
    }
    let actual = fs::read_dir(asset_directory)?
        .map(|entry| {
            let entry = entry?;
            if entry.file_type()?.is_symlink() || !entry.file_type()?.is_file() {
                return Err(GitServiceError::InvalidSnapshot);
            }
            Ok(format!("assets/{}", entry.file_name().to_string_lossy()))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual == expected {
        Ok(())
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn resolve_project_asset(root: &Path, relative: &str) -> Result<PathBuf, GitServiceError> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Err(GitServiceError::InvalidSnapshot);
    }
    let mut current = root.to_path_buf();
    for (index, component) in path.components().enumerate() {
        let std::path::Component::Normal(name) = component else {
            return Err(GitServiceError::InvalidSnapshot);
        };
        if index == 0 && name != "assets" {
            return Err(GitServiceError::InvalidSnapshot);
        }
        current.push(name);
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    if current.is_file() {
        Ok(current)
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn snapshot_extension(mime_type: &str) -> Result<&'static str, GitServiceError> {
    match mime_type {
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        _ => Err(GitServiceError::InvalidSnapshot),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    format: String,
    format_version: u16,
    project: ManifestProject,
    files: Vec<String>,
    assets: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestProject {
    id: ProjectId,
    name: String,
    schema_version: u32,
    settings: Value,
}

fn expected_manifest_files(format_version: u16) -> Result<Vec<String>, GitServiceError> {
    let files: &[&str] = match format_version {
        1 => &SNAPSHOT_FILES_V1,
        2 => &SNAPSHOT_FILES_V2,
        _ => return Err(GitServiceError::InvalidSnapshot),
    };
    Ok(files.iter().map(ToString::to_string).collect())
}

fn validate_json_file(name: &str, value: &Value) -> Result<(), GitServiceError> {
    let valid = match name {
        "manifest.json" => {
            value.get("format").and_then(Value::as_str) == Some("kraqel-project-snapshot")
        }
        "schema.json" => {
            value.get("$schema").and_then(Value::as_str)
                == Some("https://json-schema.org/draft/2020-12/schema")
        }
        "documents.json" => {
            value.get("kind").and_then(Value::as_str) == Some("documents")
                && value.get("documents").is_some_and(Value::is_array)
        }
        "story-bible.json" => {
            value.get("kind").and_then(Value::as_str) == Some("storyBible")
                && ["entities", "relations", "links"]
                    .iter()
                    .all(|key| value.get(key).is_some_and(Value::is_array))
        }
        "plotboard.json" => {
            value.get("kind").and_then(Value::as_str) == Some("plotboard")
                && ["storylines", "cards"]
                    .iter()
                    .all(|key| value.get(key).is_some_and(Value::is_array))
        }
        "timelines.json" => {
            value.get("kind").and_then(Value::as_str) == Some("timelines")
                && ["timelines", "events", "documentLinks", "entityLinks"]
                    .iter()
                    .all(|key| value.get(key).is_some_and(Value::is_array))
        }
        "visual-profiles.json" => {
            value.get("kind").and_then(Value::as_str) == Some("visualProfiles")
                && ["profiles", "stylePresets"]
                    .iter()
                    .all(|key| value.get(key).is_some_and(Value::is_array))
                && validate_visual_data(value)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn validate_visual_data(value: &Value) -> bool {
    let Some(profiles) = value.get("profiles").and_then(Value::as_array) else {
        return false;
    };
    let Some(presets) = value.get("stylePresets").and_then(Value::as_array) else {
        return false;
    };
    profiles.iter().all(|profile| {
        let Some(object) = profile.as_object() else {
            return false;
        };
        object.len() == 4
            && object
                .get("entityId")
                .and_then(Value::as_str)
                .and_then(kraqel_domain::EntityId::parse)
                .is_some()
            && object.get("traits").is_some_and(validate_string_map)
            && object.get("canonicalAssetId").is_some_and(|id| {
                id.is_null()
                    || id
                        .as_str()
                        .and_then(kraqel_domain::EntityId::parse)
                        .is_some()
            })
            && object.get("exclusions").is_some_and(validate_string_list)
    }) && presets.iter().all(|preset| {
        let Some(object) = preset.as_object() else {
            return false;
        };
        let Some(name) = object.get("name").and_then(Value::as_str) else {
            return false;
        };
        let Some(kinds) = object.get("applicableKinds").and_then(Value::as_array) else {
            return false;
        };
        let unique_kinds = kinds
            .iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        object.len() == 4
            && object
                .get("id")
                .and_then(Value::as_str)
                .and_then(kraqel_domain::EntityId::parse)
                .is_some()
            && !name.trim().is_empty()
            && name.chars().count() <= 120
            && object.get("visualRules").is_some_and(validate_string_map)
            && !kinds.is_empty()
            && kinds.len() <= 4
            && unique_kinds.len() == kinds.len()
            && unique_kinds
                .iter()
                .all(|kind| matches!(*kind, "character" | "location" | "faction" | "item"))
    })
}

fn validate_string_map(value: &Value) -> bool {
    value.as_object().is_some_and(|map| {
        map.len() <= 32
            && map.iter().all(|(key, value)| {
                !key.trim().is_empty()
                    && key.chars().count() <= 80
                    && value
                        .as_str()
                        .is_some_and(|text| !text.trim().is_empty() && text.chars().count() <= 500)
            })
    })
}

fn validate_string_list(value: &Value) -> bool {
    value.as_array().is_some_and(|items| {
        let strings = items.iter().filter_map(Value::as_str).collect::<Vec<_>>();
        strings.len() == items.len()
            && strings.len() <= 32
            && strings.iter().collect::<BTreeSet<_>>().len() == strings.len()
            && strings
                .iter()
                .all(|text| !text.trim().is_empty() && text.chars().count() <= 500)
    })
}

fn validate_visual_references(root: &Path, assets: &[Value]) -> Result<(), GitServiceError> {
    let story: Value = serde_json::from_slice(&fs::read(root.join("story-bible.json"))?)
        .map_err(|_| GitServiceError::InvalidSnapshot)?;
    let visuals: Value = serde_json::from_slice(&fs::read(root.join("visual-profiles.json"))?)
        .map_err(|_| GitServiceError::InvalidSnapshot)?;
    let entity_ids = story
        .get("entities")
        .and_then(Value::as_array)
        .ok_or(GitServiceError::InvalidSnapshot)?
        .iter()
        .filter_map(|entity| entity.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let asset_ids = assets
        .iter()
        .filter_map(|asset| asset.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let profiles = visuals
        .get("profiles")
        .and_then(Value::as_array)
        .ok_or(GitServiceError::InvalidSnapshot)?;
    if profiles.iter().all(|profile| {
        profile
            .get("entityId")
            .and_then(Value::as_str)
            .is_some_and(|id| entity_ids.contains(id))
            && profile.get("canonicalAssetId").is_none_or(|id| {
                id.is_null() || id.as_str().is_some_and(|id| asset_ids.contains(id))
            })
    }) {
        Ok(())
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn null_device() -> &'static str {
    if cfg!(windows) { "NUL" } else { "/dev/null" }
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), GitServiceError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn parse_json(text: &str) -> rusqlite::Result<Value> {
    serde_json::from_str(text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn nonnegative_u64(value: i64) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn query_rows<F>(
    connection: &Connection,
    sql: &str,
    project_id: &str,
    mut map: F,
) -> Result<Value, GitServiceError>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Value>,
{
    let mut statement = connection.prepare(sql)?;
    Ok(Value::Array(
        statement
            .query_map(params![project_id], |row| map(row))?
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn query_json_rows(
    connection: &Connection,
    sql: &str,
    project_id: &str,
    names: &[&str],
) -> Result<Value, GitServiceError> {
    query_rows(connection, sql, project_id, |row| {
        let mut object = serde_json::Map::new();
        for (index, name) in names.iter().enumerate() {
            let value = if index + 1 == names.len() {
                let text: String = row.get(index)?;
                parse_json(&text)?
            } else {
                Value::String(row.get(index)?)
            };
            object.insert((*name).to_owned(), value);
        }
        Ok(Value::Object(object))
    })
}

fn query_text_rows(
    connection: &Connection,
    sql: &str,
    project_id: &str,
    names: &[&str],
) -> Result<Value, GitServiceError> {
    query_rows(connection, sql, project_id, |row| {
        let mut object = serde_json::Map::new();
        for (index, name) in names.iter().enumerate() {
            object.insert((*name).to_owned(), Value::String(row.get(index)?));
        }
        Ok(Value::Object(object))
    })
}

fn query_mixed_rows(
    connection: &Connection,
    sql: &str,
    project_id: &str,
    names: &[&str],
) -> Result<Value, GitServiceError> {
    query_rows(connection, sql, project_id, |row| {
        let mut object = serde_json::Map::new();
        for (index, name) in names.iter().enumerate() {
            let value_ref = row.get_ref(index)?;
            let value = match value_ref {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(number) => Value::Number(number.into()),
                rusqlite::types::ValueRef::Text(text) => {
                    Value::String(String::from_utf8_lossy(text).into_owned())
                }
                _ => {
                    return Err(rusqlite::Error::InvalidColumnType(
                        index,
                        (*name).to_owned(),
                        value_ref.data_type(),
                    ));
                }
            };
            object.insert((*name).to_owned(), value);
        }
        Ok(Value::Object(object))
    })
}

fn normalize_inline(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_owned()
}
fn normalize_text(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end()
        .to_owned()
}
fn output_text(output: &Output) -> Result<String, GitServiceError> {
    String::from_utf8(output.stdout.clone()).map_err(|_| GitServiceError::GitFailed)
}

fn summarize(
    root: &Path,
    format_version: u16,
    project_id: ProjectId,
) -> Result<SnapshotSummary, GitServiceError> {
    let mut file_count = 0_u32;
    let mut byte_count = 0_u64;
    summarize_directory(root, &mut file_count, &mut byte_count)?;
    Ok(SnapshotSummary {
        format_version,
        project_id,
        file_count,
        byte_count,
    })
}

fn summarize_directory(
    path: &Path,
    files: &mut u32,
    bytes: &mut u64,
) -> Result<(), GitServiceError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            summarize_directory(&entry.path(), files, bytes)?;
        } else if metadata.is_file() {
            *files = files
                .checked_add(1)
                .ok_or(GitServiceError::InvalidSnapshot)?;
            *bytes = bytes
                .checked_add(metadata.len())
                .ok_or(GitServiceError::InvalidSnapshot)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{GitService, GitServiceError};
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId, Project, ProjectId};
    use kraqel_storage::project_manager::ProjectManager;
    use sha2::{Digest as _, Sha256};
    use std::{collections::BTreeMap, fs};
    use tempfile::tempdir;

    fn fixture() -> (tempfile::TempDir, ProjectId) {
        let root = tempdir().unwrap();
        let manager = ProjectManager::new(root.path()).unwrap();
        let id = ProjectId::new();
        let project = Project {
            id,
            name: "Byte-stabiler Kolkrabe".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let opened = manager.create_project(id.to_string(), &project).unwrap();
        opened.database().connection().execute(
            "INSERT INTO document (id, project_id, parent_id, kind, title, position, revision, content_json, plain_text, created_at, updated_at) VALUES (?1, ?2, NULL, 'chapter', 'Kapitel', 0, 0, ?3, 'Text', 'volatile-a', 'volatile-b')",
            rusqlite::params![EntityId::new().to_string(), id.to_string(), r#"{"version":1,"document":{"type":"doc","content":[]}}"#],
        ).unwrap();
        drop(opened);
        (root, id)
    }

    fn read_tree(path: &Path) -> BTreeMap<String, Vec<u8>> {
        fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    visit(root, &entry.path(), result);
                } else {
                    result.insert(
                        entry
                            .path()
                            .strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/"),
                        fs::read(entry.path()).unwrap(),
                    );
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(path, path, &mut result);
        result
    }

    use std::path::Path;

    #[test]
    fn snapshot_is_byte_stable_and_git_never_tracks_database() {
        let (root, id) = fixture();
        let manager = ProjectManager::new(root.path()).unwrap();
        let opened = manager.open_project(id.to_string()).unwrap();
        let service = GitService::new(opened.directory());
        let first = service.create_snapshot(opened.database(), id).unwrap();
        let first_tree = read_tree(&opened.directory().join("snapshot"));
        opened
            .database()
            .connection()
            .execute(
                "UPDATE project SET updated_at = 'anderer-fluechtiger-zeitwert' WHERE id = ?1",
                [id.to_string()],
            )
            .unwrap();
        let second = service.create_snapshot(opened.database(), id).unwrap();
        assert_eq!(first, second);
        assert_eq!(first_tree, read_tree(&opened.directory().join("snapshot")));
        assert!(!service.status().unwrap().clean);
        assert!(service.diff().unwrap().contains("kraqel-project-snapshot"));
        service.commit("Erster Snapshot").unwrap();
        assert!(service.status().unwrap().clean);
        opened
            .database()
            .connection()
            .execute(
                "UPDATE project SET name = 'Fachlich geänderter Kolkrabe' WHERE id = ?1",
                [id.to_string()],
            )
            .unwrap();
        service.create_snapshot(opened.database(), id).unwrap();
        assert!(!service.status().unwrap().clean);
        assert!(
            service
                .diff()
                .unwrap()
                .contains("Fachlich geänderter Kolkrabe")
        );
        let tracked = super::output_text(&service.run_git(&["ls-files"]).unwrap()).unwrap();
        assert!(!tracked.contains("sqlite"));
        assert!(!tracked.contains("backups"));
    }

    #[test]
    fn validation_rejects_manipulated_manifest() {
        let (root, id) = fixture();
        let manager = ProjectManager::new(root.path()).unwrap();
        let opened = manager.open_project(id.to_string()).unwrap();
        let service = GitService::new(opened.directory());
        service.create_snapshot(opened.database(), id).unwrap();
        fs::write(opened.directory().join("snapshot/manifest.json"), b"{}\n").unwrap();
        assert!(matches!(
            service.validate_snapshot(id),
            Err(GitServiceError::InvalidSnapshot)
        ));
    }

    #[test]
    fn validation_keeps_reading_legacy_v1_snapshots() {
        let (root, id) = fixture();
        let manager = ProjectManager::new(root.path()).unwrap();
        let opened = manager.open_project(id.to_string()).unwrap();
        let service = GitService::new(opened.directory());
        service.create_snapshot(opened.database(), id).unwrap();
        let snapshot = opened.directory().join("snapshot");
        fs::remove_file(snapshot.join("visual-profiles.json")).unwrap();
        fs::write(
            snapshot.join("schema.json"),
            include_bytes!("../../../docs/data-format/snapshot-v1.schema.json"),
        )
        .unwrap();
        let manifest_path = snapshot.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["formatVersion"] = serde_json::json!(1);
        manifest["files"] = serde_json::json!(super::SNAPSHOT_FILES_V1);
        super::write_json(manifest_path, &manifest).unwrap();

        let summary = service
            .validate_snapshot(id)
            .expect("v1 should remain readable");
        assert_eq!(summary.format_version, 1);
    }

    #[test]
    fn snapshot_copies_assets_by_hash_and_rejects_tampering() {
        let (root, id) = fixture();
        let manager = ProjectManager::new(root.path()).unwrap();
        let opened = manager.open_project(id.to_string()).unwrap();
        let bytes = b"portable image bytes";
        let checksum = format!("{:x}", Sha256::digest(bytes));
        fs::create_dir_all(opened.directory().join("assets/thumbnails")).unwrap();
        fs::write(opened.directory().join("assets/source.png"), bytes).unwrap();
        fs::write(
            opened.directory().join("assets/thumbnails/source.png"),
            b"thumbnail",
        )
        .unwrap();
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO media_asset (
                id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                mime_type, checksum, byte_count, width, height, provenance, created_at, updated_at
             ) VALUES (?1, ?2, 'image', 'assets/source.png', 'assets/thumbnails/source.png',
                       'source.png', 'Quelle', 'image/png', ?3, ?4, 1, 1, 'imported', 't', 't')",
                rusqlite::params![
                    EntityId::new().to_string(),
                    id.to_string(),
                    checksum,
                    i64::try_from(bytes.len()).unwrap()
                ],
            )
            .unwrap();
        let service = GitService::new(opened.directory());
        service.create_snapshot(opened.database(), id).unwrap();
        let snapshot_asset = opened
            .directory()
            .join(format!("snapshot/assets/{checksum}.png"));
        assert_eq!(fs::read(&snapshot_asset).unwrap(), bytes);
        fs::write(&snapshot_asset, b"tampered").unwrap();
        assert!(matches!(
            service.validate_snapshot(id),
            Err(GitServiceError::InvalidSnapshot)
        ));
    }

    #[test]
    fn snapshot_contains_profiles_and_rejects_broken_references() {
        let (root, id) = fixture();
        let manager = ProjectManager::new(root.path()).unwrap();
        let opened = manager.open_project(id.to_string()).unwrap();
        let entity_id = EntityId::new();
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO entity (id, project_id, kind, name, created_at, updated_at)
             VALUES (?1, ?2, 'character', 'Rhea', '0', '0')",
                rusqlite::params![entity_id.to_string(), id.to_string()],
            )
            .unwrap();
        opened
            .database()
            .connection()
            .execute(
                "INSERT INTO visual_profile (
                entity_id, project_id, traits_json, exclusions_json, created_at, updated_at
             ) VALUES (?1, ?2, '{\"Augen\":\"bernsteinfarben\"}', '[\"blaue Augen\"]', '0', '0')",
                rusqlite::params![entity_id.to_string(), id.to_string()],
            )
            .unwrap();
        opened.database().connection().execute(
            "INSERT INTO style_preset (
                id, project_id, name, visual_rules_json, applicable_kinds_json, created_at, updated_at
             ) VALUES (?1, ?2, 'Noir', '{\"Palette\":\"monochrom\"}', '[\"character\"]', '0', '0')",
            rusqlite::params![EntityId::new().to_string(), id.to_string()],
        ).unwrap();

        let service = GitService::new(opened.directory());
        service.create_snapshot(opened.database(), id).unwrap();
        let path = opened.directory().join("snapshot/visual-profiles.json");
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("bernsteinfarben"));
        assert!(text.contains("Noir"));
        fs::write(
            &path,
            text.replace(&entity_id.to_string(), &EntityId::new().to_string()),
        )
        .unwrap();
        assert!(matches!(
            service.validate_snapshot(id),
            Err(GitServiceError::InvalidSnapshot)
        ));
    }
}
