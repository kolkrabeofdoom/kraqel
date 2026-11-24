//! Read-only acquisition of a closed Windows project into a distinct local catalogue.
use super::{handoff, *};
use kraqel_domain::ProjectId;
use kraqel_windows_inventory::{TreeChange, is_local_drive};
use serde::Serialize;
use std::{collections::BTreeMap, ffi::OsStr, io::Read, path::Prefix};
use winapi_util::file::information;

const DB_LIMIT: u64 = 128 * 1024 * 1024;
const WORK_LIMIT: u64 = 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 16_384;

fn invalid() -> ProjectError {
    ProjectError::InvalidImportedData
}
fn io_error(source: io::Error) -> ProjectError {
    ProjectError::Io {
        operation: "Bestandsprojekt lesen",
        source,
    }
}

fn identity(file: &fs::File) -> Result<(u64, u64), ProjectError> {
    let info = information(file).map_err(io_error)?;
    if info.file_index() == 0 {
        return Err(invalid());
    }
    Ok((info.volume_serial_number(), info.file_index()))
}

/// Validates one native filename component before local filesystem access.
/// # Errors
/// Rejects device names, alternate streams and ambiguous or invalid path components.
pub fn safe_existing_name(name: &OsStr) -> Result<&str, ProjectError> {
    let name = name.to_str().ok_or_else(invalid)?;
    if name.is_empty()
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || r#"<>:"/\|?*"#.contains(c))
    {
        return Err(invalid());
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || stem
        .strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
        .is_some_and(|n| {
            n.len() == 1 && n.as_bytes()[0].is_ascii_digit() || matches!(n, "¹" | "²" | "³")
        })
    {
        return Err(invalid());
    }
    Ok(name)
}

/// Pins each directory of a local drive path before descending to the next.
/// # Errors
/// Rejects redirected paths, recall/reparse entries and non-local drives.
pub fn pin_existing_folder_path(path: &Path) -> Result<Vec<fs::File>, ProjectError> {
    let mut parts = path.components();
    let Some(Component::Prefix(prefix)) = parts.next() else {
        return Err(invalid());
    };
    let (Prefix::Disk(letter) | Prefix::VerbatimDisk(letter)) = prefix.kind() else {
        return Err(invalid());
    };
    if !letter.is_ascii_alphabetic()
        || !matches!(parts.next(), Some(Component::RootDir))
        || !is_local_drive(letter)
    {
        return Err(invalid());
    }
    let mut current = PathBuf::from(format!(r"\\?\{}:\", char::from(letter)));
    let mut pins = vec![handoff::pin_local_directory(&current)?];
    for part in parts {
        let Component::Normal(name) = part else {
            return Err(invalid());
        };
        current.push(safe_existing_name(name)?);
        pins.push(handoff::pin_local_directory(&current)?);
    }
    Ok(pins)
}

/// Categories whose contents require separate explicit archive consent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExistingExtraGroup {
    Backups,
    Exports,
    Git,
    Other,
}

/// Counts only: filenames and content stay within the native selection.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExistingExtraCounts {
    pub backups: u64,
    pub exports: u64,
    pub git: u64,
    pub other: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExistingFolderPreview {
    pub source_sha256: String,
    pub project_id: ProjectId,
    pub documents: u64,
    pub images: u64,
    pub extras: ExistingExtraCounts,
}

/// Pinned source entry exposed to the native archive callback only.
pub struct ExistingExtraFile<'a> {
    relative: &'a str,
    file: &'a fs::File,
    group: ExistingExtraGroup,
}

impl ExistingExtraFile<'_> {
    #[must_use]
    pub fn relative_path(&self) -> &Path {
        Path::new(self.relative)
    }
    #[must_use]
    pub fn group(&self) -> ExistingExtraGroup {
        self.group
    }
    /// Opens the already bound file for the selected archive group.
    /// # Errors
    /// Returns an error if the handle cannot be duplicated.
    pub fn open(&self) -> Result<fs::File, ProjectError> {
        self.file.try_clone().map_err(io_error)
    }
}

