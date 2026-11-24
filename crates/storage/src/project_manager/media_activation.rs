//! File generations for individual media activation. Only the SQLite commit makes a pair visible.
use super::handoff::{asset_parts, pin_asset, pin_local_directory};
use super::item_handoff::{ItemKey, ItemSelection};
use super::media_handoff::{self, MediaState};
use super::scene_activation::publish_receipt;
use super::scene_basis::{hash, is_hash};
use super::scene_merge::{package_database, validate_package_folder};
use super::{PreparedHandoff, ProjectError, ProjectManager, handoff};
use kraqel_domain::ProjectId;
use rusqlite::{Connection, Transaction};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const MAX_IMAGE: u64 = 25 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MediaActivation {
    previous: Option<MediaState>,
    incoming: MediaState,
}

pub(super) struct MediaSwitch<'a> {
    pub project_id: ProjectId,
    pub selection: &'a ItemSelection,
    pub candidate: &'a PreparedHandoff,
    pub undo: bool,
    pub intent: bool,
}

fn invalid() -> ProjectError {
    ProjectError::InvalidDatabase
}

fn media_io(source: io::Error) -> ProjectError {
    ProjectError::Io {
        operation: "Lokale Mediendatei binden",
        source,
    }
}

fn files(state: &MediaState) -> Result<[(&str, &str); 2], ProjectError> {
    let name = |column| {
        state
            .row
            .get(column)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(invalid)
    };
    if !is_hash(&state.original_hash) || !is_hash(&state.thumb_hash) {
        return Err(invalid());
    }
    Ok([
        (name("relative_path")?, &state.original_hash),
        (name("thumbnail_path")?, &state.thumb_hash),
    ])
}

// Retain each ancestor until the enclosing SQLite transaction commits. Holding only the leaf
// would allow another process to rename its parent and replace the path visible to readers.
fn pin_parents(
    dir: &Path,
    relative: &str,
    create: bool,
    pins: &mut Vec<fs::File>,
) -> Result<(), ProjectError> {
    let parts = asset_parts(relative)?;
    let mut parent = dir.to_owned();
    for part in &parts[..parts.len() - 1] {
        parent.push(part);
        if create {
            match fs::create_dir(&parent) {
                Ok(()) => (),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(media_io(e)),
            }
        }
        pins.push(pin_local_directory(&parent)?);
    }
    Ok(())
}

fn verified_bytes(
    dir: &Path,
    name: &str,
    expected: &str,
    pins: &mut Vec<fs::File>,
) -> Result<Vec<u8>, ProjectError> {
    pin_parents(dir, name, false, pins)?;
    read_verified(pin_asset(dir, name, MAX_IMAGE)?, expected, pins)
}

fn backup_bytes(
    backups: &Path,
    digest: &str,
    pins: &mut Vec<fs::File>,
) -> Result<Vec<u8>, ProjectError> {
    let path = backups.join(format!("medieninhalt-{digest}.bin"));
    read_verified(handoff::pinned_file(&path, false)?, digest, pins)
}

fn read_verified(
    mut file: fs::File,
    expected: &str,
    pins: &mut Vec<fs::File>,
) -> Result<Vec<u8>, ProjectError> {
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_IMAGE + 1)
        .read_to_end(&mut bytes)
        .map_err(media_io)?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_IMAGE || hash(&bytes) != expected {
        return Err(invalid());
    }
    pins.push(file);
    Ok(bytes)
}

fn copy_to_receipt(
    dir: &Path,
    state: &MediaState,
    backups: &Path,
    pins: &mut Vec<fs::File>,
) -> Result<(), ProjectError> {
    for (name, digest) in files(state)? {
        let bytes = verified_bytes(dir, name, digest, pins)?;
        publish_receipt(&backups.join(format!("medieninhalt-{digest}.bin")), &bytes)?;
    }
    Ok(())
}

fn check_owner(
    db: &Connection,
    project: ProjectId,
    key: &ItemKey,
    relative: &str,
) -> Result<(), ProjectError> {
    let [asset] = key.ids.as_slice() else {
        return Err(invalid());
    };
    let used: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM media_asset WHERE project_id=?1 AND id<>?2
         AND (relative_path=?3 COLLATE NOCASE OR thumbnail_path=?3 COLLATE NOCASE))",
        rusqlite::params![project.to_string(), asset, relative],
        |r| r.get(0),
    )?;
    if used {
        return Err(invalid());
    }
    Ok(())
}

