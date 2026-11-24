//! Explicit adoption of one media identity, including its two image files and own reference rows.
use super::item_handoff::{
    ItemKey, ItemKind, ItemRow, ItemSelection, item_row, load_item, remove_item, write_item,
};
use super::*;
use crate::media_repository as catalog;
use kraqel_asset_service::{MAX_ASSET_BYTES, MAX_THUMBNAIL_BYTES, validate_handoff_image};
use kraqel_domain::{EntityId, ProjectId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, io::Read};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MediaState {
    pub row: ItemRow,
    pub links: Vec<ItemRow>,
    pub original_hash: String,
    pub thumb_hash: String,
}
fn invalid() -> ProjectError {
    ProjectError::InvalidDatabase
}
fn text<'a>(row: &'a ItemRow, key: &str) -> Result<&'a str, ProjectError> {
    row.get(key).and_then(Value::as_str).ok_or_else(invalid)
}
fn number(row: &ItemRow, key: &str) -> Result<u32, ProjectError> {
    row.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(invalid)
}
fn hash_of<T: Serialize>(v: &T) -> Result<String, ProjectError> {
    Ok(scene_basis::hash(
        &serde_json::to_vec(v).map_err(|_| invalid())?,
    ))
}
pub(super) fn validate_catalog_row(row: &ItemRow) -> Result<(), ProjectError> {
    catalog::validate_file_contract(
        text(row, "relative_path")?,
        text(row, "thumbnail_path")?,
        text(row, "original_name")?,
        text(row, "mime_type")?,
        text(row, "checksum")?,
        u64::from(number(row, "byte_count")?),
        number(row, "width")?,
        number(row, "height")?,
    )
    .map_err(|_| invalid())?;
    let tags: Vec<String> = serde_json::from_str(text(row, "tags_json")?).map_err(|_| invalid())?;
    catalog::validate_metadata(text(row, "title")?, &tags).map_err(|_| invalid())?;
    Ok(())
}

fn links(db: &Connection, id: ProjectId, key: &ItemKey) -> Result<Vec<ItemRow>, ProjectError> {
    let [asset] = key.ids.as_slice() else {
        return Err(invalid());
    };
    let mut stmt=db.prepare("SELECT * FROM asset_link WHERE project_id=?1 AND asset_id=?2 ORDER BY target_type,target_id,role")?;
    let mut rows = stmt.query(params![id.to_string(), asset])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(item_row(row)?);
    }
    Ok(out)
}
pub(super) fn catalog_digest(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    row: &ItemRow,
) -> Result<String, ProjectError> {
    hash_of(&("media-catalog-v1", row, links(db, id, key)?))
}

impl MediaState {
    fn catalog_digest(&self) -> Result<String, ProjectError> {
        hash_of(&("media-catalog-v1", &self.row, &self.links))
    }
    /// Storage paths may be remapped in a candidate; the actual bytes and every logical field stay bound.
    pub fn digest(&self) -> Result<String, ProjectError> {
        let mut logical = self.row.clone();
        logical.remove("relative_path");
        logical.remove("thumbnail_path");
        hash_of(&(
            "media-content-v1",
            &logical,
            &self.links,
            &self.original_hash,
            &self.thumb_hash,
        ))
    }

    pub fn managed(&self) -> Result<Self, ProjectError> {
        let asset = text(&self.row, "id")?;
        if EntityId::parse(asset).is_none_or(|id| id.to_string() != asset)
            || !scene_basis::is_hash(&self.original_hash)
            || !scene_basis::is_hash(&self.thumb_hash)
        {
            return Err(invalid());
        }
        let ext = match text(&self.row, "mime_type")? {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => return Err(invalid()),
        };
        let mut managed = self.clone();
        managed.row.insert(
            "relative_path".into(),
            Value::String(format!("assets/{asset}-{}.{ext}", self.original_hash)),
        );
        managed.row.insert(
            "thumbnail_path".into(),
            Value::String(format!("assets/thumbnails/{asset}-{}.png", self.thumb_hash)),
        );
        Ok(managed)
    }
}
fn bytes(dir: &Path, path: &str, limit: usize) -> Result<Vec<u8>, ProjectError> {
    let f = handoff::pin_asset(dir, path, limit as u64)?;
    let mut out = Vec::new();
    f.take(limit as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|_| invalid())?;
    if out.len() > limit {
        return Err(invalid());
    }
    Ok(out)
}

