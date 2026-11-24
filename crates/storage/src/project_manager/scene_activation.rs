//! Activation and undo change only the reviewed scene fields in a single SQLite commit.
use super::scene_basis::{Basis, hash, is_hash};
use super::scene_merge::{confirmed_file, package_database};
use super::*;
use crate::document_repository::{MAX_EDITOR_DOCUMENT_BYTES, plain_text, validate_content};
use kraqel_domain::{EntityId, ProjectId};
use rusqlite::{Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FORMAT: &str = "kraqel-scene-activation-v1";
const MAX_RECORD: u64 = 128 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    title: String,
    content: String,
    text: String,
    revision: i64,
    parent: Option<String>,
    position: i64,
}

impl Scene {
    fn load(db: &Connection, id: ProjectId, scene: EntityId) -> Result<Self, ProjectError> {
        Ok(db.query_row(
            "SELECT title,content_json,plain_text,revision,parent_id,position FROM document
            WHERE project_id=?1 AND id=?2 AND kind='scene'",
            params![id.to_string(), scene.to_string()],
            |r| {
                Ok(Self {
                    title: r.get(0)?,
                    content: r.get(1)?,
                    text: r.get(2)?,
                    revision: r.get(3)?,
                    parent: r.get(4)?,
                    position: r.get(5)?,
                })
            },
        )?)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if self.content.len() > MAX_EDITOR_DOCUMENT_BYTES
            || self.revision < 0
            || self.revision > i64::MAX - 2
        {
            return Err(ProjectError::InvalidDatabase);
        }
        let doc = serde_json::from_str(&self.content).map_err(|_| ProjectError::InvalidDatabase)?;
        validate_content(&doc).map_err(|_| ProjectError::InvalidDatabase)?;
        if plain_text(&doc) != self.text {
            return Err(ProjectError::InvalidDatabase);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Activation {
    format: String,
    project: ProjectId,
    before: String,
    candidate: PreparedHandoff,
    previous: BTreeMap<EntityId, Scene>,
}

impl ProjectManager {
    fn with_activation_transaction<T>(
        &self,
        id: ProjectId,
        run: impl FnOnce(&Transaction<'_>, &Path) -> Result<(T, Option<fs::File>), ProjectError>,
    ) -> Result<T, ProjectError> {
        #[cfg(windows)]
        let _project = pin_candidate_directory(&self.root.join(id.to_string()))?;
        let dir = self
            .resolve_existing_directory(Path::new(&id.to_string()), ProjectError::ProjectMissing)?;
        let backup = dir.join(BACKUP_DIRECTORY_NAME);
        #[cfg(windows)]
        let _backup = pin_candidate_directory(&backup)?;
        validate_directory_metadata(&candidate_attributes(&backup)?)?;
        let _pins = handoff::pin_database_files(&dir)?;
        let mut db = Connection::open_with_flags(
            dir.join(DATABASE_FILE_NAME),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA synchronous=FULL")?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if handoff::schema(&tx)? != handoff::schema(Database::open_in_memory()?.connection())? {
            return Err(ProjectError::InvalidDatabase);
        }
        let (result, pin) = run(&tx, &backup)?;
        tx.commit()?;
        drop(pin);
        Ok(result)
    }

    /// Sichert die bisherigen ausgewählten Szenen vor der ausdrücklichen Aktivierung.
    ///
    /// # Errors
    /// Abweichende Fassungen, Revisionen oder Strukturen verhindern die Freigabe.
    pub fn secure_scene_activation(
        &self,
        id: ProjectId,
        selection: &SceneMergeSelection,
        candidate: &PreparedHandoff,
    ) -> Result<String, ProjectError> {
        self.with_activation_transaction(id, |tx, dir| {
            let mut current = Basis::from_db(tx, id)?;
            let selected: BTreeSet<_> = selection.scenes.iter().copied().collect();
            if selected.is_empty()
                || selected.len() != selection.scenes.len()
                || hash(&current.bytes()?) != selection.local
            {
                return Err(ProjectError::InvalidDatabase);
            }
            let base = Basis::read(dir, &selection.base, id)?;
            let incoming = Basis::read(dir, &selection.incoming, id)?;
            let result = package_database(
                id,
                dir,
                &candidate.folder,
                &candidate.manifest_sha256,
                &candidate.scene_basis_sha256,
            )?;
            if handoff::schema(&result)? != handoff::schema(tx)? {
                return Err(ProjectError::InvalidDatabase);
            }
            let mut previous = BTreeMap::new();
            for scene in selected {
                let prior = base
                    .scenes
                    .get(&scene)
                    .ok_or(ProjectError::InvalidDatabase)?;
                let next = incoming
                    .scenes
                    .get(&scene)
                    .ok_or(ProjectError::InvalidDatabase)?;
                if current.scenes.get(&scene) != Some(prior) || next == prior {
                    return Err(ProjectError::InvalidDatabase);
                }
                let old = Scene::load(tx, id, scene)?;
                let new = Scene::load(&result, id, scene)?;
                old.validate()?;
                new.validate()?;
                if old.parent != new.parent
                    || old.position != new.position
                    || new.revision != old.revision + 1
                {
                    return Err(ProjectError::InvalidDatabase);
                }
                previous.insert(scene, old);
                current.scenes.insert(scene, next.clone());
            }
            if hash(&current.bytes()?) != candidate.scene_basis_sha256
                || current.bytes()? != Basis::from_db(&result, id)?.bytes()?
            {
                return Err(ProjectError::InvalidDatabase);
            }
            let record = Activation {
                format: FORMAT.into(),
                project: id,
                before: selection.local.clone(),
                candidate: candidate.clone(),
                previous,
            };
            let bytes = serde_json::to_vec(&record).map_err(|_| ProjectError::InvalidDatabase)?;
            if bytes.len() as u64 > MAX_RECORD {
                return Err(ProjectError::InvalidDatabase);
            }
            let digest = hash(&bytes);
            let path = dir.join(format!("szenenaktivierung-{digest}.json"));
            publish_receipt(&path, &bytes)?;
            // Only a locally issued approval in the project root authorizes a switch.
            // Files supplied in the portable backups directory cannot register themselves.
            let marker = self
                .root
                .join(id.to_string())
                .join(format!(".szenenfreigabe-{digest}"));
            publish_receipt(&marker, digest.as_bytes())?;
            Ok((digest, None))
        })
    }

    /// Aktiviert die bestätigte Auswahl oder nimmt dieselben Änderungen zurück.
    ///
    /// # Errors
    /// Ein fehlender Beleg oder ein abweichender Szenenstand sperrt jede Änderung.
    pub fn switch_scene_activation(
        &self,
        id: ProjectId,
        ticket: &str,
        undo: bool,
    ) -> Result<(), ProjectError> {
        if !is_hash(ticket) {
            return Err(ProjectError::InvalidRelativePath);
        }
        self.with_activation_transaction(id, |tx, dir| {
            let marker = self
                .root
                .join(id.to_string())
                .join(format!(".szenenfreigabe-{ticket}"));
            confirmed_file(&marker, &hash(ticket.as_bytes()), 64)?;
            let path = dir.join(format!("szenenaktivierung-{ticket}.json"));
            // Deny replacement or removal of the recovery file through COMMIT on Windows.
            let pin = handoff::pinned_file(&path, false)?;
            let bytes = confirmed_file(&path, ticket, MAX_RECORD)?;
            let record: Activation =
                serde_json::from_slice(&bytes).map_err(|_| ProjectError::InvalidDatabase)?;
            record.switch(tx, dir, id, undo)?;
            Ok(((), Some(pin)))
        })
    }
}

impl Activation {
    fn switch(
        self,
        tx: &Transaction<'_>,
        dir: &Path,
        id: ProjectId,
        undo: bool,
    ) -> Result<(), ProjectError> {
        if self.format != FORMAT
            || self.project != id
            || self.previous.is_empty()
            || self.before == self.candidate.scene_basis_sha256
        {
            return Err(ProjectError::InvalidDatabase);
        }
        let live = hash(&Basis::from_db(tx, id)?.bytes()?);
        let after = &self.candidate.scene_basis_sha256;
        let (from, to) = if undo {
            (after, &self.before)
        } else {
            (&self.before, after)
        };
        if &live != from && &live != to {
            return Err(ProjectError::InvalidDatabase);
        }
        let already = &live == to;
        // Initial, activated and undone generations have offsets 0, 1 and 2.
        let expected_offset = i64::from(undo) + i64::from(already);
        let candidate = if undo || already {
            None
        } else {
            let db = package_database(
                id,
                dir,
                &self.candidate.folder,
                &self.candidate.manifest_sha256,
                after,
            )?;
            if handoff::schema(&db)? != handoff::schema(tx)?
                || hash(&Basis::from_db(&db, id)?.bytes()?) != *after
            {
                return Err(ProjectError::InvalidDatabase);
            }
            Some(db)
        };
        for (scene, old) in self.previous {
            old.validate()?;
            let current = Scene::load(tx, id, scene)?;
            if current.revision != old.revision + expected_offset
                || current.parent != old.parent
                || current.position != old.position
            {
                return Err(ProjectError::InvalidDatabase);
            }
            if already {
                continue;
            }
            let next = if let Some(ref db) = candidate {
                Scene::load(db, id, scene)?
            } else {
                old
            };
            next.validate()?;
            let changed = tx.execute(
                "UPDATE document SET title=?1,content_json=?2,plain_text=?3,revision=revision+1,
                 updated_at='unix:' || unixepoch() WHERE project_id=?4 AND id=?5 AND kind='scene' AND revision=?6",
                params![next.title,next.content,next.text,id.to_string(),scene.to_string(),current.revision],
            )?;
            if changed != 1 {
                return Err(ProjectError::InvalidDatabase);
            }
        }
        if hash(&Basis::from_db(tx, id)?.bytes()?) != *to {
            return Err(ProjectError::InvalidDatabase);
        }
        #[cfg(test)]
        if std::env::var("KRAQEL_TEST_ACTIVATION_CRASH").as_deref() == Ok("before-commit") {
            std::process::exit(81);
        }
        Ok(())
    }
}

fn publish_receipt(path: &Path, bytes: &[u8]) -> Result<(), ProjectError> {
    let parent = path.parent().ok_or(ProjectError::InvalidRelativePath)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(storage_io)?;
    temp.write_all(bytes).map_err(storage_io)?;
    temp.as_file().sync_all().map_err(storage_io)?;
    match temp.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(e) if e.error.kind() == io::ErrorKind::AlreadyExists => {
            confirmed_file(path, &hash(bytes), bytes.len() as u64)?;
            Ok(())
        }
        Err(e) => Err(storage_io(e.error)),
    }
}

fn storage_io(source: io::Error) -> ProjectError {
    ProjectError::Io {
        operation: "Aktivierungsbeleg sichern",
        source,
    }
}
