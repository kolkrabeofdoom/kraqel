//! Explicit, non-deleting adoption of existing scenes in a new logical copy.
use super::scene_basis::{Basis, hash, is_hash};
use super::*;
use crate::document_repository::{MAX_EDITOR_DOCUMENT_BYTES, plain_text, validate_content};
use kraqel_domain::{EntityId, ProjectId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneMergeSelection {
    pub base: String,
    pub local: String,
    pub incoming: String,
    pub folder: String,
    pub manifest: String,
    pub scenes: Vec<EntityId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    format: String,
    project_id: ProjectId,
    scene_basis_sha256: String,
    files: BTreeMap<String, String>,
}

pub(super) fn confirmed_file(path: &Path, digest: &str, max: u64) -> Result<Vec<u8>, ProjectError> {
    if !is_hash(digest) {
        return Err(ProjectError::InvalidDatabase);
    }
    let file = handoff::pinned_file(path, false)?;
    let len = file
        .metadata()
        .map_err(|_| ProjectError::InvalidDatabase)?
        .len();
    if len == 0 || len > max {
        return Err(ProjectError::InvalidDatabase);
    }
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProjectError::InvalidDatabase)?;
    if bytes.len() as u64 != len || hash(&bytes) != digest {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(bytes)
}

pub(super) fn package_database(
    id: ProjectId,
    backup: &Path,
    folder: &str,
    manifest_hash: &str,
    basis_hash: &str,
) -> Result<Connection, ProjectError> {
    let raw = folder
        .strip_prefix("uebernahme-")
        .ok_or(ProjectError::InvalidRelativePath)?;
    if ProjectId::parse(raw).is_none_or(|v| v.to_string() != raw) {
        return Err(ProjectError::InvalidRelativePath);
    }
    let dir = backup.join(folder);
    #[cfg(windows)]
    let _pin = pin_candidate_directory(&dir)?;
    validate_directory_metadata(&candidate_attributes(&dir)?)?;
    let bytes = confirmed_file(&dir.join("uebernahme.json"), manifest_hash, 128 * 1024)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| ProjectError::InvalidDatabase)?;
    if manifest.format != "kraqel-logical-preparation-v2"
        || manifest.project_id != id
        || manifest.scene_basis_sha256 != basis_hash
    {
        return Err(ProjectError::InvalidDatabase);
    }
    // The separately approved scene receipt must also be present in this exact package.
    Basis::read(&dir, basis_hash, id)?;
    let digest = manifest
        .files
        .get(DATABASE_FILE_NAME)
        .ok_or(ProjectError::InvalidDatabase)?;
    let image = confirmed_file(&dir.join(DATABASE_FILE_NAME), digest, 128 * 1024 * 1024)?;
    if !image.starts_with(b"SQLite format 3\0") || image.get(18..20) != Some(&[1, 1]) {
        return Err(ProjectError::InvalidDatabase);
    }
    let mut db = Connection::open_in_memory()?;
    db.deserialize_read_exact(MAIN_DB, image.as_slice(), image.len(), true)?;
    db.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY; PRAGMA query_only=ON")?;
    Ok(db)
}