fn install(
    db: &Connection,
    dir: &Path,
    backups: &Path,
    state: &MediaState,
    id: ProjectId,
    key: &ItemKey,
    pins: &mut Vec<fs::File>,
) -> Result<(), ProjectError> {
    // Check both destinations before publishing either one. Even a byte-identical existing
    // path may not be claimed from another database row.
    let mut missing = Vec::new();
    for (name, digest) in files(state)? {
        check_owner(db, id, key, name)?;
        pin_parents(dir, name, true, pins)?;
        match fs::symlink_metadata(dir.join(name)) {
            Ok(_) => {
                verified_bytes(dir, name, digest, pins)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                missing.push((name, digest, backup_bytes(backups, digest, pins)?));
            }
            Err(e) => return Err(media_io(e)),
        }
    }
    for (name, digest, bytes) in missing {
        publish_receipt(&dir.join(name), &bytes)?;
        verified_bytes(dir, name, digest, pins)?;
    }
    Ok(())
}

fn candidate_state(
    tx: &Connection,
    backups: &Path,
    id: ProjectId,
    selection: &ItemSelection,
    candidate: &PreparedHandoff,
    pins: &mut Vec<fs::File>,
) -> Result<MediaState, ProjectError> {
    validate_package_folder(&candidate.folder)?;
    let dir = backups.join(&candidate.folder);
    pins.push(pin_local_directory(&dir)?);
    let data = package_database(
        id,
        backups,
        &candidate.folder,
        &candidate.manifest_sha256,
        &candidate.scene_basis_sha256,
    )?;
    handoff::validate_source(&data, tx, id)?;
    let state =
        media_handoff::read_state(&data, id, &selection.change.key, &dir)?.ok_or_else(invalid)?;
    if state != state.managed()? || Some(state.digest()?) != selection.change.incoming {
        return Err(invalid());
    }
    Ok(state)
}

impl MediaActivation {
    pub(super) fn secure(
        manager: &ProjectManager,
        tx: &Transaction<'_>,
        backups: &Path,
        id: ProjectId,
        selection: &ItemSelection,
        candidate: &PreparedHandoff,
        pins: &mut Vec<fs::File>,
    ) -> Result<Self, ProjectError> {
        let dir = manager.root.join(id.to_string());
        let previous = media_handoff::read_state(tx, id, &selection.change.key, &dir)?;
        if previous.as_ref().map(MediaState::digest).transpose()? != selection.change.local {
            return Err(invalid());
        }
        let incoming = candidate_state(tx, backups, id, selection, candidate, pins)?;
        manager.with_media_source(id, selection, |_, external| {
            if external.managed()? != incoming {
                return Err(invalid());
            }
            Ok(())
        })?;
        if let Some(old) = &previous {
            copy_to_receipt(&dir, old, backups, pins)?;
        }
        copy_to_receipt(&backups.join(&candidate.folder), &incoming, backups, pins)?;
        Ok(Self { previous, incoming })
    }

    /// Returns true exactly when an intent must still be published before COMMIT.
    pub(super) fn switch(
        &self,
        manager: &ProjectManager,
        tx: &Transaction<'_>,
        backups: &Path,
        transition: &MediaSwitch<'_>,
        pins: &mut Vec<fs::File>,
    ) -> Result<bool, ProjectError> {
        let id = transition.project_id;
        let selection = transition.selection;
        let dir = manager.root.join(id.to_string());
        if self.previous.as_ref().map(MediaState::digest).transpose()? != selection.change.local
            || Some(self.incoming.digest()?) != selection.change.incoming
            || self.incoming != self.incoming.managed()?
        {
            return Err(invalid());
        }
        let (from, to) = if transition.undo {
            (Some(&self.incoming), self.previous.as_ref())
        } else {
            (self.previous.as_ref(), Some(&self.incoming))
        };
        let current = media_handoff::read_state(tx, id, &selection.change.key, &dir)?;
        if let Some(state) = &current {
            for (name, digest) in files(state)? {
                verified_bytes(&dir, name, digest, pins)?;
            }
        }
        if current.as_ref() == to && transition.intent {
            return Ok(false);
        }
        if transition.intent || current.as_ref() != from {
            return Err(invalid());
        }
        if !transition.undo {
            // Once the new DB state is visible, even removing the input or old unreferenced
            // files must leave a complete local way back. Verify before publishing intent.
            if let Some(old) = &self.previous {
                for (_, digest) in files(old)? {
                    backup_bytes(backups, digest, pins)?;
                }
            }
            manager.with_media_source(id, selection, |_, external| {
                if external.managed()? != self.incoming {
                    return Err(invalid());
                }
                Ok(())
            })?;
            if candidate_state(tx, backups, id, selection, transition.candidate, pins)?
                != self.incoming
            {
                return Err(invalid());
            }
        }
        media_handoff::apply_state(tx, id, &selection.change.key, to)?;
        if let Some(state) = to {
            install(tx, &dir, backups, state, id, &selection.change.key, pins)?;
        }
        if media_handoff::read_state(tx, id, &selection.change.key, &dir)?.as_ref() != to {
            return Err(invalid());
        }
        Ok(true)
    }
}

