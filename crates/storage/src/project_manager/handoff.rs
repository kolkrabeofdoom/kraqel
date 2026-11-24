//! Explizite lokale Vorbereitung des bestätigten Arbeitsstands.
use super::*;
use kraqel_domain::ProjectId;
use rusqlite::{TransactionBehavior, params_from_iter, types::Value};
use std::collections::BTreeMap;
use std::io::Read;

fn io_error(source: io::Error) -> ProjectError {
    ProjectError::Io {
        operation: "Übernahmekopie vorbereiten",
        source,
    }
}

type SchemaObject = (String, String, String, Option<String>);

pub(super) fn schema(db: &Connection) -> Result<Vec<SchemaObject>, ProjectError> {
    let mut stmt =
        db.prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name")?;
    Ok(stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<Result<_, _>>()?)
}

pub(super) fn validate_source(
    source: &Connection,
    reference: &Connection,
    id: ProjectId,
) -> Result<(), ProjectError> {
    if schema(source)? != schema(reference)?
        || source.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))?
            != CURRENT_DATABASE_VERSION.0
    {
        return Err(ProjectError::InvalidDatabase);
    }
    let ids: Vec<String> = source
        .prepare("SELECT id FROM project")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    if ids != [id.to_string()] {
        return Err(ProjectError::InvalidProjectMetadata);
    }
    Ok(())
}

fn rebuild(source: &Connection, id: ProjectId) -> Result<Database, ProjectError> {
    let mut fresh = Database::open_in_memory()?;
    fresh
        .connection()
        .execute_batch("PRAGMA temp_store=MEMORY")?;
    validate_source(source, fresh.connection(), id)?;
    let target = fresh.transaction()?;
    target.execute_batch("PRAGMA defer_foreign_keys=ON")?;
    let mut tables = target.prepare(
        "SELECT s.name FROM sqlite_schema s JOIN pragma_table_list p
        ON p.name=s.name AND p.schema='main' WHERE p.type='table' AND s.type='table'
        AND s.name NOT GLOB 'sqlite_*' ORDER BY s.rowid",
    )?;
    for table in tables.query_map([], |r| r.get::<_, String>(0))? {
        let table = table?;
        let query = format!("SELECT rowid,* FROM \"{table}\" ORDER BY rowid");
        let mut read = source.prepare(&query)?;
        let columns = read.column_count();
        let placeholders = vec!["?"; columns].join(",");
        let names = read
            .column_names()
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(",");
        let mut insert = target.prepare(&format!(
            "INSERT INTO \"{table}\" ({names}) VALUES ({placeholders})"
        ))?;
        let mut rows = read.query([])?;
        while let Some(row) = rows.next()? {
            let values = (0..columns)
                .map(|i| row.get::<_, Value>(i))
                .collect::<Result<Vec<_>, _>>()?;
            insert.execute(params_from_iter(values))?;
        }
    }
    for table in tables.query_map([], |row| row.get::<_, String>(0))? {
        let query = format!("SELECT rowid,* FROM \"{}\" ORDER BY rowid", table?);
        if !same_rows(source, &target, &query)? {
            return Err(ProjectError::InvalidDatabase);
        }
    }
    drop(tables);
    target
        .execute_batch("INSERT INTO document_fts(document_fts,rank) VALUES('integrity-check',1)")?;
    target.commit()?;
    Ok(fresh)
}

pub(super) fn same_rows(
    local: &Connection,
    incoming: &Connection,
    query: &str,
) -> Result<bool, ProjectError> {
    let mut left = local.prepare(query)?;
    let mut right = incoming.prepare(query)?;
    let width = left.column_count();
    let (mut a, mut b) = (left.query([])?, right.query([])?);
    loop {
        match (a.next()?, b.next()?) {
            (None, None) => return Ok(true),
            (Some(a), Some(b)) => {
                for index in 0..width {
                    if a.get_ref(index)? != b.get_ref(index)? {
                        return Ok(false);
                    }
                }
            }
            _ => return Ok(false),
        }
    }
}

fn write_database(fresh: &Database, path: &Path) -> Result<(), ProjectError> {
    fresh.connection().backup(MAIN_DB, path, None)?;
    fs::OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|f| f.sync_all())
        .map_err(io_error)?;
    verify_database(path)
}

