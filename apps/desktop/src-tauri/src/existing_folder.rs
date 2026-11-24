//! Explicit, one-use import of a closed Windows project folder. Paths stay native.
use super::*;
use kraqel_storage::project_manager::{
    ExistingExtraCounts, ExistingExtraGroup, ExistingFolderGuard, ExistingFolderPreview,
    pin_existing_folder_path, safe_existing_name,
};
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[derive(Default)]
pub(super) struct FolderState(Mutex<Option<Pending>>);

struct Pending {
    token: Uuid,
    source: PathBuf,
    preview: ExistingFolderPreview,
    created: Instant,
    guard: ExistingFolderGuard,
}

#[derive(Serialize)]
pub(super) struct Preview {
    token: Uuid,
    documents: u64,
    images: u64,
    extras: ExistingExtraCounts,
}

fn group_choice(group: ExistingExtraGroup, available: &ExistingExtraCounts) -> (&'static str, u64) {
    match group {
        ExistingExtraGroup::Backups => ("Sicherungen", available.backups),
        ExistingExtraGroup::Exports => ("Exporte", available.exports),
        ExistingExtraGroup::Git => ("Git-Historie", available.git),
        ExistingExtraGroup::Other => ("Weitere Dateien", available.other),
    }
}

const PREVIEW_LIFETIME: Duration = Duration::from_secs(600);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Cancel {
    token: Uuid,
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri supplies owned IPC arguments."
)]
pub(super) fn cancel_existing_folder(
    session: State<'_, FolderState>,
    request: CommandRequest<Cancel>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let id = request.request_id;
    session
        .0
        .lock()
        .map_err(|_| CommandError::internal(id))?
        .take_if(|p| p.token == request.payload.token);
    Ok(CommandResponse::current(id, ()))
}