#[cfg(test)]
pub(super) fn check_activation(
    manager: &ProjectManager,
    id: ProjectId,
    selection: &ItemSelection,
    candidate: &PreparedHandoff,
    incoming_folder: &Path,
    key: &ItemKey,
) {
    let project = manager.open_project(id.to_string()).unwrap();
    let dir = project.directory();
    let db = project.database().connection();
    let current = || {
        media_handoff::read_state(db, id, key, dir)
            .unwrap()
            .unwrap()
    };
    let before = current();
    let backups = dir.join(super::BACKUP_DIRECTORY_NAME);
    let candidate_dir = backups.join(&candidate.folder);
    let expected = {
        let copy = super::open_read_only(&candidate_dir.join(super::DATABASE_FILE_NAME)).unwrap();
        media_handoff::read_state(&copy, id, key, &candidate_dir)
            .unwrap()
            .unwrap()
    };
    let ticket = manager
        .secure_handoff_item(id, selection, candidate)
        .unwrap();

    // A foreign file occupying just the second target must survive; no first file becomes visible.
    let [(original, _), (thumbnail, _)] = files(&expected).unwrap();
    assert!(!dir.join(original).exists());
    let thumb = dir.join(thumbnail);
    fs::write(&thumb, b"foreign fixture").unwrap();
    assert!(manager.switch_handoff_item(id, &ticket, false).is_err());
    assert!(current() == before && !dir.join(original).exists());
    assert_eq!(fs::read(&thumb).unwrap(), b"foreign fixture");
    fs::remove_file(thumb).unwrap();

    manager.switch_handoff_item(id, &ticket, false).unwrap();
    assert!(current() == expected);
    // Undo must fail closed if someone added a relationship after approval.
    let inserted = db
        .execute(
            "INSERT INTO asset_link (project_id,asset_id,target_type,target_id,role,created_at)
         SELECT project_id,asset_id,target_type,target_id,'neue Zuordnung','test'
         FROM asset_link WHERE project_id=?1 AND asset_id=?2 LIMIT 1",
            rusqlite::params![id.to_string(), key.ids[0]],
        )
        .unwrap();
    assert_eq!(inserted, 1);
    assert!(manager.switch_handoff_item(id, &ticket, true).is_err());
    assert_eq!(
        db.execute(
            "DELETE FROM asset_link WHERE project_id=?1 AND asset_id=?2 AND role='neue Zuordnung'",
            rusqlite::params![id.to_string(), key.ids[0]]
        )
        .unwrap(),
        1
    );
    let local_original = dir.join(files(&before).unwrap()[0].0);
    fs::remove_file(&local_original).unwrap();
    let held_input = incoming_folder.with_file_name("held-input");
    let held_copy = backups.join("held-candidate");
    fs::rename(incoming_folder, &held_input).unwrap();
    fs::rename(&candidate_dir, &held_copy).unwrap();
    let restarted = ProjectManager::new(manager.root()).unwrap();
    restarted.switch_handoff_item(id, &ticket, true).unwrap();
    assert!(current() == before && local_original.is_file());
    assert!(restarted.switch_handoff_item(id, &ticket, false).is_err());
    fs::rename(held_input, incoming_folder).unwrap();
    fs::rename(held_copy, candidate_dir).unwrap();
}