pub(super) fn read_state(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    dir: &Path,
) -> Result<Option<MediaState>, ProjectError> {
    read_bound_state(db, id, key, dir, None)
}
fn read_bound_state(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    dir: &Path,
    approved: Option<&BTreeMap<String, String>>,
) -> Result<Option<MediaState>, ProjectError> {
    if key.kind != ItemKind::MediaAsset {
        return Err(invalid());
    }
    let Some(row) = load_item(db, id, key)? else {
        return Ok(None);
    };
    validate_catalog_row(&row)?;
    let path = text(&row, "relative_path")?;
    let thumb_path = text(&row, "thumbnail_path")?;
    let expected = text(&row, "checksum")?;
    if approved.is_some_and(|files| {
        files.get(path).map(String::as_str) != Some(expected) || !files.contains_key(thumb_path)
    }) {
        return Err(invalid());
    }
    let original = bytes(dir, path, MAX_ASSET_BYTES)?;
    let thumb = bytes(dir, thumb_path, MAX_THUMBNAIL_BYTES)?;
    let original_hash = scene_basis::hash(&original);
    let thumb_hash = scene_basis::hash(&thumb);
    if original.len() != number(&row, "byte_count")? as usize
        || original_hash != expected
        || approved.is_some_and(|files| files.get(thumb_path) != Some(&thumb_hash))
    {
        return Err(invalid());
    }
    validate_handoff_image(
        &original,
        &thumb,
        text(&row, "mime_type")?,
        number(&row, "width")?,
        number(&row, "height")?,
    )
    .map_err(|_| invalid())?;
    let state = MediaState {
        row,
        links: links(db, id, key)?,
        original_hash,
        thumb_hash,
    };
    validate_link_targets(db, id, key, &state.links)?;
    Ok(Some(state))
}
fn validate_link_targets(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    links: &[ItemRow],
) -> Result<(), ProjectError> {
    for link in links {
        if text(link, "project_id")? != id.to_string() || text(link, "asset_id")? != key.ids[0] {
            return Err(invalid());
        }
        let target = catalog::parse_target(text(link, "target_type")?).map_err(|_| invalid())?;
        let raw = text(link, "target_id")?;
        let target_id = EntityId::parse(raw)
            .filter(|v| v.to_string() == raw)
            .ok_or_else(invalid)?;
        catalog::require_target(db, id, target, target_id).map_err(|_| invalid())?;
        catalog::validate_role(text(link, "role")?).map_err(|_| invalid())?;
    }
    Ok(())
}

/// Operates only inside the caller's transaction. Reject all implicit CASCADE / SET NULL changes.
pub(super) fn apply_state(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    next: Option<&MediaState>,
) -> Result<(), ProjectError> {
    if db.is_autocommit() || key.kind != ItemKind::MediaAsset || key.ids.len() != 1 {
        return Err(invalid());
    }
    if let Some(next) = next {
        validate_link_targets(db, id, key, &next.links)?;
    }
    let start = db.total_changes();
    let removed = db.execute(
        "DELETE FROM asset_link WHERE project_id=?1 AND asset_id=?2",
        params![id.to_string(), key.ids[0]],
    )?;
    let mut expected = removed as u64 + 1;
    if let Some(next) = next {
        write_item(db, id, key, &next.row)?;
        for link in &next.links {
            db.execute("INSERT INTO asset_link(project_id,asset_id,target_type,target_id,role,created_at) VALUES(?1,?2,?3,?4,?5,?6)", params![id.to_string(),key.ids[0],text(link,"target_type")?,text(link,"target_id")?,text(link,"role")?,text(link,"created_at")?])?;
            expected += 1;
        }
        if links(db, id, key)? != next.links {
            return Err(invalid());
        }
    } else {
        remove_item(db, id, key)?;
    }
    if db.total_changes().checked_sub(start) != Some(expected) {
        return Err(invalid());
    }
    Ok(())
}

impl ProjectManager {
    pub(super) fn with_media_source<T>(
        &self,
        id: ProjectId,
        s: &ItemSelection,
        run: impl FnOnce(&Path, &MediaState) -> Result<T, ProjectError>,
    ) -> Result<T, ProjectError> {
        self.with_inbox_manifest(&s.folder, &s.manifest, |dir, manifest| {
            if manifest.project_id != id {
                return Err(invalid());
            }
            let hash = manifest.files.get(DATABASE_FILE_NAME).ok_or_else(invalid)?;
            let db = scene_merge::read_package_database(dir, hash)?;
            handoff::validate_source(&db, Database::open_in_memory()?.connection(), id)?;
            let state = read_bound_state(&db, id, &s.change.key, dir, Some(&manifest.files))?
                .ok_or_else(invalid)?;
            run(dir, &state)
        })
    }