fn group(relative: &str) -> ExistingExtraGroup {
    match relative
        .split('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "backups" => ExistingExtraGroup::Backups,
        "exports" => ExistingExtraGroup::Exports,
        ".git" => ExistingExtraGroup::Git,
        _ => ExistingExtraGroup::Other,
    }
}

struct Tree {
    files: BTreeMap<String, fs::File>,
    directories: BTreeMap<String, fs::File>,
    changes: TreeChange,
}

impl Tree {
    fn collect(root: &Path) -> Result<Self, ProjectError> {
        let mut tree = Self {
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
            changes: TreeChange::watch(root).map_err(io_error)?,
        };
        tree.walk(root, "", 0)?;
        Ok(tree)
    }
    fn walk(&mut self, root: &Path, rel: &str, depth: usize) -> Result<(), ProjectError> {
        if depth > 64 {
            return Err(invalid());
        }
        for entry in fs::read_dir(root.join(rel)).map_err(io_error)? {
            if self.files.len() + self.directories.len() >= MAX_ENTRIES {
                return Err(invalid());
            }
            let entry = entry.map_err(io_error)?;
            let base = entry.file_name();
            let name = safe_existing_name(&base)?;
            let relative = if rel.is_empty() {
                name.to_owned()
            } else {
                format!("{rel}/{name}")
            };
            let path = entry.path();
            if candidate_attributes(&path)?.is_dir() {
                let pin = handoff::pin_local_directory(&path)?;
                self.directories.insert(relative.clone(), pin);
                self.walk(root, &relative, depth + 1)?;
            } else {
                self.files
                    .insert(relative, handoff::pinned_file(&path, false)?);
            }
        }
        Ok(())
    }

    fn ensure_unchanged(&self) -> Result<(), ProjectError> {
        if self.changes.is_unchanged() {
            Ok(())
        } else {
            Err(invalid())
        }
    }

