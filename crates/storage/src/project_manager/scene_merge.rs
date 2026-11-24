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
pub(super) struct Manifest {
    pub(super) format: String,
    pub(super) project_id: ProjectId,
    pub(super) scene_basis_sha256: String,
    pub(super) files: BTreeMap<String, String>,
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

pub(super) fn validate_package_folder(folder: &str) -> Result<(), ProjectError> {
    let raw = folder
        .strip_prefix("uebernahme-")
        .ok_or(ProjectError::InvalidRelativePath)?;
    if ProjectId::parse(raw).is_none_or(|v| v.to_string() != raw) {
        return Err(ProjectError::InvalidRelativePath);
    }
    Ok(())
}

pub(super) fn package_database(
    id: ProjectId,
    backup: &Path,
    folder: &str,
    manifest_hash: &str,
    basis_hash: &str,
) -> Result<Connection, ProjectError> {
    validate_package_folder(folder)?;
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
    read_package_database(&dir, digest)
}

pub(super) fn read_package_database(dir: &Path, digest: &str) -> Result<Connection, ProjectError> {
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
        reviewed: Option<[i64; 2]>,
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
            let here = local
                .scenes
                .get(&scene)
                .ok_or(ProjectError::InvalidDatabase)?;
            if let Some(expected) = reviewed {
                if self.scenes.len() != 1
                    || here == old
                    || here == next
                    || [
                        review_revision(&tx, id, scene)?,
                        review_revision(&source, id, scene)?,
                    ] != expected
                {
                    return Err(ProjectError::InvalidDatabase);
                }
            } else if here != old {
                return Err(ProjectError::InvalidDatabase);
            }
            if next == old {
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

/// A native-only view. No serialization or debugging implementation may expose its contents.
pub struct ReviewedScene {
    pub title: String,
    pub text: String,
    pub revision: i64,
}

pub enum ReviewChoice {
    Cancel,
    Local,
    Incoming,
}

#[derive(Serialize)]
#[serde(tag = "choice", rename_all = "camelCase")]
pub enum ReviewOutcome {
    Cancel,
    Local,
    Incoming { candidate: PreparedHandoff },
}

pub(super) fn conflict_approval(
    id: ProjectId,
    selection: &SceneMergeSelection,
    candidate: &PreparedHandoff,
    local_revision: i64,
) -> Result<String, ProjectError> {
    Ok(hash(
        &serde_json::to_vec(&(
            "kraqel-native-conflict-v1",
            id,
            selection,
            candidate,
            local_revision,
        ))
        .map_err(|_| ProjectError::InvalidDatabase)?,
    ))
}

fn review_revision(
    db: &Connection,
    project: ProjectId,
    scene: EntityId,
) -> Result<i64, ProjectError> {
    Ok(db.query_row(
        "SELECT revision FROM document WHERE project_id=?1 AND id=?2 AND kind='scene'",
        params![project.to_string(), scene.to_string()],
        |r| r.get(0),
    )?)
}

// The search index collapses whitespace. Render inline text nodes consecutively and preserve
// paragraph boundaries for the read-only comparison. Keep unfamiliar nodes inspectable in full.
fn review_text(node: &serde_json::Value) -> Result<String, ProjectError> {
    let kind = node
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or(ProjectError::InvalidDatabase)?;
    match kind {
        "text" => node
            .get("text")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .ok_or(ProjectError::InvalidDatabase),
        "hardBreak" => Ok("\n".to_owned()),
        "doc" | "paragraph" | "heading" | "codeBlock" | "blockquote" | "listItem"
        | "orderedList" | "bulletList" => {
            let Some(children) = node.get("content") else {
                return Ok(String::new());
            };
            let children = children.as_array().ok_or(ProjectError::InvalidDatabase)?;
            let rendered = children
                .iter()
                .map(review_text)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(
                rendered.join(if matches!(kind, "paragraph" | "heading" | "codeBlock") {
                    ""
                } else {
                    "\n"
                }),
            )
        }
        _ => Ok(node.to_string()),
    }
}

fn review_view(
    db: &Connection,
    project: ProjectId,
    scene: EntityId,
) -> Result<(ReviewedScene, Option<String>, i64, String), ProjectError> {
    let (title,text,content,revision,parent,position): (String,String,String,i64,Option<String>,i64) = db.query_row(
        "SELECT title,plain_text,content_json,revision,parent_id,position FROM document WHERE project_id=?1 AND id=?2 AND kind='scene'",
        params![project.to_string(),scene.to_string()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
    if content.len() > MAX_EDITOR_DOCUMENT_BYTES || revision < 0 {
        return Err(ProjectError::InvalidDatabase);
    }
    let doc = serde_json::from_str(&content).map_err(|_| ProjectError::InvalidDatabase)?;
    validate_content(&doc).map_err(|_| ProjectError::InvalidDatabase)?;
    if plain_text(&doc) != text {
        return Err(ProjectError::InvalidDatabase);
    }
    let text = review_text(&doc.document)?;
    Ok((
        ReviewedScene {
            title,
            text,
            revision,
        },
        parent,
        position,
        content,
    ))
}

impl ProjectManager {
    /// Provides exactly one divergent pair to a native selection window.
    /// An external selection prepares a distinct copy after rechecking both versions.
    ///
    /// # Errors
    /// Rejects missing, moved, stale or unverified scenes and unavailable isolation.
    pub fn review_scene_conflict(
        &self,
        id: ProjectId,
        selection: &SceneMergeSelection,
        display: impl FnOnce(&ReviewedScene, &ReviewedScene) -> Result<ReviewChoice, ProjectError>,
    ) -> Result<ReviewOutcome, ProjectError> {
        let [scene] = selection.scenes.as_slice() else {
            return Err(ProjectError::InvalidDatabase);
        };
        let (left, right) = self.with_scene_basis(id, |local, dir, db| {
            let base = Basis::read(dir, &selection.base, id)?;
            let incoming = Basis::read(dir, &selection.incoming, id)?;
            let src = package_database(
                id,
                dir,
                &selection.folder,
                &selection.manifest,
                &selection.incoming,
            )?;
            if hash(&local.bytes()?) != selection.local
                || handoff::schema(&src)? != handoff::schema(db)?
                || incoming.bytes()? != Basis::from_db(&src, id)?.bytes()?
            {
                return Err(ProjectError::InvalidDatabase);
            }
            let hashes = [&base, &local, &incoming].map(|b| b.scenes.get(scene));
            if hashes.contains(&None)
                || hashes[0] == hashes[1]
                || hashes[1] == hashes[2]
                || hashes[0] == hashes[2]
            {
                return Err(ProjectError::InvalidDatabase);
            }
            let (mut left, lp, li, local_doc) = review_view(db, id, *scene)?;
            let (mut right, rp, ri, incoming_doc) = review_view(&src, id, *scene)?;
            if lp != rp || li != ri {
                return Err(ProjectError::InvalidDatabase);
            }
            if left.title == right.title && left.text == right.text {
                // The plain view alone cannot distinguish formatting. Show the exact structured
                // document in the isolated window alongside the text before a whole-version choice.
                left.text
                    .push_str("\n\nDokumentstruktur und Formatierung\n");
                left.text.push_str(&local_doc);
                right
                    .text
                    .push_str("\n\nDokumentstruktur und Formatierung\n");
                right.text.push_str(&incoming_doc);
            }
            Ok((left, right))
        })?;
        match display(&left, &right)? {
            ReviewChoice::Cancel => Ok(ReviewOutcome::Cancel),
            ReviewChoice::Local => self.with_scene_basis(id, |local, _, db| {
                if hash(&local.bytes()?) != selection.local
                    || review_revision(db, id, *scene)? != left.revision
                {
                    return Err(ProjectError::InvalidDatabase);
                }
                Ok(ReviewOutcome::Local)
            }),
            ReviewChoice::Incoming => Ok(ReviewOutcome::Incoming {
                candidate: self.prepare_selected_handoff(
                    id,
                    Some((selection, Some([left.revision, right.revision]))),
                )?,
            }),
        }
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
        self.prepare_selected_handoff(id, Some((selection, None)))
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
        let outcome = local
            .review_scene_conflict(id, &plan, |left, right| {
                assert_eq!(left.text, "Früherer Entwurf des Hafens");
                assert_eq!(right.text, "Das Boot legt an.");
                Ok(ReviewChoice::Incoming)
            })
            .unwrap();
        let ReviewOutcome::Incoming { candidate } = outcome else {
            panic!("missing candidate")
        };
        let marker = here.directory().join(format!(
            ".fassungswahl-{}",
            conflict_approval(id, &plan, &candidate, read(live).2.into()).unwrap()
        ));
        let parked = local_home.path().join("parked-choice");
        fs::rename(&marker, &parked).unwrap();
        assert!(
            local
                .secure_scene_activation(id, &plan, &candidate)
                .is_err()
        );
        fs::rename(parked, marker).unwrap();
        let ticket = local
            .secure_scene_activation(id, &plan, &candidate)
            .unwrap();
        local.switch_scene_activation(id, &ticket, false).unwrap();
        assert_eq!(read(live).1, "Das Boot legt an.");
        local.switch_scene_activation(id, &ticket, true).unwrap();
        assert_eq!(read(live).1, "Früherer Entwurf des Hafens");
        let candidate =
            open_read_only(&backups.join(candidate.folder).join(DATABASE_FILE_NAME)).unwrap();
        assert_eq!(read(&candidate).1, "Das Boot legt an.");
        let original = read(live);
        assert!(
            local
                .review_scene_conflict(id, &plan, |_, _| {
                    live.execute(
                        "UPDATE document SET revision=revision+1 WHERE id=?1",
                        [selected.to_string()],
                    )
                    .unwrap();
                    Ok(ReviewChoice::Incoming)
                })
                .is_err()
        );
        assert_eq!(read(live).1, original.1);
    }
}