    /// Reviews only the selected media pair. File reads and native rendering require an explicit request.
    ///
    /// # Errors
    /// Invalid image data or a changed local/source state rejects candidate preparation.
    pub fn review_media_handoff(
        &self,
        id: ProjectId,
        s: &ItemSelection,
        display: impl FnOnce(&ItemSelection, Option<&str>, &str) -> Result<ReviewChoice, ProjectError>,
    ) -> Result<(ReviewChoice, Option<(ItemSelection, PreparedHandoff)>), ProjectError> {
        let (bound, previous, incoming) = self.with_media_source(id, s, |_, remote| {
            self.with_read_snapshot(id, |backups, db| {
                let dir = backups.parent().ok_or_else(invalid)?;
                handoff::validate_source(db, Database::open_in_memory()?.connection(), id)?;
                let local = read_state(db, id, &s.change.key, dir)?;
                if local.as_ref().map(MediaState::catalog_digest).transpose()? != s.change.local
                    || Some(remote.catalog_digest()?) != s.change.incoming
                {
                    return Err(invalid());
                }
                let mut bound = s.clone();
                bound.change.local = local.as_ref().map(MediaState::digest).transpose()?;
                bound.change.incoming = Some(remote.digest()?);
                let render =
                    |v: &MediaState| serde_json::to_string_pretty(v).map_err(|_| invalid());
                Ok((
                    bound,
                    local.as_ref().map(render).transpose()?,
                    render(remote)?,
                ))
            })
        })?;
        let choice = display(&bound, previous.as_deref(), &incoming)?;
        if matches!(choice, ReviewChoice::Incoming) && bound.change.local != bound.change.incoming {
            let candidate = self.prepare_changed_handoff(id, None, Some(&bound))?;
            Ok((choice, Some((bound, candidate))))
        } else {
            // Choosing either of two identical logical versions leaves the local project active.
            Ok((
                if matches!(choice, ReviewChoice::Incoming) {
                    ReviewChoice::Local
                } else {
                    choice
                },
                None,
            ))
        }
    }