    fn read_sqlite(
        &self,
        source: &Path,
    ) -> Result<(Connection, BTreeMap<String, String>), ProjectError> {
        let main = DATABASE_FILE_NAME;
        let wal = format!("{main}-wal");
        let shm = format!("{main}-shm");
        if self.files.keys().any(|name| {
            (sqlite_component(&name.to_ascii_lowercase()) && name != &name.to_ascii_lowercase())
                || name.eq_ignore_ascii_case(&format!("{main}-journal"))
        }) {
            return Err(invalid());
        }
        let mut work = BTreeMap::new();
        let mut main_file = self.files.get(main).ok_or_else(invalid)?;
        let size = main_file.metadata().map_err(io_error)?.len();
        if !(100..=DB_LIMIT).contains(&size) {
            return Err(invalid());
        }
        let mut image = Vec::new();
        (&mut main_file)
            .take(size + 1)
            .read_to_end(&mut image)
            .map_err(io_error)?;
        if image.len() as u64 != size || !image.starts_with(b"SQLite format 3\0") {
            return Err(invalid());
        }
        work.insert(main.to_owned(), format!("{:x}", Sha256::digest(&image)));
        let wal_bytes = self
            .files
            .get(&wal)
            .map(|file| file.metadata().map(|m| m.len()))
            .transpose()
            .map_err(io_error)?
            .unwrap_or(0);
        let mut db = Connection::open_in_memory()?;
        if wal_bytes == 0 {
            if self.files.contains_key(&shm) && !self.files.contains_key(&wal) {
                return Err(invalid());
            }
            // A checkpointed database can retain WAL mode in the header after the last close.
            // No WAL file carries committed pages. Deserialize the captured main image in memory.
            match &image[18..20] {
                [1, 1] => (),
                [2, 2] => image[18..20].copy_from_slice(&[1, 1]),
                _ => return Err(invalid()),
            }
            db.deserialize_read_exact(MAIN_DB, image.as_slice(), image.len(), true)?;
        } else {
            if wal_bytes > DB_LIMIT || image[18..20] != [2, 2] || !self.files.contains_key(&shm) {
                return Err(invalid());
            }
            drop(image);
            let file = self.files.get(&wal).ok_or_else(invalid)?;
            let mut digest = Sha256::new();
            let read = io::copy(&mut file.take(wal_bytes + 1), &mut digest).map_err(io_error)?;
            if read != wal_bytes {
                return Err(invalid());
            }
            work.insert(wal, format!("{:x}", digest.finalize()));
            // Same read-only WAL path as the P3 fixture. All three original files deny
            // writes and replacement. SQLite can use only a readable existing WAL index.
            // The backup API materializes the committed pages entirely in memory.
            let reader =
                Connection::open_with_flags(source.join(main), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            reader.execute_batch(
                "PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA query_only=ON; BEGIN",
            )?;
            let pages: u32 = reader.pragma_query_value(None, "page_count", |r| r.get(0))?;
            let size: u32 = reader.pragma_query_value(None, "page_size", |r| r.get(0))?;
            let mode: String = reader.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
            if mode != "wal" || pages == 0 || u64::from(size) * u64::from(pages) > DB_LIMIT {
                return Err(invalid());
            }
            let backup = rusqlite::backup::Backup::new(&reader, &mut db)?;
            if !matches!(backup.step(-1)?, rusqlite::backup::StepResult::Done) {
                return Err(invalid());
            }
        }
        db.execute_batch(
            "PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA query_only=ON",
        )?;
        Ok((db, work))
    }
}

struct Capture {
    tree: Tree,
    preview: ExistingFolderPreview,
    media: BTreeMap<String, String>,
    fresh: Database,
    _pins: Vec<fs::File>,
    source_id: (u64, u64),
    target_id: (u64, u64),
}

/// One-shot native lease. Dropping it releases the watched tree and all reader handles.
pub struct ExistingFolderGuard(Capture);

fn sqlite_component(name: &str) -> bool {
    name == DATABASE_FILE_NAME || matches!(name, "kraqel.sqlite3-wal" | "kraqel.sqlite3-shm")
}

impl ProjectManager {
    fn capture_existing(&self, source: &Path) -> Result<Capture, ProjectError> {
        let mut pins = pin_existing_folder_path(source)?;
        let destination = pin_existing_folder_path(&self.root)?;
        let source_id = identity(pins.last().ok_or_else(invalid)?)?;
        let target_id = identity(destination.last().ok_or_else(invalid)?)?;
        for pin in &pins {
            if identity(pin)? == target_id {
                return Err(invalid());
            }
        }
        for pin in &destination {
            if identity(pin)? == source_id {
                return Err(invalid());
            }
        }
        pins.extend(destination);
        let name = source
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(invalid)?;
        let id = ProjectId::parse(name)
            .filter(|id| id.to_string() == name)
            .ok_or_else(invalid)?;
        let tree = Tree::collect(source)?;
        let (db, mut work) = tree.read_sqlite(source)?;
        let fresh = handoff::rebuild(&db, id)?;
        drop(db);
        let docs: u32 = fresh
            .connection()
            .query_row("SELECT COUNT(*) FROM document", [], |r| r.get(0))?;
        let images: u32 =
            fresh
                .connection()
                .query_row("SELECT COUNT(*) FROM media_asset", [], |r| r.get(0))?;
        let mut budget = 0u64;
        let mut stmt = fresh
            .connection()
            .prepare("SELECT relative_path,thumbnail_path FROM media_asset")?;
        for pair in stmt.query_map([], |r| Ok([r.get::<_, String>(0)?, r.get(1)?]))? {
            for path in pair? {
                let file = tree.files.get(&path).ok_or_else(invalid)?;
                budget = budget
                    .checked_add(file.metadata().map_err(io_error)?.len())
                    .ok_or_else(invalid)?;
                if budget > WORK_LIMIT {
                    return Err(invalid());
                }
            }
        }
        drop(stmt);
        let media = handoff::media_inventory(fresh.connection(), source, None, None)?;
        work.extend(media.clone());
        let mut digest = Sha256::new();
        let mut extras = ExistingExtraCounts::default();
        digest.update(b"kraqel-existing-folder-v1\0");
        digest.update(id.as_uuid().as_bytes());
        digest.update(source_id.0.to_le_bytes());
        digest.update(source_id.1.to_le_bytes());
        for (dir, entries) in [(true, &tree.directories), (false, &tree.files)] {
            for (name, pin) in entries {
                let info = information(pin).map_err(io_error)?;
                digest.update([u8::from(dir)]);
                digest.update((name.len() as u64).to_le_bytes());
                digest.update(name.as_bytes());
                digest.update(info.file_index().to_le_bytes());
                digest.update(info.volume_serial_number().to_le_bytes());
                digest.update(info.file_size().to_le_bytes());
                digest.update(info.last_write_time().ok_or_else(invalid)?.to_le_bytes());
                if let Some(sha) = work.get(name) {
                    digest.update(sha.as_bytes());
                } else if !dir && !sqlite_component(name) {
                    *match group(name) {
                        ExistingExtraGroup::Backups => &mut extras.backups,
                        ExistingExtraGroup::Exports => &mut extras.exports,
                        ExistingExtraGroup::Git => &mut extras.git,
                        ExistingExtraGroup::Other => &mut extras.other,
                    } += 1;
                }
            }
        }
        tree.ensure_unchanged()?;
        Ok(Capture {
            tree,
            fresh,
            media,
            preview: ExistingFolderPreview {
                source_sha256: format!("{:x}", digest.finalize()),
                project_id: id,
                documents: docs.into(),
                images: images.into(),
                extras,
            },
            _pins: pins,
            source_id,
            target_id,
        })
    }

