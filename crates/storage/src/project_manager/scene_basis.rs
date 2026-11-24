//! Hash-only receipts for explicit offline scene comparisons.
use super::*;
use kraqel_domain::{EntityId, ProjectId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};

const FORMAT: &str = "kraqel-scene-basis-v1";
const MAX_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Basis {
    format: String,
    project_id: ProjectId,
    pub(super) scenes: BTreeMap<EntityId, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBasisReceipt {
    pub sha256: String,
    pub scene_count: usize,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
enum Difference {
    Local,
    Incoming,
    BothEqual,
    Conflict,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneChange {
    scene_id: EntityId,
    difference: Difference,
    base_exists: bool,
    local_exists: bool,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    deletion_effects: BTreeMap<String, u32>,
    incoming_exists: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBasisComparison {
    local_sha256: String,
    changes: Vec<SceneChange>,
    unchanged: usize,
}

fn io_error(source: io::Error) -> ProjectError {
    ProjectError::Io {
        operation: "Szenenbasis prüfen",
        source,
    }
}

pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn is_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn filename(digest: &str) -> String {
    format!("szenenbasis-{digest}.json")
}

impl Basis {
    pub(super) fn from_db(db: &Connection, id: ProjectId) -> Result<Self, ProjectError> {
        if db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))?
            != CURRENT_DATABASE_VERSION.0
            || db
                .prepare("SELECT id FROM project LIMIT 2")?
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?
                != [id.to_string()]
        {
            return Err(ProjectError::InvalidDatabase);
        }
        let mut stmt = db.prepare("SELECT id,parent_id,position,title,content_json,plain_text FROM document WHERE project_id=?1 AND kind='scene'")?;
        let mut rows = stmt.query([id.to_string()])?;
        let mut scenes = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            let scene = EntityId::parse(&raw)
                .filter(|v| v.to_string() == raw)
                .ok_or(ProjectError::InvalidDatabase)?;
            let content: String = row.get(4)?;
            let mut rich: serde_json::Value =
                serde_json::from_str(&content).map_err(|_| ProjectError::InvalidDatabase)?;
            rich.sort_all_objects();
            let fields = (
                FORMAT,
                id,
                scene,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                rich,
                row.get::<_, String>(5)?,
            );
            let bytes = serde_json::to_vec(&fields).map_err(|_| ProjectError::InvalidDatabase)?;
            scenes.insert(scene, hash(&bytes));
        }
        Ok(Self {
            format: FORMAT.to_owned(),
            project_id: id,
            scenes,
        })
    }

    pub(super) fn bytes(&self) -> Result<Vec<u8>, ProjectError> {
        let bytes = serde_json::to_vec(self).map_err(|_| ProjectError::InvalidDatabase)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(ProjectError::InvalidDatabase);
        }
        Ok(bytes)
    }

    pub(super) fn read(dir: &Path, digest: &str, id: ProjectId) -> Result<Self, ProjectError> {
        if !is_hash(digest) {
            return Err(ProjectError::InvalidRelativePath);
        }
        let path = dir.join(filename(digest));
        let file = handoff::pinned_file(&path, false)?;
        let mut raw = Vec::new();
        file.take(MAX_BYTES + 1)
            .read_to_end(&mut raw)
            .map_err(io_error)?;
        if raw.len() as u64 > MAX_BYTES || hash(&raw) != digest {
            return Err(ProjectError::InvalidDatabase);
        }
        let basis: Self =
            serde_json::from_slice(&raw).map_err(|_| ProjectError::InvalidDatabase)?;
        // Enforce the producer encoding, including unique, canonical scene keys.
        if basis.format != FORMAT
            || basis.project_id != id
            || !basis.scenes.values().all(|v| is_hash(v))
            || basis.bytes()? != raw
        {
            return Err(ProjectError::InvalidDatabase);
        }
        Ok(basis)
    }
}

