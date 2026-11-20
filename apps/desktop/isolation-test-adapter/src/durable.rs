//! Research-only durable consumption and conservative process-crash reconciliation.
//! Native callers supply the private root; renderer input never supplies a filesystem path.
use super::{ImportReceipt, OneShotAdapter, TaskControl, TaskError, check_control};
use crate::FileTask;
use kraqel_storage::project_manager::ProjectManager;
use rusqlite::{Connection, OpenFlags, OptionalExtension as _, params};
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use uuid::Uuid;

type Result<T> = std::result::Result<T, TaskError>;
fn unavailable<T>(_: T) -> TaskError {
    TaskError::StateUnavailable
}

/// Native fault-injection checkpoints; no renderer command or environment switch.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskMilestone {
    Prepared,
    CommitIntent,
    Published,
}

/// One exclusive owner for a persistent journal and its private staging tree.
/// An interrupted attempt is never resumed or granted a second read.
pub struct DurableStore {
    root: PathBuf,
    projects: PathBuf,
    journal: Connection,
    _lease: Connection,
    active: Option<Uuid>,
    executed: bool,
}

impl DurableStore {
    /// Opens and reconciles the journal before any new native selection or stream read.
    /// # Errors
    /// Fails closed on contention, unsafe paths, unknown/corrupt state or ambiguous publication.
    pub fn open(root: &Path, manager: &ProjectManager) -> Result<Self> {
        fs::create_dir_all(root).map_err(unavailable)?;
        safe_tree(root)?;
        let root = root.canonicalize().map_err(unavailable)?;
        if root.starts_with(manager.root()) || manager.root().starts_with(&root) {
            return Err(TaskError::StateUnavailable);
        }
        let existing = fs::read_dir(&root).map_err(unavailable)?.next().is_some();
        let path = root.join("tasks.sqlite3");
        let exists = path.try_exists().map_err(unavailable)?;
        // Never recreate a disappeared journal inside an already used private root.
        if existing && !exists {
            return Err(TaskError::StateUnavailable);
        }
        let lease = connect(&root.join("owner.sqlite3"))?;
        lease.busy_timeout(Duration::ZERO).map_err(unavailable)?;
        lease
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(unavailable)?;
        let journal = connect(&path)?;
        journal
            .execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")
            .map_err(unavailable)?;
        let binding = format!(
            "{:x}",
            Sha256::digest(manager.root().as_os_str().as_encoded_bytes())
        );
        if !exists {
            journal.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE binding (root_hash TEXT NOT NULL);
                CREATE TABLE task (id TEXT PRIMARY KEY NOT NULL,
                    state TEXT NOT NULL CHECK (state IN ('started','committing','succeeded','failed','interrupted')),
                    receipt TEXT, error TEXT);
                PRAGMA user_version=1;").map_err(unavailable)?;
            journal
                .execute("INSERT INTO binding VALUES (?1)", [&binding])
                .map_err(unavailable)?;
            journal.execute_batch("COMMIT").map_err(unavailable)?;
        }
        let version: u32 = journal
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(unavailable)?;
        let bindings: Vec<String> = journal
            .prepare("SELECT root_hash FROM binding")
            .map_err(unavailable)?
            .query_map([], |r| r.get(0))
            .map_err(unavailable)?
            .collect::<std::result::Result<_, _>>()
            .map_err(unavailable)?;
        if version != 1 || bindings != [binding] {
            return Err(TaskError::StateUnavailable);
        }
        let integrity: String = journal
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(unavailable)?;
        if integrity != "ok" {
            return Err(TaskError::StateUnavailable);
        }
        fs::create_dir_all(root.join("staging")).map_err(unavailable)?;
        let mut store = Self {
            root,
            projects: manager.root().into(),
            journal,
            _lease: lease,
            active: None,
            executed: false,
        };
        store.reconcile()?;
        Ok(store)
    }

