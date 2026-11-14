//! Consistent project reads and serialized scene-image export orchestration.
use crate::{AppState, manager, parse_project_id};
use kraqel_domain::{CURRENT_SCHEMA_VERSION, CommandError, Project, ProjectId};
use kraqel_export::{
    ExportSourceDocument,
    illustrated::{self, IllustratedError, IllustratedFormat, IllustratedResult, ImagePlacement},
};
use kraqel_storage::{document_repository::DocumentRepository, media_repository::MediaRepository};
use serde::Deserialize;
use std::{collections::BTreeSet, sync::Mutex};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct IllustratedRequest {
    pub project_id: String,
    pub format: IllustratedFormat,
    pub placements: Vec<ImagePlacement>,
}
static ACTIVE: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

#[cfg(test)]
mod tests;
struct Active(String);
impl Drop for Active {
    fn drop(&mut self) {
        if let Ok(mut jobs) = ACTIVE.lock() {
            jobs.remove(&self.0);
        }
    }
}
struct Frozen {
    project: Project,
    documents: Vec<ExportSourceDocument>,
    assets: Vec<kraqel_domain::MediaAsset>,
}
impl Frozen {
    fn fingerprint(&self) -> serde_json::Value {
        serde_json::json!({"project":self.project,"documents":self.documents.iter().map(|d|serde_json::json!({"node":d.node,"content":d.content})).collect::<Vec<_>>(),"assets":self.assets})
    }
}
fn frozen(
    state: &AppState,
    id: ProjectId,
    request_id: uuid::Uuid,
    selection: &[ImagePlacement],
) -> Result<(Frozen, std::path::PathBuf), IllustratedError> {
    let manager = manager(state, request_id).map_err(|_| IllustratedError::Internal)?;
    let mut opened = manager
        .open_project(id.to_string())
        .map_err(|_| IllustratedError::Invalid)?;
    let directory = opened.directory().to_path_buf();
    opened
        .database()
        .connection()
        .execute_batch("BEGIN DEFERRED TRANSACTION")
        .map_err(|_| IllustratedError::Internal)?;
    let result = (|| {
        let database = opened.database_mut();
        let size:i64=database.connection().query_row("SELECT COALESCE(SUM(length(CAST(content_json AS BLOB))),0) FROM document WHERE project_id=?1",[id.to_string()],|r|r.get(0)).map_err(|_|IllustratedError::Internal)?;
        if size > 32 * 1024 * 1024 {
            return Err(IllustratedError::Invalid);
        }
        let name = database
            .connection()
            .query_row(
                "SELECT name FROM project WHERE id=?1",
                [id.to_string()],
                |r| r.get(0),
            )
            .map_err(|_| IllustratedError::Invalid)?;
        let repository = DocumentRepository::new(database);
        let nodes = repository.list(id).map_err(|_| IllustratedError::Invalid)?;
        if nodes.len() > 10000 {
            return Err(IllustratedError::Invalid);
        }
        let documents = nodes
            .into_iter()
            .map(|node| {
                let stored = repository
                    .load(id, node.id)
                    .map_err(|_| IllustratedError::Invalid)?;
                Ok(ExportSourceDocument {
                    node: stored.node,
                    content: stored.content,
                })
            })
            .collect::<Result<Vec<_>, IllustratedError>>()?;
        let repository = MediaRepository::new(database);
        let ids: BTreeSet<_> = selection.iter().map(|s| s.asset_id).collect();
        if ids.len() > 50 {
            return Err(IllustratedError::Invalid);
        }
        let assets = ids
            .into_iter()
            .map(|asset| {
                repository
                    .load(id, asset)
                    .map_err(|_| IllustratedError::Invalid)
            })
            .collect::<Result<_, _>>()?;
        Ok(Frozen {
            project: Project {
                id,
                name,
                schema_version: CURRENT_SCHEMA_VERSION,
            },
            documents,
            assets,
        })
    })();
    opened
        .database()
        .connection()
        .execute_batch("ROLLBACK")
        .map_err(|_| IllustratedError::Internal)?;
    result.map(|f| (f, directory))
}
pub(super) fn export(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &IllustratedRequest,
) -> Result<IllustratedResult, CommandError> {
    let id = parse_project_id(request_id, &request.project_id)?;
    let run = || {
        if request.placements.is_empty() || request.placements.len() > 200 {
            return Err(IllustratedError::Invalid);
        }
        if !ACTIVE
            .lock()
            .map_err(|_| IllustratedError::Internal)?
            .insert(id.to_string())
        {
            return Err(IllustratedError::Conflict);
        }
        let _active = Active(id.to_string());
        let (snapshot, directory) = frozen(state, id, request_id, &request.placements)?;
        let fingerprint = snapshot.fingerprint();
        let package = illustrated::prepare(
            &directory,
            &snapshot.project,
            &snapshot.documents,
            &snapshot.assets,
            request.format,
            &request.placements,
        )?;
        publish_checked(
            state,
            id,
            request_id,
            &request.placements,
            &directory,
            &package,
            &fingerprint,
        )
    };
    run().map_err(|error| match error {
        IllustratedError::Invalid => CommandError::invalid_request(
            request_id,
            "Die Bildauswahl oder Quelldatei ist ungültig oder überschreitet die Exportgrenzen.",
        ),
        IllustratedError::Conflict => CommandError::conflict(request_id),
        IllustratedError::Internal => CommandError::internal(request_id),
    })
}

fn publish_checked(
    state: &AppState,
    id: ProjectId,
    request_id: uuid::Uuid,
    selection: &[ImagePlacement],
    directory: &std::path::Path,
    package: &illustrated::PreparedPackage,
    fingerprint: &serde_json::Value,
) -> Result<IllustratedResult, IllustratedError> {
    let mut publication_lock = None;
    let result = illustrated::publish(directory, package, || {
        let manager = manager(state, request_id).map_err(|_| IllustratedError::Internal)?;
        let lock = manager
            .open_project(id.to_string())
            .map_err(|_| IllustratedError::Conflict)?;
        lock.database()
            .connection()
            .execute_batch("BEGIN IMMEDIATE TRANSACTION")
            .map_err(|_| IllustratedError::Conflict)?;
        let (current, _) =
            frozen(state, id, request_id, selection).map_err(|_| IllustratedError::Conflict)?;
        if current.fingerprint() != *fingerprint {
            return Err(IllustratedError::Conflict);
        }
        publication_lock = Some(lock);
        Ok(())
    });
    drop(publication_lock);
    result
}