#[tauri::command]
pub(super) async fn preview_existing_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    session: State<'_, FolderState>,
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<Option<Preview>>, CommandError> {
    require_current_contract(&request)?;
    let id = request.request_id;
    session
        .0
        .lock()
        .map_err(|_| CommandError::internal(id))?
        .take();
    let Some(folder) = app
        .dialog()
        .file()
        .set_title("Geschlossenen KRAQEL-Projektordner auswählen")
        .blocking_pick_folder()
    else {
        return Ok(CommandResponse::current(id, None));
    };
    let source = folder.into_path().map_err(|_| CommandError::internal(id))?;
    let manager = manager(&state, id)?;
    let pending = tauri::async_runtime::spawn_blocking(move || {
        let (preview, guard) = manager.preview_existing_folder(&source)?;
        Ok::<_, ProjectError>(Pending {
            token: Uuid::new_v4(),
            source,
            preview,
            guard,
            created: Instant::now(),
        })
    })
    .await
    .map_err(|_| CommandError::internal(id))?
    .map_err(|e| map_project_error(id, &e))?;
    let token = pending.token;
    let result = Preview {
        token: pending.token,
        documents: pending.preview.documents,
        images: pending.preview.images,
        extras: pending.preview.extras.clone(),
    };
    *session.0.lock().map_err(|_| CommandError::internal(id))? = Some(pending);
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(PREVIEW_LIFETIME).await;
        if let Some(session) = app.try_state::<FolderState>()
            && let Ok(mut slot) = session.0.lock()
        {
            slot.take_if(|p| p.token == token);
        }
    });
    Ok(CommandResponse::current(id, Some(result)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receive {
    token: Uuid,
    groups: Vec<ExistingExtraGroup>,
}

impl Receive {
    fn valid_groups(&self, available: &ExistingExtraCounts) -> bool {
        self.groups.len() <= 4
            && self.groups.iter().enumerate().all(|(i, group)| {
                !self.groups[..i].contains(group) && group_choice(*group, available).1 > 0
            })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum ImportOutcome {
    Received,
    ArchiveFailed,
    Cancel,
}

struct ArchiveTarget {
    destination: PathBuf,
    _pins: Vec<fs::File>,
    temp: NamedTempFile,
}

fn identity(file: &fs::File) -> Result<(u64, u64), ProjectError> {
    let info =
        winapi_util::file::information(file).map_err(|_| ProjectError::InvalidRelativePath)?;
    Ok((info.volume_serial_number(), info.file_index()))
}

fn archive_target(
    destination: PathBuf,
    source: &Path,
    catalog: &Path,
) -> Result<ArchiveTarget, ProjectError> {
    let invalid = || ProjectError::InvalidRelativePath;
    let name = safe_existing_name(destination.file_name().ok_or_else(invalid)?)?;
    if !name.to_ascii_lowercase().ends_with(".age") {
        return Err(invalid());
    }
    let parent = destination.parent().ok_or_else(invalid)?;
    let pins = pin_existing_folder_path(parent)?;
    let ancestors = pins.iter().map(identity).collect::<Result<Vec<_>, _>>()?;
    for excluded in [source, catalog] {
        let dirs = pin_existing_folder_path(excluded)?;
        if ancestors.contains(&identity(dirs.last().ok_or_else(invalid)?)?) {
            return Err(invalid());
        }
    }
    if !matches!(fs::symlink_metadata(&destination), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
    {
        return Err(invalid());
    }
    let temp = NamedTempFile::new_in(parent).map_err(|_| invalid())?;
    Ok(ArchiveTarget {
        destination,
        temp,
        _pins: pins,
    })
}

type ArchiveChoice = (ArchiveTarget, Option<kraqel_archiv::SecretString>);

fn choose_archive(
    app: &AppHandle,
    owner: isize,
    source: &Path,
    catalog: &Path,
) -> Result<Option<ArchiveChoice>, ProjectError> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Neues verschlüsseltes Zusatzarchiv")
        .set_file_name("KRAQEL-Zusatzarchiv.zip.age")
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let destination = file
        .into_path()
        .map_err(|_| ProjectError::InvalidRelativePath)?;
    let target = archive_target(destination, source, catalog)?;
    let Some(mut passphrase) = kraqel_isolation_transport::prompt_archive_passphrase(owner)
        .map_err(|_| ProjectError::InvalidImportedData)?
    else {
        return Ok(None);
    };
    Ok(Some((
        target,
        Some(kraqel_archiv::SecretString::from(std::mem::take(
            &mut *passphrase,
        ))),
    )))
}

#[tauri::command]
pub(super) async fn receive_existing_folder(
    app: AppHandle,
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    session: State<'_, FolderState>,
    request: CommandRequest<Receive>,
) -> Result<CommandResponse<ImportOutcome>, CommandError> {
    require_current_contract(&request)?;
    let id = request.request_id;
    let input = request.payload;
    let pending = session
        .0
        .lock()
        .map_err(|_| CommandError::internal(id))?
        .take_if(|p| p.token == input.token)
        .ok_or_else(|| CommandError::invalid_request(id, "Vorschau erneut öffnen."))?;
    if pending.created.elapsed() > PREVIEW_LIFETIME || !input.valid_groups(&pending.preview.extras)
    {
        return Err(CommandError::invalid_request(id, "Vorschau erneut laden."));
    }
    let owner = window.hwnd().map_err(|_| CommandError::internal(id))?.0 as isize;
    let manager = manager(&state, id)?;
    let catalog = state.project_root.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut message =
            String::from("Die geprüften Arbeitsdaten als separates lokales Projekt anlegen?");
        if !input.groups.is_empty() {
            message.push_str("\nSeparates verschlüsseltes Archiv für folgende Auswahl: ");
            message.push_str(
                &input
                    .groups
                    .iter()
                    .map(|g| group_choice(*g, &pending.preview.extras).0)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if !app
            .dialog()
            .message(message)
            .title("Projektübernahme bestätigen")
            .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancel)
            .blocking_show()
        {
            return Ok(ImportOutcome::Cancel);
        }
        let mut archive = if input.groups.is_empty() {
            None
        } else if let Some(archive) = choose_archive(&app, owner, &pending.source, &catalog)? {
            Some(archive)
        } else {
            return Ok(ImportOutcome::Cancel);
        };
        manager.receive_existing_folder(
            &pending.source,
            &pending.preview.source_sha256,
            pending.guard,
            |files| {
                if let Some((target, secret)) = archive.as_mut() {
                    let mut builder = kraqel_archiv::SupplementBuilder::new();
                    for file in files.iter().filter(|f| input.groups.contains(&f.group())) {
                        let name = file
                            .relative_path()
                            .to_str()
                            .ok_or(ProjectError::InvalidRelativePath)?
                            .replace('\\', "/");
                        builder
                            .push(&name, file.open()?)
                            .map_err(|_| ProjectError::InvalidImportedData)?;
                    }
                    let encrypted = builder
                        .seal(secret.take().ok_or(ProjectError::InvalidImportedData)?)
                        .map_err(|_| ProjectError::InvalidImportedData)?;
                    target
                        .temp
                        .write_all(&encrypted)
                        .and_then(|()| target.temp.as_file().sync_all())
                        .map_err(|_| ProjectError::InvalidImportedData)?;
                }
                Ok(())
            },
        )?;
        if let Some((target, _)) = archive
            && target.temp.persist_noclobber(target.destination).is_err()
        {
            return Ok(ImportOutcome::ArchiveFailed);
        }
        Ok::<_, ProjectError>(ImportOutcome::Received)
    })
    .await
    .map_err(|_| CommandError::internal(id))?
    .map_err(|e| {
        if matches!(e, ProjectError::ProjectAlreadyExists) {
            CommandError::conflict(id)
        } else {
            map_project_error(id, &e)
        }
    })?;
    Ok(CommandResponse::current(id, result))
}