    /// Durably consumes a controller-bound ID before opening the dialog or reading a byte.
    /// # Errors
    /// Duplicate IDs, invalid identities and unavailable storage are rejected without reading.
    pub fn claim(&mut self, id: Uuid) -> Result<()> {
        if id.is_nil() {
            return Err(TaskError::InvalidTask);
        }
        if self.active.is_some() {
            return Err(TaskError::AlreadyConsumed);
        }
        let exists: bool = self
            .journal
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM task WHERE id=?1)",
                [id.to_string()],
                |r| r.get(0),
            )
            .map_err(unavailable)?;
        if exists {
            return Err(TaskError::AlreadyConsumed);
        }
        let count: u32 = self
            .journal
            .query_row("SELECT count(*) FROM task", [], |r| r.get(0))
            .map_err(unavailable)?;
        if count >= 4096 {
            return Err(TaskError::StateUnavailable);
        }
        self.journal
            .execute(
                "INSERT INTO task(id,state) VALUES (?1,'started')",
                [id.to_string()],
            )
            .map_err(unavailable)?;
        self.active = Some(id);
        self.executed = false;
        Ok(())
    }

    /// Records an early selection/transport refusal. The ID remains consumed forever.
    /// # Errors
    /// A missing active claim or persistence failure is rejected.
    pub fn refuse(&mut self, error: TaskError) -> Result<()> {
        let id = self.active.ok_or(TaskError::AlreadyConsumed)?;
        let state: String = self
            .journal
            .query_row(
                "SELECT state FROM task WHERE id=?1",
                [id.to_string()],
                |r| r.get(0),
            )
            .map_err(unavailable)?;
        if state != "started" {
            return Err(TaskError::AlreadyConsumed);
        }
        self.clean_stage(id)?;
        self.update(id, "failed", None, Some(error))?;
        self.active = None;
        Ok(())
    }

    /// Executes into private staging; persists publication intent before the final rename.
    /// # Errors
    /// All read/import/control/journal failures are data-poor. Ambiguous commit errors leave
    /// the intent intact for reconciliation instead of permitting another import.
    pub fn execute(
        &mut self,
        task: &FileTask,
        input: &mut impl Read,
        mut control: impl FnMut() -> TaskControl,
        before_publish: impl FnOnce() -> TaskControl,
        mut observe: impl FnMut(TaskMilestone),
    ) -> Result<ImportReceipt> {
        let id = self.active.ok_or(TaskError::AlreadyConsumed)?;
        if self.executed {
            return Err(TaskError::AlreadyConsumed);
        }
        self.executed = true;
        if task.task_id != id {
            self.refuse(TaskError::InvalidTask)?;
            return Err(TaskError::InvalidTask);
        }
        let stage = self.stage(id);
        // A claim is committed before this directory can contain any imported data.
        fs::create_dir(&stage).map_err(unavailable)?;
        let manager = ProjectManager::new(&stage).map_err(unavailable)?;
        let prepared = OneShotAdapter::new(id).execute(task, input, &manager, &mut control);
        let receipt = match prepared {
            Ok(receipt) => receipt,
            Err(error) => {
                self.refuse(error)?;
                return Err(error);
            }
        };
        observe(TaskMilestone::Prepared);
        if let Err(error) = check_control(task, before_publish()) {
            self.refuse(error)?;
            return Err(error);
        }
        // Persist the complete receipt before the only move into the visible project root.
        self.update(id, "committing", Some(&receipt), None)?;
        observe(TaskMilestone::CommitIntent);
        let source = stage.join(receipt.project_id.to_string());
        let target = self.projects.join(receipt.project_id.to_string());
        safe_tree(&source)?;
        if target.try_exists().map_err(unavailable)? {
            return Err(TaskError::StateUnavailable);
        }
        fs::rename(&source, &target).map_err(unavailable)?;
        observe(TaskMilestone::Published);
        self.verify_published(&receipt)?;
        self.clean_stage(id)?;
        self.update(id, "succeeded", Some(&receipt), None)?;
        self.active = None;
        Ok(receipt)
    }

    /// Returns only a persisted terminal receipt or refusal, never a new authorization.
    /// # Errors
    /// Unknown IDs and unreadable/unfinished journal entries fail closed.
    pub fn outcome(&self, id: Uuid) -> Result<std::result::Result<ImportReceipt, TaskError>> {
        let row: Option<(String, Option<String>, Option<String>)> = self
            .journal
            .query_row(
                "SELECT state,receipt,error FROM task WHERE id=?1",
                [id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(unavailable)?;
        match row {
            Some((state, Some(receipt), None)) if state == "succeeded" => {
                Ok(Ok(decode_receipt(id, &receipt)?))
            }
            Some((state, None, Some(error))) if state == "failed" || state == "interrupted" => {
                Ok(Err(serde_json::from_str(&error).map_err(unavailable)?))
            }
            _ => Err(TaskError::StateUnavailable),
        }
    }

    fn stage(&self, id: Uuid) -> PathBuf {
        self.root.join("staging").join(id.to_string())
    }
    fn clean_stage(&self, id: Uuid) -> Result<()> {
        let path = self.stage(id);
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(TaskError::StateUnavailable),
            Ok(_) => (),
        }
        safe_tree(&path)?;
        let canonical = path.canonicalize().map_err(unavailable)?;
        if canonical.parent() != Some(self.root.join("staging").as_path()) {
            return Err(TaskError::StateUnavailable);
        }
        fs::remove_dir_all(path).map_err(unavailable)?;
        Ok(())
    }
    fn update(
        &self,
        id: Uuid,
        state: &str,
        receipt: Option<&ImportReceipt>,
        error: Option<TaskError>,
    ) -> Result<()> {
        let receipt = receipt
            .map(serde_json::to_string)
            .transpose()
            .map_err(unavailable)?;
        let error = error
            .map(|e| serde_json::to_string(&e))
            .transpose()
            .map_err(unavailable)?;
        let changed = self
            .journal
            .execute(
                "UPDATE task SET state=?2,receipt=?3,error=?4 WHERE id=?1",
                params![id.to_string(), state, receipt, error],
            )
            .map_err(unavailable)?;
        if changed != 1 {
            return Err(TaskError::StateUnavailable);
        }
        Ok(())
    }
    fn reconcile(&mut self) -> Result<()> {
        let rows: Vec<(String, String, Option<String>)> = self
            .journal
            .prepare("SELECT id,state,receipt FROM task")
            .map_err(unavailable)?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(unavailable)?
            .collect::<std::result::Result<_, _>>()
            .map_err(unavailable)?;
        // Unknown staging directories are never guessed to be disposable.
        for entry in fs::read_dir(self.root.join("staging")).map_err(unavailable)? {
            let entry = entry.map_err(unavailable)?;
            if !rows
                .iter()
                .any(|(id, _, _)| entry.file_name() == id.as_str())
            {
                return Err(TaskError::StateUnavailable);
            }
        }
        for (id, state, receipt) in rows {
            let parsed = Uuid::parse_str(&id).map_err(unavailable)?;
            if parsed.is_nil() || parsed.to_string() != id {
                return Err(TaskError::StateUnavailable);
            }
            match state.as_str() {
                "started" if receipt.is_none() => {
                    self.clean_stage(parsed)?;
                    self.update(parsed, "interrupted", None, Some(TaskError::Interrupted))?;
                }
                "committing" => {
                    let receipt = decode_receipt(
                        parsed,
                        receipt.as_deref().ok_or(TaskError::StateUnavailable)?,
                    )?;
                    if self
                        .projects
                        .join(receipt.project_id.to_string())
                        .try_exists()
                        .map_err(unavailable)?
                    {
                        self.verify_published(&receipt)?;
                        self.clean_stage(parsed)?;
                        self.update(parsed, "succeeded", Some(&receipt), None)?;
                    } else {
                        self.clean_stage(parsed)?;
                        self.update(parsed, "interrupted", None, Some(TaskError::Interrupted))?;
                    }
                }
                "succeeded" | "failed" | "interrupted" => {
                    let _ = self.outcome(parsed)?;
                }
                _ => return Err(TaskError::StateUnavailable),
            }
        }
        Ok(())
    }
    fn verify_published(&self, receipt: &ImportReceipt) -> Result<()> {
        let path = self.projects.join(receipt.project_id.to_string());
        safe_tree(&path)?;
        let db = Connection::open_with_flags(
            path.join("kraqel.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(unavailable)?;
        let integrity: String = db
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(unavailable)?;
        let id: String = db
            .query_row("SELECT id FROM project", [], |r| r.get(0))
            .map_err(unavailable)?;
        let documents: u32 = db
            .query_row("SELECT count(*) FROM document", [], |r| r.get(0))
            .map_err(unavailable)?;
        let entities: u32 = db
            .query_row("SELECT count(*) FROM entity", [], |r| r.get(0))
            .map_err(unavailable)?;
        let assets: u32 = db
            .query_row("SELECT count(*) FROM media_asset", [], |r| r.get(0))
            .map_err(unavailable)?;
        let foreign_key_error = db
            .prepare("PRAGMA foreign_key_check")
            .map_err(unavailable)?
            .exists([])
            .map_err(unavailable)?;
        if integrity != "ok"
            || foreign_key_error
            || id != receipt.project_id.to_string()
            || documents != receipt.document_count
            || entities != receipt.entity_count
            || assets != receipt.asset_count
        {
            return Err(TaskError::StateUnavailable);
        }
        Ok(())
    }
}

fn decode_receipt(id: Uuid, text: &str) -> Result<ImportReceipt> {
    if text.len() > 2048 {
        return Err(TaskError::StateUnavailable);
    }
    let receipt: ImportReceipt = serde_json::from_str(text).map_err(unavailable)?;
    if receipt.task_id != id
        || receipt.contract_version != 1
        || receipt.byte_count == 0
        || receipt.byte_count > super::MAX_TASK_BYTES
        || receipt.sha256.len() != 64
        || !receipt.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(TaskError::StateUnavailable);
    }
    Ok(receipt)
}

fn connect(path: &Path) -> Result<Connection> {
    for suffix in ["", "-journal", "-wal", "-shm"] {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        let path = PathBuf::from(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() && !linked(&metadata) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Ok(_) | Err(_) => return Err(TaskError::StateUnavailable),
        }
    }
    Connection::open(path).map_err(unavailable)
}
fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
fn safe_tree(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(unavailable)?;
    if linked(&metadata) || (!metadata.is_file() && !metadata.is_dir()) {
        return Err(TaskError::StateUnavailable);
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(unavailable)? {
            safe_tree(&entry.map_err(unavailable)?.path())?;
        }
    }
    Ok(())
}