fn digest_file(path: &Path) -> Result<String, ProjectError> {
    let mut file = fs::File::open(path).map_err(io_error)?;
    let mut digest = Sha256::new();
    io::copy(&mut file, &mut digest).map_err(io_error)?;
    Ok(format!("{:x}", digest.finalize()))
}

fn unavailable_file(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        // Reparse points, offline data, recall-on-open and recall-on-access.
        metadata.file_attributes() & (0x400 | 0x1000 | 0x0004_0000 | 0x0040_0000) != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn pin_local_directory(path: &Path) -> Result<fs::File, ProjectError> {
    let handle = pin_candidate_directory(path)?;
    if unavailable_file(&handle.metadata().map_err(io_error)?) {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(handle)
}

pub(super) fn pinned_file(path: &Path, writable_source: bool) -> Result<fs::File, ProjectError> {
    let attributes = candidate_attributes(path)?;
    if !attributes.is_file() || unavailable_file(&attributes) {
        return Err(ProjectError::InvalidDatabase);
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        options
            .share_mode(if writable_source { 3 } else { 1 })
            .custom_flags(0x0020_0000);
    }
    #[cfg(not(windows))]
    let _ = writable_source;
    let file = options.open(path).map_err(io_error)?;
    let info = file.metadata().map_err(io_error)?;
    #[cfg(windows)]
    let links = winapi_util::file::information(&file)
        .map_err(io_error)?
        .number_of_links();
    #[cfg(unix)]
    let links = std::os::unix::fs::MetadataExt::nlink(&info);
    if !info.is_file() || unavailable_file(&info) || links != 1 {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(file)
}

pub(super) fn pin_database_files(dir: &Path) -> Result<Vec<fs::File>, ProjectError> {
    let mut pins = vec![pinned_file(&dir.join(DATABASE_FILE_NAME), true)?];
    for suffix in ["-wal", "-shm", "-journal"] {
        let sidecar = dir.join(format!("{DATABASE_FILE_NAME}{suffix}"));
        match fs::symlink_metadata(&sidecar) {
            Ok(_) => pins.push(pinned_file(&sidecar, true)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(io_error(e)),
        }
    }
    Ok(pins)
}

fn inspect_asset(
    root: &Path,
    stage: Option<&Path>,
    relative: &str,
    limit: u64,
) -> Result<(String, u64), ProjectError> {
    // Only regular application asset names and local directory components. No Windows streams or aliases.
    let parts: Vec<_> = relative.split('/').collect();
    if parts.len() < 2
        || parts[0] != "assets"
        || parts[1..].iter().any(|s| {
            let stem = s.split('.').next().unwrap_or_default().to_ascii_uppercase();
            let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || stem
                    .strip_prefix("COM")
                    .or_else(|| stem.strip_prefix("LPT"))
                    .is_some_and(|n| n.len() == 1 && n.as_bytes()[0].is_ascii_digit());
            device
                || s.is_empty()
                || s.starts_with('.')
                || s.ends_with('.')
                || s.chars()
                    .any(|c| c.is_control() || r#"<>:"\|?*"#.contains(c))
                || s.ends_with(' ')
        })
    {
        return Err(ProjectError::InvalidRelativePath);
    }
    let mut parent = root.to_path_buf();
    #[cfg(windows)]
    let mut pins = Vec::new();
    for part in &parts[..parts.len() - 1] {
        parent.push(part);
        #[cfg(windows)]
        pins.push(pin_local_directory(&parent)?);
        validate_directory_metadata(&candidate_attributes(&parent)?)?;
    }
    let file = root.join(relative);
    let input = pinned_file(&file, false)?;
    let info = input.metadata().map_err(io_error)?;
    if !info.is_file() || is_reparse_point(&info) || info.len() == 0 || info.len() > limit {
        return Err(ProjectError::InvalidDatabase);
    }
    let Some(stage) = stage else {
        let mut hash = Sha256::new();
        let read = io::copy(&mut input.take(limit + 1), &mut hash).map_err(io_error)?;
        if read != info.len() {
            return Err(ProjectError::InvalidDatabase);
        }
        return Ok((format!("{:x}", hash.finalize()), read));
    };
    let dest = stage.join(relative);
    fs::create_dir_all(dest.parent().ok_or(ProjectError::InvalidRelativePath)?)
        .map_err(io_error)?;
    let mut output = fs::File::create_new(&dest).map_err(io_error)?;
    let copied = io::copy(&mut input.take(limit + 1), &mut output).map_err(io_error)?;
    if copied != info.len() {
        return Err(ProjectError::InvalidDatabase);
    }
    output.sync_all().map_err(io_error)?;
    Ok((digest_file(&dest)?, copied))
}

pub(super) fn media_inventory(
    source: &Connection,
    dir: &Path,
    stage: Option<&Path>,
    approved: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, ProjectError> {
    let mut files = BTreeMap::new();
    let mut stmt = source
        .prepare("SELECT relative_path,thumbnail_path,checksum,byte_count FROM media_asset")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let (original, thumb, expected, size): (String, String, String, u32) =
            (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
        if original.starts_with("assets/thumbnails/") || !thumb.starts_with("assets/thumbnails/") {
            return Err(ProjectError::InvalidDatabase);
        }
        if approved.is_some_and(|files| {
            files.get(&original) != Some(&expected) || !files.contains_key(&thumb)
        }) {
            return Err(ProjectError::InvalidDatabase);
        }
        let (hash, len) = inspect_asset(dir, stage, &original, 25 * 1024 * 1024)?;
        if hash != expected || len != u64::from(size) {
            return Err(ProjectError::InvalidDatabase);
        }
        files.insert(original, hash);
        files.insert(
            thumb.clone(),
            inspect_asset(dir, stage, &thumb, 25 * 1024 * 1024)?.0,
        );
    }
    drop(rows);
    drop(stmt);
    Ok(files)
}

/// Öffentliche Metadaten einer vollständig veröffentlichten Arbeitskopie.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedHandoff {
    /// Relativer Ordnername innerhalb des lokalen Sicherungsordners.
    pub folder: String,
    /// Unabhängig vom Ordner ausgegebener SHA-256 über die Inventardatei.
    pub manifest_sha256: String,
    /// Separater Beleg der Szenenbasis aus derselben Transaktion.
    pub scene_basis_sha256: String,
}

impl ProjectManager {
    /// Bereitet die aktuellen Datensätze und referenzierten Bilder im lokalen Sicherungsordner vor.
    ///
    /// # Errors
    /// Eine abweichende Kennung, unbekanntes Schema oder unvollständige Medien sperren die Veröffentlichung.
    pub fn prepare_handoff(&self, id: ProjectId) -> Result<PreparedHandoff, ProjectError> {
        self.prepare_selected_handoff(id, None)
    }

    #[cfg(windows)]
    pub(super) fn with_inbox_manifest<T>(
        &self,
        folder: &str,
        inventory_sha256: &str,
        run: impl FnOnce(&Path, scene_merge::Manifest) -> Result<T, ProjectError>,
    ) -> Result<T, ProjectError> {
        scene_merge::validate_package_folder(folder)?;
        let inbox = self.root.join("empfang");
        let dir = inbox.join(folder);
        let _root = pin_local_directory(&self.root)?;
        let _inbox = pin_local_directory(&inbox)?;
        let _package = pin_local_directory(&dir)?;
        let bytes = scene_merge::confirmed_file(
            &dir.join("uebernahme.json"),
            inventory_sha256,
            128 * 1024,
        )?;
        let inventory: scene_merge::Manifest =
            serde_json::from_slice(&bytes).map_err(|_| ProjectError::InvalidDatabase)?;
        if inventory.format != "kraqel-logical-preparation-v2" {
            return Err(ProjectError::InvalidDatabase);
        }
        run(&dir, inventory)
    }

    /// Receives an explicitly selected v2 package from the local inbox.
    ///
    /// # Errors
    /// An occupied destination or a mismatching receipt prevents publication.
    #[cfg(windows)]
    pub fn receive_handoff(
        &self,
        folder: &str,
        inventory_sha256: &str,
    ) -> Result<(), ProjectError> {
        self.with_inbox_manifest(folder, inventory_sha256, |dir, mut inventory| {
            let id = inventory.project_id;
            let dest = self.root.join(id.to_string());
            match fs::symlink_metadata(&dest) {
                Ok(_) => return Err(ProjectError::ProjectAlreadyExists),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(io_error(e)),
            }
            scene_basis::Basis::read(dir, &inventory.scene_basis_sha256, id)?;
            let db_hash = inventory
                .files
                .remove(DATABASE_FILE_NAME)
                .ok_or(ProjectError::InvalidDatabase)?;
            let source = scene_merge::read_package_database(dir, &db_hash)?;
            let fresh = rebuild(&source, id)?;
            let mut stage = TemporaryDirectory::create(&self.root)?;
            let backups = stage.path().join(BACKUP_DIRECTORY_NAME);
            fs::create_dir(&backups).map_err(io_error)?;
            if scene_basis::stage_basis(fresh.connection(), id, &backups)?
                != inventory.scene_basis_sha256
                || media_inventory(
                    fresh.connection(),
                    dir,
                    Some(stage.path()),
                    Some(&inventory.files),
                )? != inventory.files
            {
                return Err(ProjectError::InvalidDatabase);
            }
            write_database(&fresh, &stage.path().join(DATABASE_FILE_NAME))?;
            // Windows refuses to rename a directory over another directory.
            fs::rename(stage.path(), dest).map_err(io_error)?;
            stage.keep();
            Ok(())
        })
    }

    pub(super) fn prepare_selected_handoff(
        &self,
        id: ProjectId,
        selection: Option<(&SceneMergeSelection, Option<[i64; 2]>)>,
    ) -> Result<PreparedHandoff, ProjectError> {
        #[cfg(windows)]
        let directory = self.root.join(id.to_string());
        #[cfg(windows)]
        let _project_pin = pin_candidate_directory(&directory)?;
        let dir = self
            .resolve_existing_directory(Path::new(&id.to_string()), ProjectError::ProjectMissing)?;
        let path =
            self.resolve_regular_child(&dir, DATABASE_FILE_NAME, ProjectError::DatabaseMissing)?;
        let pins = pin_database_files(&dir)?;
        let mut db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        db.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY")?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let backup = dir.join(BACKUP_DIRECTORY_NAME);
        #[cfg(windows)]
        let _backup_pin = pin_candidate_directory(&backup)?;
        validate_directory_metadata(&candidate_attributes(&backup)?)?;
        let mut stage = TemporaryDirectory::create(&backup)?;
        let main = stage.path().join(DATABASE_FILE_NAME);
        let mut fresh = rebuild(&tx, id)?;
        if let Some((selection, reviewed)) = selection {
            selection.apply(&mut fresh, id, &backup, reviewed)?;
            // Rematerialize logical rows. Edited SQLite and FTS pages can retain the old text.
            fresh = rebuild(fresh.connection(), id)?;
        }
        write_database(&fresh, &main)?;
        let mut files = media_inventory(&tx, &dir, Some(stage.path()), None)?;
        files.insert(DATABASE_FILE_NAME.to_owned(), digest_file(&main)?);
        let scene_basis_sha256 = scene_basis::stage_basis(fresh.connection(), id, stage.path())?;
        let receipt = serde_json::json!({"format":"kraqel-logical-preparation-v2","projectId":id,"files":files,"sceneBasisSha256":scene_basis_sha256});
        let mut manifest =
            fs::File::create_new(stage.path().join("uebernahme.json")).map_err(io_error)?;
        serde_json::to_writer(&mut manifest, &receipt)
            .map_err(|_| ProjectError::InvalidDatabase)?;
        manifest.sync_all().map_err(io_error)?;
        drop(manifest);
        let manifest_sha256 = digest_file(&stage.path().join("uebernahme.json"))?;
        tx.commit()?;
        drop(pins);
        let name = format!("uebernahme-{}", ProjectId::new());
        let candidate = PreparedHandoff {
            folder: name,
            manifest_sha256,
            scene_basis_sha256,
        };
        if let Some((selection, Some([local_revision, _]))) = selection {
            let digest = scene_merge::conflict_approval(id, selection, &candidate, local_revision)?;
            scene_activation::publish_receipt(
                &dir.join(format!(".fassungswahl-{digest}")),
                digest.as_bytes(),
            )?;
        }
        fs::rename(stage.path(), backup.join(&candidate.folder)).map_err(io_error)?;
        stage.keep();
        Ok(candidate)
    }
}