    /// Computes a metadata-only preview from a bounded, pinned source tree.
    /// # Errors
    /// Rejects inconsistent, active, redirected, or unsupported project sources.
    pub fn preview_existing_folder(
        &self,
        source: &Path,
    ) -> Result<(ExistingFolderPreview, ExistingFolderGuard), ProjectError> {
        let capture = self.capture_existing(source)?;
        Ok((capture.preview.clone(), ExistingFolderGuard(capture)))
    }

    /// Publishes working data under a free project UUID after rechecking the preview.
    /// # Errors
    /// Changes before confirmation or during the copy discard the staged project.
    pub fn receive_existing_folder(
        &self,
        source: &Path,
        expected: &str,
        guard: ExistingFolderGuard,
        extra_action: impl FnOnce(&[ExistingExtraFile<'_>]) -> Result<(), ProjectError>,
    ) -> Result<ProjectId, ProjectError> {
        let capture = guard.0;
        capture.tree.ensure_unchanged()?;
        let source_pins = pin_existing_folder_path(source)?;
        let target_pins = pin_existing_folder_path(&self.root)?;
        if expected != capture.preview.source_sha256
            || identity(source_pins.last().ok_or_else(invalid)?)? != capture.source_id
            || identity(target_pins.last().ok_or_else(invalid)?)? != capture.target_id
        {
            return Err(invalid());
        }
        let id = capture.preview.project_id;
        let dest = self.root.join(id.to_string());
        match fs::symlink_metadata(&dest) {
            Ok(_) => return Err(ProjectError::ProjectAlreadyExists),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(io_error(e)),
        }
        let mut stage = TemporaryDirectory::create(&self.root)?;
        fs::create_dir(stage.path().join(BACKUP_DIRECTORY_NAME)).map_err(io_error)?;
        if handoff::media_inventory(
            capture.fresh.connection(),
            source,
            Some(stage.path()),
            Some(&capture.media),
        )? != capture.media
        {
            return Err(invalid());
        }
        handoff::write_database(&capture.fresh, &stage.path().join(DATABASE_FILE_NAME))?;
        let extras = capture
            .tree
            .files
            .iter()
            .filter(|(name, _)| !sqlite_component(name) && !capture.media.contains_key(*name))
            .map(|(name, file)| ExistingExtraFile {
                relative: name,
                file,
                group: group(name),
            })
            .collect::<Vec<_>>();
        capture.tree.ensure_unchanged()?;
        extra_action(&extras)?;
        // Snapshot instant: all data have been read, all source pins are held, and
        // this recursive one-shot signal includes creations and deletions in subdirectories.
        capture.tree.ensure_unchanged()?;
        fs::rename(stage.path(), dest).map_err(io_error)?;
        stage.keep();
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId};

    #[test]
    fn folder_import_binds_wal_and_discards_a_changed_tree() {
        // The desktop sandbox can redirect its system temp directory through a junction.
        // This contract specifically needs a regular local Windows directory.
        let binary = std::env::current_exe().unwrap();
        let test_dir = binary.parent().unwrap();
        let sample = tempfile::tempdir_in(test_dir).unwrap();
        let source = tempfile::tempdir_in(test_dir).unwrap();
        let target = tempfile::tempdir_in(test_dir).unwrap();
        let project = Project {
            id: ProjectId::new(),
            name: "Hafen".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let name = project.id.to_string();
        let manager = ProjectManager::new(sample.path()).unwrap();
        let live = manager.create_project(&name, &project).unwrap();
        let db = live.database().connection();
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA wal_autocheckpoint=0")
            .unwrap();
        db.execute("INSERT INTO document(id,project_id,kind,title,plain_text,position,created_at,updated_at) VALUES(?1,?2,'scene','Hafen','Waltext',0,'t','t')",params![EntityId::new().to_string(),name]).unwrap();
        let folder = source.path().join(&name);
        fs::create_dir(&folder).unwrap();
        let mut original_files = Vec::new();
        for suffix in ["", "-wal", "-shm"] {
            let file = format!("{DATABASE_FILE_NAME}{suffix}");
            let bytes = fs::read(live.directory().join(&file)).unwrap();
            fs::write(folder.join(&file), &bytes).unwrap();
            original_files.push((file, bytes));
        }
        drop(live);
        fs::create_dir(folder.join("backups")).unwrap();
        fs::write(folder.join("backups/frueher"), b"private").unwrap();
        let manager = ProjectManager::new(target.path()).unwrap();
        let (preview, guard) = manager.preview_existing_folder(&folder).unwrap();
        let (_, stale) = manager.preview_existing_folder(&folder).unwrap();
        assert_eq!((preview.documents, preview.extras.backups), (1, 1));
        assert!(
            manager
                .receive_existing_folder(&folder, &preview.source_sha256, guard, |extras| {
                    assert_eq!(extras.len(), 1);
                    fs::write(folder.join("backups/neu"), b"new").unwrap();
                    Ok(())
                })
                .is_err()
        );
        assert!(!target.path().join(&name).exists());
        assert!(
            manager
                .receive_existing_folder(&folder, &preview.source_sha256, stale, |_| panic!(
                    "stale approval"
                ))
                .is_err()
        );
        let (current, guard) = manager.preview_existing_folder(&folder).unwrap();
        assert_eq!(
            manager
                .receive_existing_folder(&folder, &current.source_sha256, guard, |_| Ok(()))
                .unwrap(),
            project.id
        );
        let copy = manager.open_project(&name).unwrap();
        assert_eq!(
            copy.database()
                .connection()
                .query_row("SELECT plain_text FROM document", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "Waltext"
        );
        assert!(!copy.directory().join("backups/frueher").exists());
        for (name, bytes) in original_files {
            assert_eq!(
                Sha256::digest(fs::read(folder.join(name)).unwrap()),
                Sha256::digest(bytes)
            );
        }
    }
}