    pub(super) fn stage_selected_media(
        &self,
        id: ProjectId,
        s: &ItemSelection,
        live: &Path,
        stage: &Path,
        fresh: &mut Database,
        files: &mut BTreeMap<String, String>,
    ) -> Result<(), ProjectError> {
        let current = read_state(fresh.connection(), id, &s.change.key, live)?;
        if current.as_ref().map(MediaState::digest).transpose()? != s.change.local {
            return Err(invalid());
        }
        self.with_media_source(id, s, |dir, incoming| {
            if Some(incoming.digest()?) != s.change.incoming {
                return Err(invalid());
            }
            let managed = incoming.managed()?;
            for (field, digest, limit) in [
                ("relative_path", &incoming.original_hash, MAX_ASSET_BYTES),
                ("thumbnail_path", &incoming.thumb_hash, MAX_THUMBNAIL_BYTES),
            ] {
                let target = text(&managed.row, field)?;
                let content = bytes(dir, text(&incoming.row, field)?, limit)?;
                if scene_basis::hash(&content) != *digest || files.contains_key(target) {
                    return Err(invalid());
                }
                let path = stage.join(target);
                fs::create_dir_all(path.parent().ok_or_else(invalid)?).map_err(|_| invalid())?;
                let mut dest = fs::File::create_new(path).map_err(|_| invalid())?;
                dest.write_all(&content)
                    .and_then(|()| dest.sync_all())
                    .map_err(|_| invalid())?;
                files.insert(target.to_owned(), digest.clone());
            }
            let tx = fresh.transaction()?;
            apply_state(&tx, id, &s.change.key, Some(&managed))?;
            tx.commit()?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_asset_service::{PreparedImage, prepare_image};
    use tempfile::tempdir;

    fn replace_bytes(dir: &Path, image: &PreparedImage) {
        fs::write(dir.join("assets/test.png"), &image.original_bytes).unwrap();
        fs::write(dir.join("assets/thumbnails/test.png"), &image.thumbnail_png).unwrap();
    }

    #[test]
    fn reviewed_media_binds_own_files_and_links_from_source_to_undo() {
        let root = tempdir().unwrap();
        let manager = ProjectManager::new(root.path()).unwrap();
        let id = ProjectId::new();
        let project = Project {
            id,
            name: "Bilderauswahl".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let opened = manager.create_project(id.to_string(), &project).unwrap();
        let db = opened.database().connection();
        let dir = opened.directory();
        fs::create_dir_all(dir.join("assets/thumbnails")).unwrap();
        let remote = prepare_image(
            "test.png",
            include_bytes!("../../../asset-service/tests/fixtures/scene.png"),
        )
        .unwrap();
        let local = prepare_image(
            "test.png",
            include_bytes!("../../../asset-service/tests/fixtures/metadata.png"),
        )
        .unwrap();
        let asset = EntityId::new().to_string();
        let entity = EntityId::new().to_string();
        let project_id = id.to_string();
        replace_bytes(dir, &remote);
        db.execute("INSERT INTO entity(id,project_id,kind,name,created_at,updated_at) VALUES(?,?,'character','Figur','t','t')",params![entity,project_id]).unwrap();
        db.execute("INSERT INTO media_asset(id,project_id,kind,relative_path,thumbnail_path,original_name,title,mime_type,checksum,byte_count,width,height,provenance,created_at,updated_at) VALUES(?,?,'image','assets/test.png','assets/thumbnails/test.png','test.png','Ferne Fassung','image/png',?,?,?,?,'imported','t','t')",params![asset,project_id,remote.checksum,remote.original_bytes.len() as i64,remote.width,remote.height]).unwrap();
        db.execute(
            "INSERT INTO asset_link VALUES(?,?,'entity',?,'reference','t')",
            params![project_id, asset, entity],
        )
        .unwrap();
        let receipt = manager.prepare_handoff(id).unwrap();
        let inbox = root.path().join("empfang");
        fs::create_dir(&inbox).unwrap();
        let incoming = inbox.join(&receipt.folder);
        fs::rename(
            dir.join(BACKUP_DIRECTORY_NAME).join(&receipt.folder),
            &incoming,
        )
        .unwrap();
        fs::write(incoming.join("private.txt"), b"do not copy").unwrap();
        let selection = || {
            let mut infos = manager
                .list_handoff_items(
                    id,
                    &receipt.folder,
                    &receipt.manifest_sha256,
                    ItemKind::MediaAsset,
                )
                .unwrap();
            assert_eq!(infos.len(), 1);
            ItemSelection {
                change: infos.pop().unwrap(),
                folder: receipt.folder.clone(),
                manifest: receipt.manifest_sha256.clone(),
            }
        };

        let same_catalog = selection();
        assert_eq!(same_catalog.change.local, same_catalog.change.incoming);
        fs::write(dir.join("assets/thumbnails/test.png"), &local.thumbnail_png).unwrap();
        let (choice, candidate) = manager
            .review_media_handoff(id, &same_catalog, |full, _, _| {
                assert_ne!(full.change.local, full.change.incoming);
                Ok(ReviewChoice::Cancel)
            })
            .unwrap();
        assert!(matches!(choice, ReviewChoice::Cancel) && candidate.is_none());

        replace_bytes(dir, &local);
        db.execute("UPDATE media_asset SET title='Lokale Fassung',checksum=?,byte_count=?,width=?,height=? WHERE id=?",params![local.checksum,local.original_bytes.len() as i64,local.width,local.height,asset]).unwrap();
        db.execute("DELETE FROM asset_link WHERE asset_id=?", [&asset])
            .unwrap();
        let current = selection();
        assert!(
            manager
                .review_media_handoff(id, &current, |_, _, _| {
                    fs::write(
                        dir.join("assets/thumbnails/test.png"),
                        &remote.thumbnail_png,
                    )
                    .unwrap();
                    Ok(ReviewChoice::Incoming)
                })
                .is_err()
        );
        fs::write(dir.join("assets/thumbnails/test.png"), &local.thumbnail_png).unwrap();

        let (_, choice) = manager
            .review_media_handoff(id, &current, |_, before, after| {
                assert!(before.unwrap().contains("Lokale Fassung"));
                assert!(after.contains("Ferne Fassung") && after.contains(&entity));
                Ok(ReviewChoice::Incoming)
            })
            .unwrap();
        let (bound, candidate) = choice.unwrap();
        let result = dir.join(BACKUP_DIRECTORY_NAME).join(&candidate.folder);
        assert!(!result.join("private.txt").exists());
        assert!(!result.join("assets/test.png").exists());
        let stored = read_state(
            &open_read_only(&result.join(DATABASE_FILE_NAME)).unwrap(),
            id,
            &bound.change.key,
            &result,
        )
        .unwrap()
        .unwrap();
        assert_eq!(Some(stored.digest().unwrap()), bound.change.incoming);
        assert_eq!(stored.links.len(), 1);
        assert_eq!(
            fs::read(dir.join("assets/test.png")).unwrap(),
            local.original_bytes
        );
        crate::project_manager::media_activation::check_activation(
            &manager,
            id,
            &bound,
            &candidate,
            &incoming,
            &bound.change.key,
        );
    }
}