impl SceneMergeSelection {
    pub(super) fn apply(
        &self,
        target: &mut Database,
        id: ProjectId,
        backup: &Path,
    ) -> Result<(), ProjectError> {
        let selected: BTreeSet<_> = self.scenes.iter().copied().collect();
        if selected.is_empty() || selected.len() != self.scenes.len() {
            return Err(ProjectError::InvalidDatabase);
        }
        let mut local = Basis::from_db(target.connection(), id)?;
        if hash(&local.bytes()?) != self.local {
            return Err(ProjectError::InvalidDatabase);
        }
        let base = Basis::read(backup, &self.base, id)?;
        let incoming = Basis::read(backup, &self.incoming, id)?;
        let source = package_database(id, backup, &self.folder, &self.manifest, &self.incoming)?;
        if handoff::schema(&source)? != handoff::schema(target.connection())?
            || Basis::from_db(&source, id)?.bytes()? != incoming.bytes()?
        {
            return Err(ProjectError::InvalidDatabase);
        }
        let tx = target.transaction()?;
        for scene in selected {
            // All three must exist. Missing remote scenes never become DELETE operations.
            let old = base
                .scenes
                .get(&scene)
                .ok_or(ProjectError::InvalidDatabase)?;
            let next = incoming
                .scenes
                .get(&scene)
                .ok_or(ProjectError::InvalidDatabase)?;
            if local.scenes.get(&scene) != Some(old) || next == old {
                return Err(ProjectError::InvalidDatabase);
            }
            let (parent, position, title, content, text): (
                Option<String>,
                i64,
                String,
                String,
                String,
            ) = source.query_row(
                "SELECT parent_id,position,title,content_json,plain_text FROM document
                     WHERE project_id=?1 AND id=?2 AND kind='scene'",
                params![id.to_string(), scene.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?;
            if content.len() > MAX_EDITOR_DOCUMENT_BYTES {
                return Err(ProjectError::InvalidDatabase);
            }
            let document =
                serde_json::from_str(&content).map_err(|_| ProjectError::InvalidDatabase)?;
            validate_content(&document).map_err(|_| ProjectError::InvalidDatabase)?;
            if plain_text(&document) != text {
                return Err(ProjectError::InvalidDatabase);
            }
            // Moving a scene would require reviewing sibling order as well.
            let changed = tx.execute(
                "UPDATE document SET title=?1,content_json=?2,plain_text=?3,revision=revision+1,
                 updated_at='unix:' || unixepoch()
                 WHERE project_id=?4 AND id=?5 AND kind='scene' AND parent_id IS ?6 AND position=?7",
                params![title,content,text,id.to_string(),scene.to_string(),parent,position],
            )?;
            if changed != 1 {
                return Err(ProjectError::InvalidDatabase);
            }
            local.scenes.insert(scene, next.clone());
        }
        // Verify the entire result, including every unselected scene, before publication.
        if Basis::from_db(&tx, id)?.scenes != local.scenes {
            return Err(ProjectError::InvalidDatabase);
        }
        tx.commit()?;
        Ok(())
    }
}

impl ProjectManager {
    /// Baut eine neue Arbeitskopie mit ausdrücklich ausgewählten einseitigen Szenenänderungen.
    ///
    /// # Errors
    /// Unbestätigte Quellen, fehlende Szenen, Strukturänderungen und Konflikte sperren die gesamte Auswahl.
    pub fn prepare_scene_merge(
        &self,
        id: ProjectId,
        selection: &SceneMergeSelection,
    ) -> Result<PreparedHandoff, ProjectError> {
        self.prepare_selected_handoff(id, Some(selection))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project};

    #[test]
    fn adopts_only_selected_scene_into_new_copy_and_refuses_remote_deletion() {
        if let Ok(root) = std::env::var("KRAQEL_TEST_ACTIVATION_HOME") {
            let manager = ProjectManager::new(root).unwrap();
            let id = ProjectId::parse(&std::env::var("KRAQEL_TEST_ACTIVATION_PROJECT").unwrap())
                .unwrap();
            manager
                .switch_scene_activation(
                    id,
                    &std::env::var("KRAQEL_TEST_ACTIVATION_TICKET").unwrap(),
                    false,
                )
                .unwrap();
            std::process::exit(82); // Committed, response never delivered.
        }
        let local_home = tempfile::tempdir().unwrap();
        let other_home = tempfile::tempdir().unwrap();
        let local = ProjectManager::new(local_home.path()).unwrap();
        let other = ProjectManager::new(other_home.path()).unwrap();
        let model = Project {
            id: ProjectId::new(),
            name: "Schreibstudio".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let id = model.id;
        let selected = EntityId::new();
        let retained = EntityId::new();
        let here = local.create_project(id.to_string(), &model).unwrap();
        let there = other.create_project(id.to_string(), &model).unwrap();
        let editor_json = |text: &str| {
            serde_json::json!({"version":1,"document":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":text,"marks":[{"type":"bold"}]}]}]}}).to_string()
        };
        for project in [&here, &there] {
            for (position, scene, text) in [
                (0, selected, "Früherer Entwurf des Hafens"),
                (1, retained, "Die Uhr am Turm"),
            ] {
                project.database().connection().execute(
                    "INSERT INTO document(id,project_id,kind,title,position,plain_text,content_json,created_at,updated_at)
                    VALUES(?1,?2,'scene',?3,?4,?3,?5,'t','t')",params![scene.to_string(),id.to_string(),text,position,editor_json(text)]).unwrap();
            }
        }
        let live = here.database().connection();
        live.execute(
            "INSERT INTO plot_card(project_id,document_id,position,updated_at) VALUES(?1,?2,0,'t')",
            params![id.to_string(), retained.to_string()],
        )
        .unwrap();
        let base = local.save_scene_basis(id).unwrap().sha256;
        let remote = there.database().connection();
        remote.execute("UPDATE document SET title='Am Hafen',plain_text='Das Boot legt an.',content_json=?1,revision=4 WHERE id=?2",
            params![editor_json("Das Boot legt an."),selected.to_string()]).unwrap();
        remote
            .execute("DELETE FROM document WHERE id=?1", [retained.to_string()])
            .unwrap();
        let source = other.prepare_handoff(id).unwrap();
        let src = there.directory().join("backups").join(&source.folder);
        let backups = here.directory().join("backups");
        let pkg = backups.join(&source.folder);
        fs::create_dir(&pkg).unwrap();
        let basis_file = format!("szenenbasis-{}.json", source.scene_basis_sha256);
        fs::copy(src.join(&basis_file), backups.join(&basis_file)).unwrap();
        for file in ["uebernahme.json", DATABASE_FILE_NAME, &basis_file] {
            fs::copy(src.join(file), pkg.join(file)).unwrap();
        }
        let mut plan = SceneMergeSelection {
            base: base.clone(),
            local: base.clone(),
            incoming: source.scene_basis_sha256,
            folder: source.folder,
            manifest: source.manifest_sha256,
            scenes: vec![selected, retained],
        };
        let count = || fs::read_dir(&backups).unwrap().count();
        let initial = count();
        assert!(local.prepare_scene_merge(id, &plan).is_err());
        assert_eq!(count(), initial);
        plan.scenes = vec![selected];
        let receipt = local.prepare_scene_merge(id, &plan).unwrap();
        let result = backups.join(&receipt.folder).join(DATABASE_FILE_NAME);
        let copy = open_read_only(&result).unwrap();
        let read = |db: &Connection| {
            db.query_row(
                "SELECT title,plain_text,revision,content_json FROM document WHERE id=?1",
                [selected.to_string()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, u32>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            )
            .unwrap()
        };
        let (title, text, rev, rich) = read(&copy);
        assert_eq!(
            (title.as_str(), text.as_str(), rev, rich),
            (
                "Am Hafen",
                "Das Boot legt an.",
                1,
                editor_json("Das Boot legt an.")
            )
        );
        assert_eq!(
            copy.query_row(
                "SELECT count(*) FROM plot_card WHERE document_id=?1",
                [retained.to_string()],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(read(live).2, 0);
        assert_eq!(read(remote).2, 4);
        let old = "Früherer Entwurf des Hafens".as_bytes();
        assert!(
            !fs::read(&result)
                .unwrap()
                .windows(old.len())
                .any(|s| s == old)
        );
        let ticket = local.secure_scene_activation(id, &plan, &receipt).unwrap();
        // Even with the correct source revision, backup and digest, local approval is required.
        let marker = here.directory().join(format!(".szenenfreigabe-{ticket}"));
        let parked = local_home.path().join("parked-approval");
        fs::rename(&marker, &parked).unwrap();
        assert!(local.switch_scene_activation(id, &ticket, false).is_err());
        fs::rename(parked, marker).unwrap();
        // Even an edit which only advances the revision invalidates the old activation.
        live.execute(
            "UPDATE document SET revision=revision+1 WHERE id=?1",
            [selected.to_string()],
        )
        .unwrap();
        assert!(local.switch_scene_activation(id, &ticket, false).is_err());
        live.execute(
            "UPDATE document SET revision=0 WHERE id=?1",
            [selected.to_string()],
        )
        .unwrap();
        // New manager instance: no process-local authority or remembered state is needed.
        let restarted = ProjectManager::new(local_home.path()).unwrap();
        assert!(
            restarted
                .switch_scene_activation(id, &ticket, true)
                .is_err()
        );
        for (point, code, revision) in [("before-commit", 81, 0), ("after-commit", 82, 1)] {
            let status=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","project_manager::scene_merge::tests::adopts_only_selected_scene_into_new_copy_and_refuses_remote_deletion"])
                .env("KRAQEL_TEST_ACTIVATION_HOME",local_home.path())
                .env("KRAQEL_TEST_ACTIVATION_PROJECT",id.to_string())
                .env("KRAQEL_TEST_ACTIVATION_TICKET",&ticket)
                .env("KRAQEL_TEST_ACTIVATION_CRASH",point).status().unwrap();
            assert_eq!(status.code(), Some(code));
            assert_eq!(read(live).2, revision);
        }
        restarted
            .switch_scene_activation(id, &ticket, false)
            .unwrap();
        restarted
            .switch_scene_activation(id, &ticket, false)
            .unwrap();
        assert_eq!(read(live), read(&copy));
        live.execute(
            "UPDATE plot_card SET status='revised' WHERE document_id=?1",
            [retained.to_string()],
        )
        .unwrap();
        restarted
            .switch_scene_activation(id, &ticket, true)
            .unwrap();
        restarted
            .switch_scene_activation(id, &ticket, true)
            .unwrap();
        assert_eq!(read(live).1, "Früherer Entwurf des Hafens");
        assert_eq!(read(live).2, 2);
        assert!(
            restarted
                .switch_scene_activation(id, &ticket, false)
                .is_err()
        );
        assert_eq!(
            live.query_row(
                "SELECT status FROM plot_card WHERE document_id=?1",
                [retained.to_string()],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "revised"
        );
        live.execute(
            "UPDATE document SET title='Späterer lokaler Entwurf' WHERE id=?1",
            [selected.to_string()],
        )
        .unwrap();
        assert!(local.prepare_scene_merge(id, &plan).is_err());
        plan.local = local.save_scene_basis(id).unwrap().sha256;
        let before = count();
        assert!(local.prepare_scene_merge(id, &plan).is_err());
        assert_eq!(count(), before);
    }
}
