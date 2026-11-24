//! Locally approved activation and undo of an individual project row.
use super::item_handoff::{
    ItemKey, ItemRow, ItemSelection, load_item, remove_item, row_hash, write_item,
};
use super::scene_activation::publish_receipt;
use super::scene_basis::{hash, is_hash};
use super::scene_merge::{confirmed_file, package_database};
use super::{PreparedHandoff, ProjectError, ProjectManager, handoff};
use kraqel_domain::ProjectId;
use rusqlite::Connection;
#[cfg(test)]
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

const FORMAT: &str = "kraqel-item-activation-v1";
const MAX_RECORD: u64 = 128 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    format: String,
    project: ProjectId,
    selection: ItemSelection,
    candidate: PreparedHandoff,
    previous: Option<ItemRow>,
}

fn approval(
    id: ProjectId,
    selection: &ItemSelection,
    candidate: &PreparedHandoff,
) -> Result<String, ProjectError> {
    Ok(hash(
        &serde_json::to_vec(&(FORMAT, id, selection, candidate))
            .map_err(|_| ProjectError::InvalidDatabase)?,
    ))
}

/// Called only by the preparation path after native confirmation of this exact choice.
pub(super) fn approve_candidate(
    project_dir: &Path,
    id: ProjectId,
    selection: &ItemSelection,
    candidate: &PreparedHandoff,
) -> Result<(), ProjectError> {
    let digest = approval(id, selection, candidate)?;
    publish_receipt(
        &project_dir.join(format!(".eintragswahl-{digest}")),
        digest.as_bytes(),
    )
}

fn optional_hash(row: Option<&ItemRow>) -> Result<Option<String>, ProjectError> {
    row.map(row_hash).transpose()
}

fn verify_marker(path: &Path, ticket: &str) -> Result<bool, ProjectError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            confirmed_file(path, &hash(ticket.as_bytes()), 64)?;
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(ProjectError::Io {
            operation: "Lokalen Freigabestand prüfen",
            source,
        }),
    }
}

impl ProjectManager {
    /// Sichert die Ausgangszeile für genau eine lokal bestätigte Übernahme.
    ///
    /// # Errors
    /// Eine abweichende Auswahl, Kandidatenkopie oder Ausgangsfassung sperrt die Freigabe.
    pub fn secure_handoff_item(
        &self,
        id: ProjectId,
        selection: &ItemSelection,
        candidate: &PreparedHandoff,
    ) -> Result<String, ProjectError> {
        self.with_activation_transaction(id, |tx, backups| {
            let project = self.root.join(id.to_string());
            let proof = approval(id, selection, candidate)?;
            if !verify_marker(&project.join(format!(".eintragswahl-{proof}")), &proof)? {
                return Err(ProjectError::InvalidDatabase);
            }
            let previous = load_item(tx, id, &selection.change.key)?;
            if optional_hash(previous.as_ref())? != selection.change.local
                || selection.change.local == selection.change.incoming
                || selection.change.incoming.is_none()
            {
                return Err(ProjectError::InvalidDatabase);
            }
            verify_recoverable(tx, id, &selection.change.key, previous.as_ref())?;
            self.confirm_item_source(id, selection)?;
            candidate_row(tx, backups, id, selection, candidate)?;
            let record = Record {
                format: FORMAT.into(),
                project: id,
                selection: selection.clone(),
                previous,
                candidate: candidate.clone(),
            };
            let bytes = serde_json::to_vec(&record).map_err(|_| ProjectError::InvalidDatabase)?;
            if bytes.len() as u64 > MAX_RECORD {
                return Err(ProjectError::InvalidDatabase);
            }
            let ticket = hash(&bytes);
            publish_receipt(
                &backups.join(format!("eintragsaktivierung-{ticket}.json")),
                &bytes,
            )?;
            publish_receipt(
                &project.join(format!(".eintragsfreigabe-{ticket}")),
                ticket.as_bytes(),
            )?;
            Ok((ticket, None))
        })
    }

    fn confirm_item_source(
        &self,
        id: ProjectId,
        selection: &ItemSelection,
    ) -> Result<(), ProjectError> {
        self.with_item_source(id, &selection.folder, &selection.manifest, |db| {
            if optional_hash(load_item(db, id, &selection.change.key)?.as_ref())?
                != selection.change.incoming
            {
                return Err(ProjectError::InvalidDatabase);
            }
            Ok(())
        })
    }

