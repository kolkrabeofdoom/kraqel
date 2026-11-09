use kraqel_domain::{
    CURRENT_SCHEMA_VERSION, CommandError, CommandRequest, CommandResponse, DocumentKind,
    DocumentNode, EDITOR_DOCUMENT_VERSION, EditorDocument, EntityId, Project, ProjectId,
    is_valid_project_name, require_current_contract,
};
use kraqel_storage::{
    document_repository::{DocumentError, DocumentRepository},
    project_manager::{ProjectError, ProjectManager},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{Manager as _, State};

struct AppState {
    project_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ApplicationStatusRequest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplicationStatusResponse {
    version: &'static str,
    schema_version: u32,
    offline_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ValidateProjectNameRequest {
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct ValidateProjectNameResponse {
    valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateProjectRequest {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRequest {
    project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentRequest {
    project_id: String,
    document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDocumentRequest {
    project_id: String,
    parent_id: Option<String>,
    kind: DocumentKind,
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveDocumentRequest {
    project_id: String,
    document_id: String,
    expected_revision: u64,
    content: EditorDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectOverviewResponse {
    id: ProjectId,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceResponse {
    project: Project,
    documents: Vec<DocumentNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredDocumentResponse {
    node: DocumentNode,
    content: EditorDocument,
    plain_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveDocumentResponse {
    revision: u64,
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri übergibt deserialisierte Command-Argumente im Besitz des Handlers"
)]
fn application_status(
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<ApplicationStatusResponse>, CommandError> {
    require_current_contract(&request)?;
    let ApplicationStatusRequest {} = request.payload;

    Ok(CommandResponse::current(
        request.request_id,
        ApplicationStatusResponse {
            version: env!("CARGO_PKG_VERSION"),
            schema_version: kraqel_domain::CURRENT_SCHEMA_VERSION.0,
            offline_ready: true,
        },
    ))
}

#[tauri::command]
fn validate_project_name(
    request: CommandRequest<ValidateProjectNameRequest>,
) -> Result<CommandResponse<ValidateProjectNameResponse>, CommandError> {
    require_current_contract(&request)?;
    let name = request.payload.name;

    Ok(CommandResponse::current(
        request.request_id,
        ValidateProjectNameResponse {
            valid: is_valid_project_name(&name),
        },
    ))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_projects(
    state: State<'_, AppState>,
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<Vec<ProjectOverviewResponse>>, CommandError> {
    require_current_contract(&request)?;
    let data = list_projects_impl(&state, request.request_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_project(
    state: State<'_, AppState>,
    request: CommandRequest<CreateProjectRequest>,
) -> Result<CommandResponse<WorkspaceResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = create_project_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn open_workspace(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<WorkspaceResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = open_workspace_impl(&state, request.request_id, &request.payload.project_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_document(
    state: State<'_, AppState>,
    request: CommandRequest<CreateDocumentRequest>,
) -> Result<CommandResponse<DocumentNode>, CommandError> {
    require_current_contract(&request)?;
    let data = create_document_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn load_document(
    state: State<'_, AppState>,
    request: CommandRequest<DocumentRequest>,
) -> Result<CommandResponse<StoredDocumentResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = load_document_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn save_document(
    state: State<'_, AppState>,
    request: CommandRequest<SaveDocumentRequest>,
) -> Result<CommandResponse<SaveDocumentResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = save_document_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

fn list_projects_impl(
    state: &AppState,
    request_id: uuid::Uuid,
) -> Result<Vec<ProjectOverviewResponse>, CommandError> {
    manager(state, request_id)?
        .list_projects()
        .map(|projects| {
            projects
                .into_iter()
                .map(|project| ProjectOverviewResponse {
                    id: project.id,
                    name: project.name,
                })
                .collect()
        })
        .map_err(|error| map_project_error(request_id, &error))
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn create_project_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: CreateProjectRequest,
) -> Result<WorkspaceResponse, CommandError> {
    if !is_valid_project_name(&request.name) {
        return Err(CommandError::invalid_request(
            request_id,
            "Der Projektname ist ungültig.",
        ));
    }
    let project = Project {
        id: ProjectId::new(),
        name: request.name.trim().to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .create_project(project.id.to_string(), &project)
        .map_err(|error| map_project_error(request_id, &error))?;
    let mut repository = DocumentRepository::new(opened.database_mut());
    let chapter = repository
        .create(project.id, None, DocumentKind::Chapter, "Kapitel 1")
        .map_err(|error| map_document_error(request_id, &error))?;
    repository
        .create(project.id, Some(chapter.id), DocumentKind::Scene, "Szene 1")
        .map_err(|error| map_document_error(request_id, &error))?;
    let documents = repository
        .list(project.id)
        .map_err(|error| map_document_error(request_id, &error))?;
    Ok(WorkspaceResponse { project, documents })
}

fn open_workspace_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    project_id: &str,
) -> Result<WorkspaceResponse, CommandError> {
    let project_id = parse_project_id(request_id, project_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let name: String = opened
        .database()
        .connection()
        .query_row(
            "SELECT name FROM project WHERE id = ?1",
            [project_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|_| CommandError::internal(request_id))?;
    let repository = DocumentRepository::new(opened.database_mut());
    let documents = repository
        .list(project_id)
        .map_err(|error| map_document_error(request_id, &error))?;
    Ok(WorkspaceResponse {
        project: Project {
            id: project_id,
            name,
            schema_version: CURRENT_SCHEMA_VERSION,
        },
        documents,
    })
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn create_document_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: CreateDocumentRequest,
) -> Result<DocumentNode, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let parent_id = request
        .parent_id
        .as_deref()
        .map(|value| parse_entity_id(request_id, value))
        .transpose()?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    DocumentRepository::new(opened.database_mut())
        .create(project_id, parent_id, request.kind, &request.title)
        .map_err(|error| map_document_error(request_id, &error))
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn load_document_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: DocumentRequest,
) -> Result<StoredDocumentResponse, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let stored = DocumentRepository::new(opened.database_mut())
        .load(project_id, document_id)
        .map_err(|error| map_document_error(request_id, &error))?;
    Ok(StoredDocumentResponse {
        node: stored.node,
        content: stored.content,
        plain_text: stored.plain_text,
    })
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn save_document_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentResponse, CommandError> {
    if request.content.version != EDITOR_DOCUMENT_VERSION {
        return Err(CommandError::invalid_request(
            request_id,
            "Die Editor-Dokumentversion wird nicht unterstützt.",
        ));
    }
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let revision = DocumentRepository::new(opened.database_mut())
        .save_scene(
            project_id,
            document_id,
            request.expected_revision,
            &request.content,
        )
        .map_err(|error| map_document_error(request_id, &error))?;
    Ok(SaveDocumentResponse { revision })
}

fn manager(state: &AppState, request_id: uuid::Uuid) -> Result<ProjectManager, CommandError> {
    ProjectManager::new(&state.project_root).map_err(|_| CommandError::internal(request_id))
}

fn parse_project_id(request_id: uuid::Uuid, value: &str) -> Result<ProjectId, CommandError> {
    ProjectId::parse(value).ok_or_else(|| {
        CommandError::invalid_request(request_id, "Die Projektkennung ist ungültig.")
    })
}

fn parse_entity_id(request_id: uuid::Uuid, value: &str) -> Result<EntityId, CommandError> {
    EntityId::parse(value).ok_or_else(|| {
        CommandError::invalid_request(request_id, "Die Dokumentkennung ist ungültig.")
    })
}

fn map_project_error(request_id: uuid::Uuid, error: &ProjectError) -> CommandError {
    match error {
        ProjectError::ProjectMissing | ProjectError::DatabaseMissing => {
            CommandError::invalid_request(request_id, "Das lokale Projekt wurde nicht gefunden.")
        }
        _ => CommandError::internal(request_id),
    }
}

fn map_document_error(request_id: uuid::Uuid, error: &DocumentError) -> CommandError {
    match error {
        DocumentError::RevisionConflict => CommandError::conflict(request_id),
        DocumentError::InvalidInput
        | DocumentError::ChapterMustBeRoot
        | DocumentError::InvalidParent
        | DocumentError::NotFound => {
            CommandError::invalid_request(request_id, "Die Dokumentanfrage ist ungültig.")
        }
        DocumentError::InvalidStoredData
        | DocumentError::Storage(_)
        | DocumentError::InvalidSystemTime => CommandError::internal(request_id),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Startet die Desktop-Anwendung.
///
/// # Panics
///
/// Bricht ab, wenn die Tauri-Laufzeit nicht initialisiert oder ausgeführt werden kann.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let project_root = app.path().app_local_data_dir()?.join("projects");
            fs::create_dir_all(&project_root)?;
            app.manage(AppState { project_root });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            application_status,
            validate_project_name,
            list_projects,
            create_project,
            open_workspace,
            create_document,
            load_document,
            save_document
        ])
        .run(tauri::generate_context!())
        .expect("KRAQEL konnte nicht gestartet werden");
}

#[cfg(test)]
mod tests {
    use kraqel_domain::{
        COMMAND_CONTRACT_VERSION, CommandErrorCode, CommandRequest, EDITOR_DOCUMENT_VERSION,
        EditorDocument,
    };
    use serde_json::json;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{
        AppState, ApplicationStatusRequest, CreateProjectRequest, DocumentRequest,
        SaveDocumentRequest, ValidateProjectNameRequest, application_status, create_project_impl,
        list_projects_impl, load_document_impl, save_document_impl, validate_project_name,
    };

    const REQUEST_ID: Uuid = Uuid::from_u128(0x018f_47ea_9b2d_7cc1_98e8_b5f4_f6b7_41a2);

    #[test]
    fn status_reports_versioned_offline_foundation() {
        let response = application_status(CommandRequest {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id: REQUEST_ID,
            payload: ApplicationStatusRequest {},
        })
        .expect("current contract should succeed");

        assert_eq!(response.request_id, REQUEST_ID);
        assert_eq!(response.data.version, "0.1.0");
        assert!(response.data.offline_ready);
    }

    #[test]
    fn validation_returns_only_the_result_not_the_supplied_name() {
        let response = validate_project_name(CommandRequest {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id: REQUEST_ID,
            payload: ValidateProjectNameRequest {
                name: "Die linke Hand der Dunkelheit".to_owned(),
            },
        })
        .expect("valid request should succeed");
        let json = serde_json::to_string(&response).expect("response should serialize");

        assert!(response.data.valid);
        assert!(!json.contains("Dunkelheit"));
    }

    #[test]
    fn unsupported_contract_returns_sanitized_error() {
        let error = application_status(CommandRequest {
            contract_version: 0,
            request_id: REQUEST_ID,
            payload: ApplicationStatusRequest {},
        })
        .expect_err("old contract should fail");

        assert_eq!(error.code, CommandErrorCode::UnsupportedContractVersion);
        assert!(!error.retryable);
    }

    #[test]
    fn project_scene_roundtrip_uses_versioned_commands_and_revisions() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
        };
        let workspace = create_project_impl(
            &state,
            REQUEST_ID,
            CreateProjectRequest {
                name: "Morgenrot".to_owned(),
            },
        )
        .expect("project should be created");
        let scene = workspace
            .documents
            .iter()
            .find(|document| document.kind == kraqel_domain::DocumentKind::Scene)
            .expect("initial scene should exist");
        let content = EditorDocument {
            version: EDITOR_DOCUMENT_VERSION,
            document: json!({
                "type": "doc",
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Hallo Welt"}]}]
            }),
        };

        let saved = save_document_impl(
            &state,
            REQUEST_ID,
            SaveDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: 0,
                content: content.clone(),
            },
        )
        .expect("scene should save");
        assert_eq!(saved.revision, 1);

        let loaded = load_document_impl(
            &state,
            REQUEST_ID,
            DocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
            },
        )
        .expect("scene should load");
        assert_eq!(loaded.content, content);
        assert_eq!(loaded.plain_text, "Hallo Welt");
        assert_eq!(
            list_projects_impl(&state, REQUEST_ID)
                .expect("projects should list")
                .len(),
            1
        );

        let conflict = save_document_impl(
            &state,
            REQUEST_ID,
            SaveDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: 0,
                content,
            },
        )
        .expect_err("stale revision should conflict");
        assert_eq!(conflict.code, CommandErrorCode::Conflict);
    }
}
