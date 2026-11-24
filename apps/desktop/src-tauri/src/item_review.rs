//! Desktop IPC for explicit, row-bound native comparisons. Renderer replies contain only receipts.
use super::*;
use kraqel_storage::project_manager::{
    ItemChange, ItemKind, ItemSelection, ReviewChoice, ReviewOutcome,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ListRequest {
    project_id: String,
    folder: String,
    manifest: String,
    kind: ItemKind,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReviewRequest {
    project_id: String,
    selection: ItemSelection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SecureRequest {
    project_id: String,
    selection: ItemSelection,
    candidate: PreparedHandoff,
}

#[tauri::command]
pub(super) async fn list_handoff_items(
    state: State<'_, AppState>,
    request: CommandRequest<ListRequest>,
) -> Result<CommandResponse<Vec<ItemChange>>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let p = request.payload;
    let id = parse_project_id(request_id, &p.project_id)?;
    let manager = manager(&state, request_id)?;
    let changes = tauri::async_runtime::spawn_blocking(move || {
        manager.list_handoff_items(id, &p.folder, &p.manifest, p.kind)
    })
    .await
    .map_err(|_| CommandError::internal(request_id))?
    .map_err(|e| map_project_error(request_id, &e))?;
    Ok(CommandResponse::current(request_id, changes))
}

#[tauri::command]
pub(super) async fn missing_handoff_parent(
    state: State<'_, AppState>,
    request: CommandRequest<ReviewRequest>,
) -> Result<CommandResponse<Option<kraqel_storage::project_manager::ItemKey>>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let payload = request.payload;
    let id = parse_project_id(request_id, &payload.project_id)?;
    let manager = manager(&state, request_id)?;
    let parent = tauri::async_runtime::spawn_blocking(move || {
        manager.missing_handoff_parent(id, &payload.selection)
    })
    .await
    .map_err(|_| CommandError::internal(request_id))?
    .map_err(|e| map_project_error(request_id, &e))?;
    Ok(CommandResponse::current(request_id, parent))
}

#[tauri::command]
pub(super) async fn review_handoff_item(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CommandRequest<ReviewRequest>,
) -> Result<CommandResponse<ReviewOutcome>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let p = request.payload;
    let id = parse_project_id(request_id, &p.project_id)?;
    let manager = manager(&state, request_id)?;
    let home = state
        .project_root
        .parent()
        .ok_or_else(|| CommandError::internal(request_id))?
        .to_owned();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let prompt = app
            .dialog()
            .message(concat!(
                "Den ausgewählten Datensatz mit allen Feldern im isolierten Windows-Fenster anzeigen? ",
                "Die bestätigte externe Wahl bereitet eine separate Arbeitskopie vor."
            ))
            .title("Datensatzvergleich öffnen")
            .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancel);
        if !prompt.blocking_show() {
            return Ok(ReviewOutcome::Cancel);
        }
        let (local, incoming) = manager.review_handoff_item(id, &p.selection)?;
        let choice = kraqel_isolation_transport::open_item_review(
            &home,
            id,
            &p.selection,
            local,
            incoming,
            [
                include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/pipe-service.exe")),
                include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/native-leaf.exe")),
            ],
        )
        .map_err(|_| ProjectError::InvalidDatabase)?;
        match choice {
            ReviewChoice::Cancel => Ok(ReviewOutcome::Cancel),
            ReviewChoice::Local => Ok(ReviewOutcome::Local),
            ReviewChoice::Incoming => Ok(ReviewOutcome::Incoming {
                candidate: manager.prepare_handoff_item(id, &p.selection)?,
            }),
        }
    })
    .await
    .map_err(|_| CommandError::internal(request_id))?
    .map_err(|e| map_project_error(request_id, &e))?;
    Ok(CommandResponse::current(request_id, outcome))
}

#[tauri::command]
pub(super) async fn secure_handoff_item(
    state: State<'_, AppState>,
    request: CommandRequest<SecureRequest>,
) -> Result<CommandResponse<String>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let p = request.payload;
    let id = parse_project_id(request_id, &p.project_id)?;
    let manager = manager(&state, request_id)?;
    let ticket = tauri::async_runtime::spawn_blocking(move || {
        manager.secure_handoff_item(id, &p.selection, &p.candidate)
    })
    .await
    .map_err(|_| CommandError::internal(request_id))?
    .map_err(|e| map_project_error(request_id, &e))?;
    Ok(CommandResponse::current(request_id, ticket))
}

#[tauri::command]
pub(super) async fn switch_handoff_item(
    state: State<'_, AppState>,
    request: CommandRequest<SwitchActivationRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let p = request.payload;
    let id = parse_project_id(request_id, &p.project_id)?;
    let manager = manager(&state, request_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        manager.switch_handoff_item(id, &p.ticket, p.undo)
    })
    .await
    .map_err(|_| CommandError::internal(request_id))?
    .map_err(|e| map_project_error(request_id, &e))?;
    Ok(CommandResponse::current(request_id, ()))
}

/// The existing desktop scene test covers the live/local pair. This one exercises the
/// composite-key item binding and the missing local side at the actual role boundary.
#[cfg(test)]
#[test]
#[ignore = "interactive Windows desktop; choose the external item"]
fn isolated_incoming_link_choice() {
    use kraqel_domain::EntityId;
    use kraqel_storage::project_manager::ItemKey;
    let home = tempfile::tempdir().unwrap();
    let project = kraqel_domain::ProjectId::new();
    let ids = vec![EntityId::new().to_string(), EntityId::new().to_string()];
    let selection = ItemSelection {
        folder: format!("uebernahme-{}", uuid::Uuid::new_v4()),
        manifest: "a".repeat(64),
        change: ItemChange {
            key: ItemKey {
                kind: ItemKind::EventEntityLink,
                ids,
            },
            local: None,
            incoming: Some("b".repeat(64)),
        },
    };
    let choice = kraqel_isolation_transport::open_item_review(
        home.path(),
        project,
        &selection,
        None,
        "Synthetische Ereigniszuordnung".into(),
        [
            include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/pipe-service.exe")),
            include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/native-leaf.exe")),
        ],
    )
    .unwrap();
    assert!(matches!(choice, ReviewChoice::Incoming));
    assert!(
        !home
            .path()
            .join("KRAQEL-Manuskriptvergleich/active")
            .exists()
    );
}