    /// Aktiviert den gesicherten Eintrag oder stellt seinen vorherigen Stand wieder her.
    ///
    /// # Errors
    /// Fremde Änderungen und neu hinzugekommene Abhängigkeiten verhindern den Wechsel.
    pub fn switch_handoff_item(
        &self,
        id: ProjectId,
        ticket: &str,
        undo: bool,
    ) -> Result<(), ProjectError> {
        if !is_hash(ticket) {
            return Err(ProjectError::InvalidRelativePath);
        }
        self.with_activation_transaction(id, |tx, backups| {
            let project = self.root.join(id.to_string());
            if !verify_marker(&project.join(format!(".eintragsfreigabe-{ticket}")), ticket)? {
                return Err(ProjectError::InvalidDatabase);
            }
            let path = backups.join(format!("eintragsaktivierung-{ticket}.json"));
            let pin = handoff::pinned_file(&path, false)?;
            let raw = confirmed_file(&path, ticket, MAX_RECORD)?;
            let record: Record =
                serde_json::from_slice(&raw).map_err(|_| ProjectError::InvalidDatabase)?;
            if record.format != FORMAT
                || record.project != id
                || record.selection.change.incoming.is_none()
                || record.selection.change.local == record.selection.change.incoming
                || optional_hash(record.previous.as_ref())? != record.selection.change.local
            {
                return Err(ProjectError::InvalidDatabase);
            }
            let applied = project.join(format!(".eintrag-aktiviert-{ticket}"));
            let reversed = project.join(format!(".eintrag-zurueck-{ticket}"));
            let activated = verify_marker(&applied, ticket)?;
            let undone = verify_marker(&reversed, ticket)?;
            if (undo && !activated) || (!undo && undone) {
                return Err(ProjectError::InvalidDatabase);
            }
            let selection = &record.selection;
            let current = optional_hash(load_item(tx, id, &selection.change.key)?.as_ref())?;
            let (from, to, intent) = if undo {
                (&selection.change.incoming, &selection.change.local, undone)
            } else {
                (
                    &selection.change.local,
                    &selection.change.incoming,
                    activated,
                )
            };
            if current == *to && intent {
                return Ok(((), Some(pin)));
            }
            // These rows have no monotonic revision. A return to the exact source bytes
            // after an issued intent is ambiguous (interrupted COMMIT or later edits).
            // A used ticket may acknowledge its target but may never apply the same write twice.
            if intent || current != *from {
                return Err(ProjectError::InvalidDatabase);
            }
            let next = if undo {
                record.previous
            } else {
                // The time mode of the old timeline can change after approval.
                verify_recoverable(tx, id, &selection.change.key, record.previous.as_ref())?;
                self.confirm_item_source(id, selection)?;
                Some(candidate_row(
                    tx,
                    backups,
                    id,
                    selection,
                    &record.candidate,
                )?)
            };
            set_exact_row(tx, id, &selection.change.key, next.as_ref(), to.as_deref())?;
            // Publish under the SQLite writer lock. After loss of the reply a retry checks
            // whether the target row is already durable.
            publish_receipt(if undo { &reversed } else { &applied }, ticket.as_bytes())?;
            Ok(((), Some(pin)))
        })
    }
}

fn verify_recoverable(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    previous: Option<&ItemRow>,
) -> Result<(), ProjectError> {
    if let Some(row) = previous {
        super::item_handoff::validate_fields(db, id, key, row)?;
    }
    Ok(())
}

fn set_exact_row(
    db: &rusqlite::Transaction<'_>,
    id: ProjectId,
    key: &ItemKey,
    next: Option<&ItemRow>,
    expected: Option<&str>,
) -> Result<(), ProjectError> {
    let count = db.total_changes();
    match next {
        Some(row) => write_item(db, id, key, row)?,
        None => remove_item(db, id, key)?,
    }
    // Counts cascades, SET NULL actions and trigger writes including polymorphic asset links.
    // Returning an error rolls the entire surrounding transaction back before intent publication.
    if db.total_changes().checked_sub(count) != Some(1)
        || optional_hash(load_item(db, id, key)?.as_ref())?.as_deref() != expected
    {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(())
}

fn candidate_row(
    tx: &Connection,
    backups: &Path,
    id: ProjectId,
    selection: &ItemSelection,
    candidate: &PreparedHandoff,
) -> Result<ItemRow, ProjectError> {
    let db = package_database(
        id,
        backups,
        &candidate.folder,
        &candidate.manifest_sha256,
        &candidate.scene_basis_sha256,
    )?;
    handoff::validate_source(&db, tx, id)?;
    let row = load_item(&db, id, &selection.change.key)?.ok_or(ProjectError::InvalidDatabase)?;
    if Some(row_hash(&row)?) != selection.change.incoming {
        return Err(ProjectError::InvalidDatabase);
    }
    Ok(row)
}

#[cfg(test)]
pub(super) fn check_linked_undo(
    manager: &ProjectManager,
    id: ProjectId,
    ticket: &str,
    new_entity: &str,
    existing_entity: &str,
) {
    let project = manager.open_project(id.to_string()).unwrap();
    let db = project.database().connection();
    let link = kraqel_domain::EntityId::new().to_string();
    db.execute(
        "INSERT INTO relation (id,project_id,source_entity_id,target_entity_id,kind,created_at,updated_at)
         VALUES (?1,?2,?3,?4,'reference','test','test')",
        params![link, id.to_string(), new_entity, existing_entity],
    ).unwrap();
    assert!(manager.switch_handoff_item(id, ticket, true).is_err());
    let preserved: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM relation r JOIN entity e ON r.source_entity_id=e.id
        WHERE r.id=?1 AND r.source_entity_id=?2 AND r.target_entity_id=?3)",
            params![link, new_entity, existing_entity],
            |row| row.get(0),
        )
        .unwrap();
    assert!(preserved);
    assert_eq!(
        db.execute("DELETE FROM relation WHERE id=?1", [link])
            .unwrap(),
        1
    );
}