/// Writes the basis alongside a newly staged logical handoff, from its existing transaction.
pub(super) fn stage_basis(
    db: &Connection,
    id: ProjectId,
    stage: &Path,
) -> Result<String, ProjectError> {
    let bytes = Basis::from_db(db, id)?.bytes()?;
    let digest = hash(&bytes);
    let mut file = fs::File::create_new(stage.join(filename(&digest))).map_err(io_error)?;
    file.write_all(&bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    Ok(digest)
}

impl ProjectManager {
    pub(super) fn with_scene_basis<T>(
        &self,
        id: ProjectId,
        run: impl FnOnce(Basis, &Path, &Connection) -> Result<T, ProjectError>,
    ) -> Result<T, ProjectError> {
        #[cfg(windows)]
        let _project = pin_candidate_directory(&self.root.join(id.to_string()))?;
        let dir = self
            .resolve_existing_directory(Path::new(&id.to_string()), ProjectError::ProjectMissing)?;
        let backup = dir.join(BACKUP_DIRECTORY_NAME);
        validate_directory_metadata(&candidate_attributes(&backup)?)?;
        #[cfg(windows)]
        let _backup = pin_candidate_directory(&backup)?;
        let _files = handoff::pin_database_files(&dir)?;
        let mut db = open_read_only(&dir.join(DATABASE_FILE_NAME))?;
        db.execute_batch(
            "PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA query_only=ON",
        )?;
        let tx = db.transaction()?;
        let basis = Basis::from_db(&tx, id)?;
        let result = run(basis, &backup, &tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Saves a hash-only scene list without retaining manuscript history.
    ///
    /// # Errors
    /// Rejects an unverifiable project or an occupied path with different bytes.
    pub fn save_scene_basis(&self, id: ProjectId) -> Result<SceneBasisReceipt, ProjectError> {
        self.with_scene_basis(id, |basis, dir, _| {
            let bytes = basis.bytes()?;
            let sha256 = hash(&bytes);
            let path = dir.join(filename(&sha256));
            let mut file = tempfile::NamedTempFile::new_in(dir).map_err(io_error)?;
            file.write_all(&bytes).map_err(io_error)?;
            file.as_file().sync_all().map_err(io_error)?;
            match file.persist_noclobber(path) {
                Ok(_) => (),
                Err(e) if e.error.kind() == io::ErrorKind::AlreadyExists => {
                    Basis::read(dir, &sha256, id)?;
                }
                Err(e) => return Err(io_error(e.error)),
            }
            Ok(SceneBasisReceipt {
                sha256,
                scene_count: basis.scenes.len(),
            })
        })
    }

    /// Compares two explicitly identified receipts to the current committed local state.
    ///
    /// # Errors
    /// Rejects missing, damaged or foreign receipts before reporting changes.
    pub fn compare_scene_bases(
        &self,
        id: ProjectId,
        base: &str,
        incoming: &str,
    ) -> Result<SceneBasisComparison, ProjectError> {
        self.with_scene_basis(id, |local, dir, db| {
            let base = Basis::read(dir, base, id)?;
            let incoming = Basis::read(dir, incoming, id)?;
            let ids: BTreeSet<_> = base
                .scenes
                .keys()
                .chain(local.scenes.keys())
                .chain(incoming.scenes.keys())
                .copied()
                .collect();
            let mut changes = Vec::new();
            let mut unchanged = 0;
            for scene_id in ids {
                let (a, b, c) = (
                    base.scenes.get(&scene_id),
                    local.scenes.get(&scene_id),
                    incoming.scenes.get(&scene_id),
                );
                let difference = if a == b && b == c {
                    unchanged += 1;
                    continue;
                } else if b == c {
                    Difference::BothEqual
                } else if b == a {
                    Difference::Incoming
                } else if c == a {
                    Difference::Local
                } else {
                    Difference::Conflict
                };
                changes.push(SceneChange {
                    scene_id,
                    difference,
                    base_exists: a.is_some(),
                    deletion_effects: BTreeMap::new(),
                    local_exists: b.is_some(),
                    incoming_exists: c.is_some(),
                });
            }
            let mut removals: BTreeMap<_, _> = changes
                .iter_mut()
                .filter(|c| c.base_exists && c.local_exists && !c.incoming_exists)
                .map(|c| (c.scene_id, &mut c.deletion_effects))
                .collect();
            if !removals.is_empty() {
                // Count direct user-visible dependants in the same SQLite snapshot as the scene diff.
                let mut stmt = db.prepare("SELECT scene,kind,count(*) FROM (
                    SELECT document_id AS scene,'plotCards' AS kind FROM plot_card WHERE project_id=?1
                    UNION ALL SELECT document_id,'storyLinks' FROM link WHERE project_id=?1
                    UNION ALL SELECT document_id,'timelineLinks' FROM event_document_link WHERE project_id=?1
                    UNION ALL SELECT target_id,'mediaLinks' FROM asset_link WHERE project_id=?1 AND target_type='document'
                    UNION ALL SELECT source_document_id,'promptDrafts' FROM image_prompt_draft WHERE project_id=?1
                ) GROUP BY scene,kind")?;
                let mut rows = stmt.query([id.to_string()])?;
                while let Some(row) = rows.next()? {
                    let scene: String = row.get(0)?;
                    let key = EntityId::parse(&scene).ok_or(ProjectError::InvalidDatabase)?;
                    if let Some(effects) = removals.get_mut(&key) {
                        effects.insert(row.get(1)?, row.get(2)?);
                    }
                }
            }
            Ok(SceneBasisComparison {
                local_sha256: hash(&local.bytes()?),
                changes,
                unchanged,
            })
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project};

    #[test]
    fn compares_additions_deletions_and_conflicts_without_retaining_scene_text() {
        let tmp = tempfile::tempdir().unwrap();
        let manager = ProjectManager::new(tmp.path()).unwrap();
        let id = ProjectId::new();
        let project = manager
            .create_project(
                id.to_string(),
                &Project {
                    id,
                    name: "Test".into(),
                    schema_version: CURRENT_SCHEMA_VERSION,
                },
            )
            .unwrap();
        let db = project.database().connection();
        let scene = EntityId::new().to_string();
        let insert = |title: &str| {
            db.execute("INSERT INTO document(id, project_id, kind, title, plain_text, position, created_at, updated_at) VALUES(?1,?2,'scene',?3,'Privater Absatz',0,'t','t')",
                params![scene,id.to_string(),title]).unwrap();
        };
        let empty = manager.save_scene_basis(id).unwrap().sha256;
        insert("Ausgangstitel");
        let initial = manager.save_scene_basis(id).unwrap().sha256;
        db.execute(
            "UPDATE document SET revision=revision+1,updated_at='later'",
            [],
        )
        .unwrap();
        assert_eq!(manager.save_scene_basis(id).unwrap().sha256, initial);
        let added = manager.compare_scene_bases(id, &empty, &empty).unwrap();
        assert_eq!(added.changes[0].difference, Difference::Local);
        let backup = project.directory().join(BACKUP_DIRECTORY_NAME);
        let raw = fs::read_to_string(backup.join(filename(&initial))).unwrap();
        assert!(!raw.contains("Ausgangstitel") && !raw.contains("Privater Absatz"));
        db.execute(
            "UPDATE document SET content_json=?1",
            [r#"{"marks":["bold"]}"#],
        )
        .unwrap();
        let incoming = manager.save_scene_basis(id).unwrap().sha256;
        let equal = manager
            .compare_scene_bases(id, &initial, &incoming)
            .unwrap();
        assert_eq!(equal.changes[0].difference, Difference::BothEqual);
        db.execute("DELETE FROM document", []).unwrap();
        let result = manager
            .compare_scene_bases(id, &initial, &incoming)
            .unwrap();
        assert_eq!(result.changes[0].difference, Difference::Conflict);
        assert!(!result.changes[0].local_exists && result.changes[0].incoming_exists);
        assert_eq!(result.local_sha256, empty);
        assert_eq!(
            manager
                .compare_scene_bases(id, &empty, &incoming)
                .unwrap()
                .changes[0]
                .difference,
            Difference::Incoming
        );
        insert("Weitere Bearbeitung");
        db.execute(
            "INSERT INTO plot_card(project_id,document_id,position,updated_at) VALUES(?1,?2,0,'t')",
            params![id.to_string(), scene],
        )
        .unwrap();
        let deletion = manager.compare_scene_bases(id, &initial, &empty).unwrap();
        assert_eq!(
            deletion.changes[0].deletion_effects.get("plotCards"),
            Some(&1)
        );
        fs::write(backup.join(filename(&incoming)), raw).unwrap();
        assert!(
            manager
                .compare_scene_bases(id, &initial, &incoming)
                .is_err()
        );
        assert!(
            manager
                .compare_scene_bases(id, "../outside", &empty)
                .is_err()
        );
    }
}
