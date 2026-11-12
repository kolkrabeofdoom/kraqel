use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use kraqel_ai_service::{
    CloudProviderId, CloudTextProvider, CredentialError, CredentialStore, OllamaProvider,
    OsCredentialStore, ReleasedTextContext, TextGenerationRequest, TextModel, TextProvider,
    TextProviderDescriptor, TextProviderError, TextStreamEvent,
};
use kraqel_asset_service::{
    AssetError, prepare_image, read_thumbnail, remove_asset_files, store_image, verify_original,
};
use kraqel_continuity::{Finding, SuggestionReview, review_suggestion};
use kraqel_domain::{
    AssetTargetKind, CURRENT_SCHEMA_VERSION, CommandError, CommandRequest, CommandResponse,
    DocumentKind, DocumentNode, EDITOR_DOCUMENT_VERSION, EditorDocument, EntityId, EntityKind,
    ImagePromptComposition, ImagePromptDraft, MediaAsset, PlotBoard, PlotStatus, Project,
    ProjectId, StoryEntity, StoryEntityDetails, StoryRelation, Storyline, StylePreset, Timeline,
    TimelineDetails, TimelineEvent, TimelineMode, VisualProfile, is_valid_project_name,
    require_current_contract,
};
use kraqel_export::{
    ExportError, ExportFormat, ExportResult, ExportSourceDocument, PromptExportResult,
    export_project, export_prompt_draft,
};
use kraqel_git_service::{
    CommitResult, GitService, GitServiceError, SnapshotSummary, VersionControlStatus,
    import_snapshot_archive,
};
use kraqel_storage::{
    continuity_repository::{ContinuityError, ContinuityRepository},
    document_repository::{DocumentError, DocumentRepository},
    media_repository::{MediaError, MediaRepository},
    project_manager::{ProjectError, ProjectManager},
    prompt_repository::{PromptError, PromptRepository},
    story_bible_repository::{StoryBibleError, StoryBibleRepository},
    structure_repository::{StructureError, StructureRepository},
    visual_repository::{VisualError, VisualRepository},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
    process::Command,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Manager as _, State, ipc::Channel};
use zeroize::Zeroizing;

const AI_RELEASE_LIFETIME: Duration = Duration::from_secs(10 * 60);
const EXAMPLE_PROJECT_ARCHIVE: &[u8] =
    include_bytes!("../../../../examples/die-stadt-die-den-dienstag-verkaufte.kraqel");

struct AppState {
    project_root: PathBuf,
    ai_releases: Mutex<HashMap<uuid::Uuid, AiReleaseRecord>>,
}

#[derive(Debug, Clone)]
struct AiReleaseRecord {
    project_id: String,
    document_id: String,
    revision: u64,
    provider_id: String,
    model: String,
    instruction: String,
    created_at: Instant,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameDocumentRequest {
    project_id: String,
    document_id: String,
    expected_title: String,
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderDocumentsRequest {
    project_id: String,
    parent_id: Option<String>,
    expected_order: Vec<String>,
    ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteDocumentRequest {
    project_id: String,
    document_id: String,
    expected_title: String,
    expected_revision: u64,
    expected_child_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportProjectRequest {
    project_id: String,
    format: ExportFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiContextRequest {
    project_id: String,
    document_id: String,
    provider_id: String,
    model: String,
    instruction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiContextPreview {
    document_id: EntityId,
    document_title: String,
    revision: u64,
    text: String,
    provider_id: String,
    provider_display_name: String,
    provider_location: String,
    model: String,
    instruction: String,
    metadata: BTreeMap<String, String>,
    release_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamTextDraftRequest {
    project_id: String,
    document_id: String,
    expected_revision: u64,
    model: String,
    instruction: String,
    context_approved: bool,
    provider_id: String,
    release_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewTextSuggestionRequest {
    project_id: String,
    document_id: String,
    expected_revision: u64,
    proposed_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OllamaModelsResponse {
    provider: TextProviderDescriptor,
    available: bool,
    models: Vec<TextModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudProviderStatus {
    provider: TextProviderDescriptor,
    credential_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudProviderRequest {
    provider_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveCloudCredentialRequest {
    provider_id: String,
    api_key: String,
}

impl std::fmt::Debug for SaveCloudCredentialRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SaveCloudCredentialRequest")
            .field("provider_id", &self.provider_id)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteDocumentResponse {
    deleted_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListStoryEntitiesRequest {
    project_id: String,
    query: Option<String>,
    kind: Option<EntityKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoryEntityRequest {
    project_id: String,
    entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "Die expliziten Bezeichner bilden den stabilen Tauri-Requestvertrag ab"
)]
#[serde(rename_all = "camelCase")]
struct MergeStoryEntitiesRequest {
    project_id: String,
    source_entity_id: String,
    target_entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteStoryEntityRequest {
    project_id: String,
    entity_id: Option<String>,
    kind: EntityKind,
    name: String,
    properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateStoryRelationRequest {
    project_id: String,
    source_entity_id: String,
    target_entity_id: String,
    kind: String,
    properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteStoryRelationRequest {
    project_id: String,
    relation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_field_names,
    reason = "Feldnamen spiegeln den öffentlichen IPC-Vertrag"
)]
struct LinkStoryEntityRequest {
    project_id: String,
    document_id: String,
    entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorylineWriteRequest {
    project_id: String,
    name: String,
    color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorylineDeleteRequest {
    project_id: String,
    storyline_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlotCardWriteRequest {
    project_id: String,
    document_id: String,
    status: PlotStatus,
    storyline_id: Option<String>,
    pov_entity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderRequest {
    project_id: String,
    parent_id: Option<String>,
    ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineWriteRequest {
    project_id: String,
    name: String,
    mode: TimelineMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineRequest {
    project_id: String,
    timeline_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventWriteRequest {
    project_id: String,
    timeline_id: String,
    title: String,
    description: String,
    time_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventRequest {
    project_id: String,
    event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_field_names,
    reason = "Feldnamen spiegeln den öffentlichen IPC-Vertrag"
)]
struct EventLinkRequest {
    project_id: String,
    timeline_id: String,
    event_id: String,
    document_id: Option<String>,
    entity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportMediaRequest {
    project_id: String,
    file_name: String,
    title: String,
    tags: Vec<String>,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaAssetRequest {
    project_id: String,
    asset_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMediaRequest {
    project_id: String,
    asset_id: String,
    title: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplaceMediaRequest {
    project_id: String,
    asset_id: String,
    file_name: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetLinkRequest {
    project_id: String,
    asset_id: String,
    target_kind: AssetTargetKind,
    target_id: String,
    role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaAssetView {
    asset: MediaAsset,
    thumbnail_data_url: Option<String>,
    file_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteVisualProfileRequest {
    project_id: String,
    entity_id: String,
    traits: BTreeMap<String, String>,
    canonical_asset_id: Option<String>,
    exclusions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteStylePresetRequest {
    project_id: String,
    preset_id: Option<String>,
    name: String,
    visual_rules: BTreeMap<String, String>,
    applicable_kinds: Vec<EntityKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StylePresetRequest {
    project_id: String,
    preset_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComposeImagePromptRequest {
    project_id: String,
    document_id: String,
    entity_ids: Vec<String>,
    style_preset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateImagePromptDraftRequest {
    project_id: String,
    name: String,
    composition: ImagePromptComposition,
    prompt_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateImagePromptDraftRequest {
    project_id: String,
    draft_id: String,
    name: String,
    prompt_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImagePromptDraftRequest {
    project_id: String,
    draft_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitSnapshotRequest {
    project_id: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionControlDiffResponse {
    diff: String,
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
fn create_example_project(
    state: State<'_, AppState>,
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<WorkspaceResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = create_example_project_impl(&state, request.request_id)?;
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

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn rename_document(
    state: State<'_, AppState>,
    request: CommandRequest<RenameDocumentRequest>,
) -> Result<CommandResponse<DocumentNode>, CommandError> {
    require_current_contract(&request)?;
    let data = rename_document_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn reorder_documents(
    state: State<'_, AppState>,
    request: CommandRequest<ReorderDocumentsRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    reorder_documents_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_document(
    state: State<'_, AppState>,
    request: CommandRequest<DeleteDocumentRequest>,
) -> Result<CommandResponse<DeleteDocumentResponse>, CommandError> {
    require_current_contract(&request)?;
    let data = delete_document_impl(&state, request.request_id, request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn export_manuscript(
    state: State<'_, AppState>,
    request: CommandRequest<ExportProjectRequest>,
) -> Result<CommandResponse<ExportResult>, CommandError> {
    require_current_contract(&request)?;
    let data = export_manuscript_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn open_export_directory(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let directory =
        resolve_export_directory(&state, request.request_id, &request.payload.project_id)?;
    open_directory(&directory).map_err(|_| CommandError::internal(request.request_id))?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
async fn list_ollama_models(
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<OllamaModelsResponse>, CommandError> {
    require_current_contract(&request)?;
    let ApplicationStatusRequest {} = request.payload;
    let provider = OllamaProvider::local_default()
        .map_err(|error| map_text_provider_error(request.request_id, &error))?;
    let descriptor = provider.descriptor();
    let (available, models) = match provider.list_models().await {
        Ok(models) => (true, models),
        Err(TextProviderError::Unavailable) => (false, Vec::new()),
        Err(error) => return Err(map_text_provider_error(request.request_id, &error)),
    };
    Ok(CommandResponse::current(
        request.request_id,
        OllamaModelsResponse {
            provider: descriptor,
            available,
            models,
        },
    ))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_cloud_provider_statuses(
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<Vec<CloudProviderStatus>>, CommandError> {
    require_current_contract(&request)?;
    let ApplicationStatusRequest {} = request.payload;
    let store = OsCredentialStore;
    let statuses = CloudProviderId::ALL
        .into_iter()
        .map(|provider| {
            Ok(CloudProviderStatus {
                provider: provider.descriptor(),
                credential_configured: store
                    .contains(provider)
                    .map_err(|error| map_credential_error(request.request_id, error))?,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(CommandResponse::current(request.request_id, statuses))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn save_cloud_credential(
    request: CommandRequest<SaveCloudCredentialRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let request_id = request.request_id;
    let provider = parse_cloud_provider(request_id, &request.payload.provider_id)?;
    let api_key = Zeroizing::new(request.payload.api_key);
    OsCredentialStore
        .set(provider, &api_key)
        .map_err(|error| map_credential_error(request_id, error))?;
    Ok(CommandResponse::current(request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_cloud_credential(
    request: CommandRequest<CloudProviderRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let provider = parse_cloud_provider(request.request_id, &request.payload.provider_id)?;
    OsCredentialStore
        .delete(provider)
        .map_err(|error| map_credential_error(request.request_id, error))?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
async fn list_cloud_models(
    request: CommandRequest<CloudProviderRequest>,
) -> Result<CommandResponse<Vec<TextModel>>, CommandError> {
    require_current_contract(&request)?;
    let provider_id = parse_cloud_provider(request.request_id, &request.payload.provider_id)?;
    let api_key = OsCredentialStore
        .get(provider_id)
        .map_err(|error| map_credential_error(request.request_id, error))?;
    let provider = CloudTextProvider::production(provider_id, api_key)
        .map_err(|error| map_text_provider_error(request.request_id, &error))?;
    let models = provider
        .list_models()
        .await
        .map_err(|error| map_text_provider_error(request.request_id, &error))?;
    Ok(CommandResponse::current(request.request_id, models))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn preview_ai_context(
    state: State<'_, AppState>,
    request: CommandRequest<AiContextRequest>,
) -> Result<CommandResponse<AiContextPreview>, CommandError> {
    require_current_contract(&request)?;
    let preview = preview_ai_context_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, preview))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
async fn stream_text_draft(
    state: State<'_, AppState>,
    request: CommandRequest<StreamTextDraftRequest>,
    on_event: Channel<TextStreamEvent>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let generation = prepare_text_generation(&state, request.request_id, &request.payload)?;
    let mut send = |event| {
        on_event
            .send(event)
            .map_err(|_| TextProviderError::StreamClosed)
    };
    if request.payload.provider_id == kraqel_ai_service::OLLAMA_PROVIDER_ID {
        OllamaProvider::local_default()
            .map_err(|error| map_text_provider_error(request.request_id, &error))?
            .stream_draft(&generation, &mut send)
            .await
            .map_err(|error| map_text_provider_error(request.request_id, &error))?;
    } else {
        let provider_id = parse_cloud_provider(request.request_id, &request.payload.provider_id)?;
        let api_key = OsCredentialStore
            .get(provider_id)
            .map_err(|error| map_credential_error(request.request_id, error))?;
        CloudTextProvider::production(provider_id, api_key)
            .map_err(|error| map_text_provider_error(request.request_id, &error))?
            .stream_draft(&generation, &mut send)
            .await
            .map_err(|error| map_text_provider_error(request.request_id, &error))?;
    }
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn analyze_continuity(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<Finding>>, CommandError> {
    require_current_contract(&request)?;
    let project_id = parse_project_id(request.request_id, &request.payload.project_id)?;
    let manager = manager(&state, request.request_id)?;
    let opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request.request_id, &error))?;
    let findings = ContinuityRepository::new(opened.database())
        .analyze(project_id)
        .map_err(|error| map_continuity_error(request.request_id, &error))?;
    Ok(CommandResponse::current(request.request_id, findings))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn review_text_suggestion(
    state: State<'_, AppState>,
    request: CommandRequest<ReviewTextSuggestionRequest>,
) -> Result<CommandResponse<SuggestionReview>, CommandError> {
    require_current_contract(&request)?;
    let review = review_text_suggestion_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, review))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_media_assets(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<MediaAssetView>>, CommandError> {
    require_current_contract(&request)?;
    let data = list_media_assets_impl(&state, request.request_id, &request.payload.project_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn import_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<ImportMediaRequest>,
) -> Result<CommandResponse<MediaAssetView>, CommandError> {
    require_current_contract(&request)?;
    let data = import_media_asset_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn update_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<UpdateMediaRequest>,
) -> Result<CommandResponse<MediaAssetView>, CommandError> {
    require_current_contract(&request)?;
    let data = update_media_asset_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn replace_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<ReplaceMediaRequest>,
) -> Result<CommandResponse<MediaAssetView>, CommandError> {
    require_current_contract(&request)?;
    let data = replace_media_asset_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<MediaAssetRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_media_asset_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn link_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<AssetLinkRequest>,
) -> Result<CommandResponse<MediaAssetView>, CommandError> {
    require_current_contract(&request)?;
    let data = change_media_link_impl(&state, request.request_id, &request.payload, true)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn unlink_media_asset(
    state: State<'_, AppState>,
    request: CommandRequest<AssetLinkRequest>,
) -> Result<CommandResponse<MediaAssetView>, CommandError> {
    require_current_contract(&request)?;
    let data = change_media_link_impl(&state, request.request_id, &request.payload, false)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_visual_profiles(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<VisualProfile>>, CommandError> {
    require_current_contract(&request)?;
    let data = list_visual_profiles_impl(&state, request.request_id, &request.payload.project_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn save_visual_profile(
    state: State<'_, AppState>,
    request: CommandRequest<WriteVisualProfileRequest>,
) -> Result<CommandResponse<VisualProfile>, CommandError> {
    require_current_contract(&request)?;
    let data = save_visual_profile_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_visual_profile(
    state: State<'_, AppState>,
    request: CommandRequest<StoryEntityRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_visual_profile_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_style_presets(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<StylePreset>>, CommandError> {
    require_current_contract(&request)?;
    let data = list_style_presets_impl(&state, request.request_id, &request.payload.project_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_style_preset(
    state: State<'_, AppState>,
    request: CommandRequest<WriteStylePresetRequest>,
) -> Result<CommandResponse<StylePreset>, CommandError> {
    require_current_contract(&request)?;
    let data = write_style_preset_impl(&state, request.request_id, &request.payload, true)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn update_style_preset(
    state: State<'_, AppState>,
    request: CommandRequest<WriteStylePresetRequest>,
) -> Result<CommandResponse<StylePreset>, CommandError> {
    require_current_contract(&request)?;
    let data = write_style_preset_impl(&state, request.request_id, &request.payload, false)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_style_preset(
    state: State<'_, AppState>,
    request: CommandRequest<StylePresetRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_style_preset_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn compose_image_prompt(
    state: State<'_, AppState>,
    request: CommandRequest<ComposeImagePromptRequest>,
) -> Result<CommandResponse<ImagePromptComposition>, CommandError> {
    require_current_contract(&request)?;
    let data = compose_image_prompt_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_image_prompt_drafts(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<ImagePromptDraft>>, CommandError> {
    require_current_contract(&request)?;
    let data =
        list_image_prompt_drafts_impl(&state, request.request_id, &request.payload.project_id)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_image_prompt_draft(
    state: State<'_, AppState>,
    request: CommandRequest<CreateImagePromptDraftRequest>,
) -> Result<CommandResponse<ImagePromptDraft>, CommandError> {
    require_current_contract(&request)?;
    let data = create_image_prompt_draft_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn update_image_prompt_draft(
    state: State<'_, AppState>,
    request: CommandRequest<UpdateImagePromptDraftRequest>,
) -> Result<CommandResponse<ImagePromptDraft>, CommandError> {
    require_current_contract(&request)?;
    let data = update_image_prompt_draft_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_image_prompt_draft(
    state: State<'_, AppState>,
    request: CommandRequest<ImagePromptDraftRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_image_prompt_draft_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn export_image_prompt_draft(
    state: State<'_, AppState>,
    request: CommandRequest<ImagePromptDraftRequest>,
) -> Result<CommandResponse<PromptExportResult>, CommandError> {
    require_current_contract(&request)?;
    let data = export_image_prompt_draft_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_story_entities(
    state: State<'_, AppState>,
    request: CommandRequest<ListStoryEntitiesRequest>,
) -> Result<CommandResponse<Vec<StoryEntity>>, CommandError> {
    require_current_contract(&request)?;
    let data = list_story_entities_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_story_entity(
    state: State<'_, AppState>,
    request: CommandRequest<WriteStoryEntityRequest>,
) -> Result<CommandResponse<StoryEntity>, CommandError> {
    require_current_contract(&request)?;
    let data = write_story_entity_impl(&state, request.request_id, &request.payload, false)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn update_story_entity(
    state: State<'_, AppState>,
    request: CommandRequest<WriteStoryEntityRequest>,
) -> Result<CommandResponse<StoryEntity>, CommandError> {
    require_current_contract(&request)?;
    let data = write_story_entity_impl(&state, request.request_id, &request.payload, true)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn load_story_entity(
    state: State<'_, AppState>,
    request: CommandRequest<StoryEntityRequest>,
) -> Result<CommandResponse<StoryEntityDetails>, CommandError> {
    require_current_contract(&request)?;
    let data = load_story_entity_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_story_entity(
    state: State<'_, AppState>,
    request: CommandRequest<StoryEntityRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_story_entity_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn merge_story_entities(
    state: State<'_, AppState>,
    request: CommandRequest<MergeStoryEntitiesRequest>,
) -> Result<CommandResponse<StoryEntityDetails>, CommandError> {
    require_current_contract(&request)?;
    let data = merge_story_entities_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_story_relation(
    state: State<'_, AppState>,
    request: CommandRequest<CreateStoryRelationRequest>,
) -> Result<CommandResponse<StoryRelation>, CommandError> {
    require_current_contract(&request)?;
    let data = create_story_relation_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_story_relation(
    state: State<'_, AppState>,
    request: CommandRequest<DeleteStoryRelationRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    delete_story_relation_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn link_story_entity(
    state: State<'_, AppState>,
    request: CommandRequest<LinkStoryEntityRequest>,
) -> Result<CommandResponse<StoryEntityDetails>, CommandError> {
    require_current_contract(&request)?;
    let data = link_story_entity_impl(&state, request.request_id, &request.payload)?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn get_plot_board(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<PlotBoard>, CommandError> {
    require_current_contract(&request)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.plot_board(project),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_storyline(
    state: State<'_, AppState>,
    request: CommandRequest<StorylineWriteRequest>,
) -> Result<CommandResponse<Storyline>, CommandError> {
    require_current_contract(&request)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| {
            repository.create_storyline(project, &request.payload.name, &request.payload.color)
        },
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_storyline(
    state: State<'_, AppState>,
    request: CommandRequest<StorylineDeleteRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let id = parse_entity_id(request.request_id, &request.payload.storyline_id)?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.delete_storyline(project, id),
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn update_plot_card(
    state: State<'_, AppState>,
    request: CommandRequest<PlotCardWriteRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let document = parse_entity_id(request.request_id, &request.payload.document_id)?;
    let storyline =
        parse_optional_entity_id(request.request_id, request.payload.storyline_id.as_deref())?;
    let pov =
        parse_optional_entity_id(request.request_id, request.payload.pov_entity_id.as_deref())?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| {
            repository.update_card(project, document, request.payload.status, storyline, pov)
        },
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn reorder_plot_cards(
    state: State<'_, AppState>,
    request: CommandRequest<ReorderRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    if request.payload.parent_id.is_some() {
        return Err(CommandError::invalid_request(
            request.request_id,
            "Die Kartenreihenfolge ist ungültig.",
        ));
    }
    let ids = parse_entity_ids(request.request_id, &request.payload.ordered_ids)?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.reorder_cards(project, &ids),
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn list_timelines(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<Vec<Timeline>>, CommandError> {
    require_current_contract(&request)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.list_timelines(project),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_timeline(
    state: State<'_, AppState>,
    request: CommandRequest<TimelineWriteRequest>,
) -> Result<CommandResponse<Timeline>, CommandError> {
    require_current_contract(&request)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| {
            repository.create_timeline(project, &request.payload.name, request.payload.mode)
        },
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_timeline(
    state: State<'_, AppState>,
    request: CommandRequest<TimelineRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let id = parse_entity_id(request.request_id, &request.payload.timeline_id)?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.delete_timeline(project, id),
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn load_timeline(
    state: State<'_, AppState>,
    request: CommandRequest<TimelineRequest>,
) -> Result<CommandResponse<TimelineDetails>, CommandError> {
    require_current_contract(&request)?;
    let id = parse_entity_id(request.request_id, &request.payload.timeline_id)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.load_timeline(project, id),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_timeline_event(
    state: State<'_, AppState>,
    request: CommandRequest<EventWriteRequest>,
) -> Result<CommandResponse<TimelineEvent>, CommandError> {
    require_current_contract(&request)?;
    let timeline = parse_entity_id(request.request_id, &request.payload.timeline_id)?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| {
            repository.create_event(
                project,
                timeline,
                &request.payload.title,
                &request.payload.description,
                request.payload.time_value.as_deref(),
            )
        },
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn delete_timeline_event(
    state: State<'_, AppState>,
    request: CommandRequest<EventRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let event = parse_entity_id(request.request_id, &request.payload.event_id)?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.delete_event(project, event),
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn reorder_timeline_events(
    state: State<'_, AppState>,
    request: CommandRequest<ReorderRequest>,
) -> Result<CommandResponse<()>, CommandError> {
    require_current_contract(&request)?;
    let timeline =
        request.payload.parent_id.as_deref().ok_or_else(|| {
            CommandError::invalid_request(request.request_id, "Die Zeitebene fehlt.")
        })?;
    let timeline = parse_entity_id(request.request_id, timeline)?;
    let ids = parse_entity_ids(request.request_id, &request.payload.ordered_ids)?;
    with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| repository.reorder_events(project, timeline, &ids),
    )?;
    Ok(CommandResponse::current(request.request_id, ()))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn link_timeline_event(
    state: State<'_, AppState>,
    request: CommandRequest<EventLinkRequest>,
) -> Result<CommandResponse<TimelineDetails>, CommandError> {
    require_current_contract(&request)?;
    let event = parse_entity_id(request.request_id, &request.payload.event_id)?;
    let timeline = parse_entity_id(request.request_id, &request.payload.timeline_id)?;
    let document =
        parse_optional_entity_id(request.request_id, request.payload.document_id.as_deref())?;
    let entity =
        parse_optional_entity_id(request.request_id, request.payload.entity_id.as_deref())?;
    let data = with_structure(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, repository| {
            let before = repository.load_timeline(project, timeline)?;
            if !before.events.iter().any(|item| item.id == event) {
                return Err(StructureError::InvalidInput);
            }
            repository.link_event(project, event, document, entity)?;
            repository.load_timeline(project, timeline)
        },
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn create_project_snapshot(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<SnapshotSummary>, CommandError> {
    require_current_contract(&request)?;
    let data = with_git_service(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, service, database| service.create_snapshot(database, project),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn validate_project_snapshot(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<SnapshotSummary>, CommandError> {
    require_current_contract(&request)?;
    let data = with_git_service(
        &state,
        request.request_id,
        &request.payload.project_id,
        |project, service, _| service.validate_snapshot(project),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn get_version_control_status(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<VersionControlStatus>, CommandError> {
    require_current_contract(&request)?;
    let data = with_git_service(
        &state,
        request.request_id,
        &request.payload.project_id,
        |_, service, _| service.status(),
    )?;
    Ok(CommandResponse::current(request.request_id, data))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn get_version_control_diff(
    state: State<'_, AppState>,
    request: CommandRequest<ProjectRequest>,
) -> Result<CommandResponse<VersionControlDiffResponse>, CommandError> {
    require_current_contract(&request)?;
    let diff = with_git_service(
        &state,
        request.request_id,
        &request.payload.project_id,
        |_, service, _| service.diff(),
    )?;
    Ok(CommandResponse::current(
        request.request_id,
        VersionControlDiffResponse { diff },
    ))
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri besitzt Command-Argumente"
)]
fn commit_project_snapshot(
    state: State<'_, AppState>,
    request: CommandRequest<CommitSnapshotRequest>,
) -> Result<CommandResponse<CommitResult>, CommandError> {
    require_current_contract(&request)?;
    let data = with_git_service(
        &state,
        request.request_id,
        &request.payload.project_id,
        |_, service, _| service.commit(&request.payload.message),
    )?;
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

fn create_example_project_impl(
    state: &AppState,
    request_id: uuid::Uuid,
) -> Result<WorkspaceResponse, CommandError> {
    let manager = manager(state, request_id)?;
    let (opened, summary) = import_snapshot_archive(&manager, EXAMPLE_PROJECT_ARCHIVE)
        .map_err(|error| map_git_error(request_id, &error))?;
    drop(opened);
    open_workspace_impl(state, request_id, &summary.project_id.to_string())
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

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn rename_document_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: RenameDocumentRequest,
) -> Result<DocumentNode, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    DocumentRepository::new(opened.database_mut())
        .rename(
            project_id,
            document_id,
            &request.expected_title,
            &request.title,
        )
        .map_err(|error| map_document_error(request_id, &error))
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn reorder_documents_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: ReorderDocumentsRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let parent_id = request
        .parent_id
        .as_deref()
        .map(|value| parse_entity_id(request_id, value))
        .transpose()?;
    let expected_order = parse_entity_ids(request_id, &request.expected_order)?;
    let ordered_ids = parse_entity_ids(request_id, &request.ordered_ids)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    DocumentRepository::new(opened.database_mut())
        .reorder(project_id, parent_id, &expected_order, &ordered_ids)
        .map_err(|error| map_document_error(request_id, &error))
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "DTO wird an der Servicegrenze übernommen"
)]
fn delete_document_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: DeleteDocumentRequest,
) -> Result<DeleteDocumentResponse, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let expected_child_ids = parse_entity_ids(request_id, &request.expected_child_ids)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    DocumentRepository::new(opened.database_mut())
        .delete(
            project_id,
            document_id,
            &request.expected_title,
            request.expected_revision,
            &expected_child_ids,
        )
        .map_err(|error| map_document_error(request_id, &error))?;
    let mut deleted_ids = Vec::with_capacity(expected_child_ids.len() + 1);
    deleted_ids.push(document_id);
    deleted_ids.extend(expected_child_ids);
    Ok(DeleteDocumentResponse { deleted_ids })
}

fn export_manuscript_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ExportProjectRequest,
) -> Result<ExportResult, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
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
    let project = Project {
        id: project_id,
        name,
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    let project_directory = opened.directory().to_path_buf();
    let repository = DocumentRepository::new(opened.database_mut());
    let documents = repository
        .list(project_id)
        .map_err(|error| map_document_error(request_id, &error))?
        .into_iter()
        .map(|node| {
            repository
                .load(project_id, node.id)
                .map(|stored| ExportSourceDocument {
                    node: stored.node,
                    content: stored.content,
                })
                .map_err(|error| map_document_error(request_id, &error))
        })
        .collect::<Result<Vec<_>, _>>()?;
    export_project(&project_directory, &project, &documents, request.format)
        .map_err(|error| map_export_error(request_id, &error))
}

fn resolve_export_directory(
    state: &AppState,
    request_id: uuid::Uuid,
    raw_project_id: &str,
) -> Result<PathBuf, CommandError> {
    let project_id = parse_project_id(request_id, raw_project_id)?;
    let manager = manager(state, request_id)?;
    let opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let project_directory =
        fs::canonicalize(opened.directory()).map_err(|_| CommandError::internal(request_id))?;
    let export_directory = project_directory.join("exports");
    match fs::symlink_metadata(&export_directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(CommandError::internal(request_id));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&export_directory).map_err(|_| CommandError::internal(request_id))?;
        }
        Err(_) => return Err(CommandError::internal(request_id)),
    }
    let canonical_export =
        fs::canonicalize(export_directory).map_err(|_| CommandError::internal(request_id))?;
    if canonical_export.parent() != Some(project_directory.as_path()) {
        return Err(CommandError::internal(request_id));
    }
    Ok(canonical_export)
}

#[cfg(target_os = "windows")]
fn open_directory(directory: &std::path::Path) -> std::io::Result<()> {
    Command::new("explorer.exe")
        .arg(directory)
        .spawn()
        .map(drop)
}

#[cfg(target_os = "macos")]
fn open_directory(directory: &std::path::Path) -> std::io::Result<()> {
    Command::new("open").arg(directory).spawn().map(drop)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_directory(directory: &std::path::Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(directory).spawn().map(drop)
}

fn preview_ai_context_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &AiContextRequest,
) -> Result<AiContextPreview, CommandError> {
    if request.model.trim().is_empty()
        || request.model.chars().count() > 200
        || request.model.chars().any(char::is_control)
        || request.instruction.trim().is_empty()
        || request.instruction.chars().count() > 4_000
    {
        return Err(CommandError::invalid_request(
            request_id,
            "Das ausgewählte Modell ist ungültig.",
        ));
    }
    let descriptor = if request.provider_id == kraqel_ai_service::OLLAMA_PROVIDER_ID {
        OllamaProvider::local_default()
            .map_err(|error| map_text_provider_error(request_id, &error))?
            .descriptor()
    } else {
        parse_cloud_provider(request_id, &request.provider_id)?.descriptor()
    };
    let stored = load_ai_scene(state, request_id, &request.project_id, &request.document_id)?;
    let release_id = uuid::Uuid::new_v4();
    let mut releases = state
        .ai_releases
        .lock()
        .map_err(|_| CommandError::internal(request_id))?;
    releases.retain(|_, release| release.created_at.elapsed() <= AI_RELEASE_LIFETIME);
    releases.insert(
        release_id,
        AiReleaseRecord {
            project_id: request.project_id.clone(),
            document_id: request.document_id.clone(),
            revision: stored.node.revision,
            provider_id: request.provider_id.clone(),
            model: request.model.clone(),
            instruction: request.instruction.trim().to_owned(),
            created_at: Instant::now(),
        },
    );
    let mut metadata = BTreeMap::new();
    metadata.insert("Szenentitel".to_owned(), stored.node.title.clone());
    Ok(AiContextPreview {
        document_id: stored.node.id,
        document_title: stored.node.title,
        revision: stored.node.revision,
        text: stored.plain_text,
        provider_id: descriptor.id.to_owned(),
        provider_display_name: descriptor.display_name.to_owned(),
        provider_location: match descriptor.location {
            kraqel_ai_service::ProviderLocation::Local => "local",
            kraqel_ai_service::ProviderLocation::Cloud => "cloud",
        }
        .to_owned(),
        model: request.model.clone(),
        instruction: request.instruction.trim().to_owned(),
        metadata,
        release_id,
    })
}

fn prepare_text_generation(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &StreamTextDraftRequest,
) -> Result<TextGenerationRequest, CommandError> {
    if !request.context_approved {
        return Err(CommandError::invalid_request(
            request_id,
            "Der angezeigte Kontext wurde nicht freigegeben.",
        ));
    }
    let release_id = uuid::Uuid::parse_str(&request.release_id).map_err(|_| {
        CommandError::invalid_request(request_id, "Die Kontextfreigabe ist ungültig.")
    })?;
    let release = state
        .ai_releases
        .lock()
        .map_err(|_| CommandError::internal(request_id))?
        .remove(&release_id)
        .ok_or_else(|| {
            CommandError::invalid_request(
                request_id,
                "Die Kontextfreigabe fehlt oder wurde bereits verwendet.",
            )
        })?;
    if release.created_at.elapsed() > AI_RELEASE_LIFETIME
        || release.project_id != request.project_id
        || release.document_id != request.document_id
        || release.revision != request.expected_revision
        || release.provider_id != request.provider_id
        || release.model != request.model
        || release.instruction != request.instruction.trim()
    {
        return Err(CommandError::invalid_request(
            request_id,
            "Die Kontextfreigabe passt nicht zur Textanfrage.",
        ));
    }
    let stored = load_ai_scene(state, request_id, &request.project_id, &request.document_id)?;
    if stored.node.revision != request.expected_revision {
        return Err(CommandError::conflict(request_id));
    }
    Ok(TextGenerationRequest {
        model: request.model.clone(),
        instruction: request.instruction.clone(),
        context: ReleasedTextContext {
            label: stored.node.title,
            text: stored.plain_text,
            approved: true,
        },
    })
}

fn load_ai_scene(
    state: &AppState,
    request_id: uuid::Uuid,
    project_id: &str,
    document_id: &str,
) -> Result<kraqel_storage::document_repository::StoredDocument, CommandError> {
    let project_id = parse_project_id(request_id, project_id)?;
    let document_id = parse_entity_id(request_id, document_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let stored = DocumentRepository::new(opened.database_mut())
        .load(project_id, document_id)
        .map_err(|error| map_document_error(request_id, &error))?;
    if stored.node.kind != DocumentKind::Scene {
        return Err(CommandError::invalid_request(
            request_id,
            "Nur Szenen können als Textkontext freigegeben werden.",
        ));
    }
    Ok(stored)
}

fn review_text_suggestion_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ReviewTextSuggestionRequest,
) -> Result<SuggestionReview, CommandError> {
    if request.proposed_text.len() > 2 * 1024 * 1024 {
        return Err(CommandError::invalid_request(
            request_id,
            "Der Textvorschlag ist zu groß.",
        ));
    }
    let stored = load_ai_scene(state, request_id, &request.project_id, &request.document_id)?;
    if stored.node.revision != request.expected_revision {
        return Err(CommandError::conflict(request_id));
    }
    Ok(review_suggestion(
        stored.node.id,
        stored.node.revision,
        &stored.plain_text,
        &request.proposed_text,
    ))
}

fn list_media_assets_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
) -> Result<Vec<MediaAssetView>, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    let assets = MediaRepository::new(opened.database_mut())
        .list(project_id)
        .map_err(|error| map_media_error(request_id, &error))?;
    Ok(assets
        .into_iter()
        .map(|asset| media_view(&directory, asset))
        .collect())
}

fn import_media_asset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ImportMediaRequest,
) -> Result<MediaAssetView, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let prepared = prepare_image(&request.file_name, &request.bytes)
        .map_err(|error| map_asset_error(request_id, &error))?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    if let Some(existing) = MediaRepository::new(opened.database_mut())
        .find_by_checksum(project_id, &prepared.checksum)
        .map_err(|error| map_media_error(request_id, &error))?
    {
        return Ok(media_view(&directory, existing));
    }
    let id = EntityId::new();
    let stored = store_image(&directory, id, &prepared)
        .map_err(|error| map_asset_error(request_id, &error))?;
    let inserted = MediaRepository::new(opened.database_mut()).insert(
        project_id,
        id,
        &stored.relative_path,
        &stored.thumbnail_path,
        &prepared.original_name,
        &request.title,
        prepared.mime_type,
        &prepared.checksum,
        prepared.original_bytes.len() as u64,
        prepared.width,
        prepared.height,
        &request.tags,
    );
    match inserted {
        Ok(asset) => Ok(media_view(&directory, asset)),
        Err(error) => {
            let _ = remove_asset_files(&directory, &stored.relative_path, &stored.thumbnail_path);
            Err(map_media_error(request_id, &error))
        }
    }
}

fn update_media_asset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &UpdateMediaRequest,
) -> Result<MediaAssetView, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let asset_id = parse_entity_id(request_id, &request.asset_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    let asset = MediaRepository::new(opened.database_mut())
        .update_metadata(project_id, asset_id, &request.title, &request.tags)
        .map_err(|error| map_media_error(request_id, &error))?;
    Ok(media_view(&directory, asset))
}

fn replace_media_asset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ReplaceMediaRequest,
) -> Result<MediaAssetView, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let asset_id = parse_entity_id(request_id, &request.asset_id)?;
    let prepared = prepare_image(&request.file_name, &request.bytes)
        .map_err(|error| map_asset_error(request_id, &error))?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    let mut repository = MediaRepository::new(opened.database_mut());
    let previous = repository
        .load(project_id, asset_id)
        .map_err(|error| map_media_error(request_id, &error))?;
    if previous.checksum == prepared.checksum {
        return Ok(media_view(&directory, previous));
    }
    if repository
        .find_by_checksum(project_id, &prepared.checksum)
        .map_err(|error| map_media_error(request_id, &error))?
        .is_some()
    {
        return Err(CommandError::invalid_request(
            request_id,
            "Dieses Bild ist bereits in der Medienbibliothek vorhanden.",
        ));
    }
    let stored = store_image(&directory, asset_id, &prepared)
        .map_err(|error| map_asset_error(request_id, &error))?;
    let replaced = repository.replace_content(
        project_id,
        asset_id,
        &stored.relative_path,
        &stored.thumbnail_path,
        &prepared.original_name,
        prepared.mime_type,
        &prepared.checksum,
        prepared.original_bytes.len() as u64,
        prepared.width,
        prepared.height,
    );
    match replaced {
        Ok(asset) => {
            let _ = remove_asset_files(
                &directory,
                &previous.relative_path,
                &previous.thumbnail_path,
            );
            Ok(media_view(&directory, asset))
        }
        Err(error) => {
            let _ = remove_asset_files(&directory, &stored.relative_path, &stored.thumbnail_path);
            Err(map_media_error(request_id, &error))
        }
    }
}

fn delete_media_asset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &MediaAssetRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let asset_id = parse_entity_id(request_id, &request.asset_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    let deleted = MediaRepository::new(opened.database_mut())
        .delete(project_id, asset_id)
        .map_err(|error| map_media_error(request_id, &error))?;
    let _ = remove_asset_files(&directory, &deleted.relative_path, &deleted.thumbnail_path);
    Ok(())
}

fn change_media_link_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &AssetLinkRequest,
    add: bool,
) -> Result<MediaAssetView, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let asset_id = parse_entity_id(request_id, &request.asset_id)?;
    let target_id = parse_entity_id(request_id, &request.target_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let directory = opened.directory().to_path_buf();
    let mut repository = MediaRepository::new(opened.database_mut());
    let asset = if add {
        repository.link(
            project_id,
            asset_id,
            request.target_kind,
            target_id,
            &request.role,
        )
    } else {
        repository.unlink(
            project_id,
            asset_id,
            request.target_kind,
            target_id,
            &request.role,
        )
    }
    .map_err(|error| map_media_error(request_id, &error))?;
    Ok(media_view(&directory, asset))
}

fn media_view(directory: &std::path::Path, asset: MediaAsset) -> MediaAssetView {
    match verify_original(directory, &asset.relative_path, &asset.checksum)
        .and_then(|()| read_thumbnail(directory, &asset.thumbnail_path))
    {
        Ok(bytes) => MediaAssetView {
            asset,
            thumbnail_data_url: Some(format!("data:image/png;base64,{}", BASE64.encode(bytes))),
            file_available: true,
        },
        Err(_) => MediaAssetView {
            asset,
            thumbnail_data_url: None,
            file_available: false,
        },
    }
}

fn list_visual_profiles_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
) -> Result<Vec<VisualProfile>, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    VisualRepository::new(opened.database_mut())
        .list_profiles(project_id)
        .map_err(|error| map_visual_error(request_id, &error))
}

fn save_visual_profile_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &WriteVisualProfileRequest,
) -> Result<VisualProfile, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let entity_id = parse_entity_id(request_id, &request.entity_id)?;
    let canonical_asset_id = request
        .canonical_asset_id
        .as_deref()
        .map(|value| parse_entity_id(request_id, value))
        .transpose()?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    VisualRepository::new(opened.database_mut())
        .upsert_profile(
            project_id,
            entity_id,
            &request.traits,
            canonical_asset_id,
            &request.exclusions,
        )
        .map_err(|error| map_visual_error(request_id, &error))
}

fn delete_visual_profile_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &StoryEntityRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let entity_id = parse_entity_id(request_id, &request.entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    VisualRepository::new(opened.database_mut())
        .delete_profile(project_id, entity_id)
        .map_err(|error| map_visual_error(request_id, &error))
}

fn list_style_presets_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
) -> Result<Vec<StylePreset>, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    VisualRepository::new(opened.database_mut())
        .list_presets(project_id)
        .map_err(|error| map_visual_error(request_id, &error))
}

fn write_style_preset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &WriteStylePresetRequest,
    create: bool,
) -> Result<StylePreset, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let mut repository = VisualRepository::new(opened.database_mut());
    let result = if create {
        if request.preset_id.is_some() {
            return Err(CommandError::invalid_request(
                request_id,
                "Eine neue Stilvorlage darf noch keine Kennung besitzen.",
            ));
        }
        repository.create_preset(
            project_id,
            &request.name,
            &request.visual_rules,
            &request.applicable_kinds,
        )
    } else {
        let preset_id = request
            .preset_id
            .as_deref()
            .ok_or_else(|| {
                CommandError::invalid_request(request_id, "Die Stilvorlagenkennung fehlt.")
            })
            .and_then(|value| parse_entity_id(request_id, value))?;
        repository.update_preset(
            project_id,
            preset_id,
            &request.name,
            &request.visual_rules,
            &request.applicable_kinds,
        )
    };
    result.map_err(|error| map_visual_error(request_id, &error))
}

fn delete_style_preset_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &StylePresetRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let preset_id = parse_entity_id(request_id, &request.preset_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    VisualRepository::new(opened.database_mut())
        .delete_preset(project_id, preset_id)
        .map_err(|error| map_visual_error(request_id, &error))
}

fn compose_image_prompt_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ComposeImagePromptRequest,
) -> Result<ImagePromptComposition, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let entity_ids = request
        .entity_ids
        .iter()
        .map(|value| parse_entity_id(request_id, value))
        .collect::<Result<Vec<_>, _>>()?;
    let style_preset_id = request
        .style_preset_id
        .as_deref()
        .map(|value| parse_entity_id(request_id, value))
        .transpose()?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    PromptRepository::new(opened.database_mut())
        .compose(project_id, document_id, &entity_ids, style_preset_id)
        .map_err(|error| map_prompt_error(request_id, &error))
}

fn list_image_prompt_drafts_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
) -> Result<Vec<ImagePromptDraft>, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    PromptRepository::new(opened.database_mut())
        .list_drafts(project_id)
        .map_err(|error| map_prompt_error(request_id, &error))
}

fn create_image_prompt_draft_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &CreateImagePromptDraftRequest,
) -> Result<ImagePromptDraft, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    PromptRepository::new(opened.database_mut())
        .create_draft(
            project_id,
            &request.name,
            &request.composition,
            &request.prompt_text,
        )
        .map_err(|error| map_prompt_error(request_id, &error))
}

fn update_image_prompt_draft_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &UpdateImagePromptDraftRequest,
) -> Result<ImagePromptDraft, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let draft_id = parse_entity_id(request_id, &request.draft_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    PromptRepository::new(opened.database_mut())
        .update_draft(project_id, draft_id, &request.name, &request.prompt_text)
        .map_err(|error| map_prompt_error(request_id, &error))
}

fn delete_image_prompt_draft_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ImagePromptDraftRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let draft_id = parse_entity_id(request_id, &request.draft_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    PromptRepository::new(opened.database_mut())
        .delete_draft(project_id, draft_id)
        .map_err(|error| map_prompt_error(request_id, &error))
}

fn export_image_prompt_draft_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ImagePromptDraftRequest,
) -> Result<PromptExportResult, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let draft_id = parse_entity_id(request_id, &request.draft_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let project_directory = opened.directory().to_path_buf();
    let name: String = opened
        .database()
        .connection()
        .query_row(
            "SELECT name FROM project WHERE id = ?1",
            [project_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|_| CommandError::internal(request_id))?;
    let draft = PromptRepository::new(opened.database_mut())
        .load_draft(project_id, draft_id)
        .map_err(|error| map_prompt_error(request_id, &error))?;
    export_prompt_draft(
        &project_directory,
        &Project {
            id: project_id,
            name,
            schema_version: CURRENT_SCHEMA_VERSION,
        },
        &draft,
    )
    .map_err(|error| map_prompt_export_error(request_id, &error))
}

fn list_story_entities_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &ListStoryEntitiesRequest,
) -> Result<Vec<StoryEntity>, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .list(project_id, request.query.as_deref(), request.kind)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn write_story_entity_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &WriteStoryEntityRequest,
    updating: bool,
) -> Result<StoryEntity, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let mut repository = StoryBibleRepository::new(opened.database_mut());
    let result = if updating {
        let entity_id = request.entity_id.as_deref().ok_or_else(|| {
            CommandError::invalid_request(request_id, "Die Entitätskennung fehlt.")
        })?;
        repository.update_entity(
            project_id,
            parse_entity_id(request_id, entity_id)?,
            request.kind,
            &request.name,
            &request.properties,
        )
    } else {
        if request.entity_id.is_some() {
            return Err(CommandError::invalid_request(
                request_id,
                "Neue Einträge dürfen keine Entitätskennung vorgeben.",
            ));
        }
        repository.create_entity(project_id, request.kind, &request.name, &request.properties)
    };
    result.map_err(|error| map_story_bible_error(request_id, &error))
}

fn load_story_entity_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &StoryEntityRequest,
) -> Result<StoryEntityDetails, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let entity_id = parse_entity_id(request_id, &request.entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .load(project_id, entity_id)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn delete_story_entity_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &StoryEntityRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let entity_id = parse_entity_id(request_id, &request.entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .delete_entity(project_id, entity_id)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn merge_story_entities_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &MergeStoryEntitiesRequest,
) -> Result<StoryEntityDetails, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let source_entity_id = parse_entity_id(request_id, &request.source_entity_id)?;
    let target_entity_id = parse_entity_id(request_id, &request.target_entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .merge_entities(project_id, source_entity_id, target_entity_id)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn create_story_relation_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &CreateStoryRelationRequest,
) -> Result<StoryRelation, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let source_entity_id = parse_entity_id(request_id, &request.source_entity_id)?;
    let target_entity_id = parse_entity_id(request_id, &request.target_entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .create_relation(
            project_id,
            source_entity_id,
            target_entity_id,
            &request.kind,
            &request.properties,
        )
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn delete_story_relation_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &DeleteStoryRelationRequest,
) -> Result<(), CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let relation_id = parse_entity_id(request_id, &request.relation_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    StoryBibleRepository::new(opened.database_mut())
        .delete_relation(project_id, relation_id)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn link_story_entity_impl(
    state: &AppState,
    request_id: uuid::Uuid,
    request: &LinkStoryEntityRequest,
) -> Result<StoryEntityDetails, CommandError> {
    let project_id = parse_project_id(request_id, &request.project_id)?;
    let document_id = parse_entity_id(request_id, &request.document_id)?;
    let entity_id = parse_entity_id(request_id, &request.entity_id)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    let mut repository = StoryBibleRepository::new(opened.database_mut());
    repository
        .link_document(project_id, document_id, entity_id)
        .map_err(|error| map_story_bible_error(request_id, &error))?;
    repository
        .load(project_id, entity_id)
        .map_err(|error| map_story_bible_error(request_id, &error))
}

fn manager(state: &AppState, request_id: uuid::Uuid) -> Result<ProjectManager, CommandError> {
    ProjectManager::new(&state.project_root).map_err(|_| CommandError::internal(request_id))
}

fn with_structure<T>(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
    operation: impl FnOnce(ProjectId, &mut StructureRepository<'_>) -> Result<T, StructureError>,
) -> Result<T, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let mut opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    operation(
        project_id,
        &mut StructureRepository::new(opened.database_mut()),
    )
    .map_err(|error| map_structure_error(request_id, &error))
}

fn with_git_service<T>(
    state: &AppState,
    request_id: uuid::Uuid,
    project_value: &str,
    operation: impl FnOnce(
        ProjectId,
        &GitService<'_>,
        &kraqel_storage::Database,
    ) -> Result<T, GitServiceError>,
) -> Result<T, CommandError> {
    let project_id = parse_project_id(request_id, project_value)?;
    let manager = manager(state, request_id)?;
    let opened = manager
        .open_project(project_id.to_string())
        .map_err(|error| map_project_error(request_id, &error))?;
    operation(
        project_id,
        &GitService::new(opened.directory()),
        opened.database(),
    )
    .map_err(|error| map_git_error(request_id, &error))
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

fn parse_optional_entity_id(
    request_id: uuid::Uuid,
    value: Option<&str>,
) -> Result<Option<EntityId>, CommandError> {
    value
        .map(|item| parse_entity_id(request_id, item))
        .transpose()
}

fn parse_entity_ids(
    request_id: uuid::Uuid,
    values: &[String],
) -> Result<Vec<EntityId>, CommandError> {
    values
        .iter()
        .map(|value| parse_entity_id(request_id, value))
        .collect()
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

fn map_story_bible_error(request_id: uuid::Uuid, error: &StoryBibleError) -> CommandError {
    match error {
        StoryBibleError::InvalidInput | StoryBibleError::NotFound => {
            CommandError::invalid_request(request_id, "Die Story-Bibel-Anfrage ist ungültig.")
        }
        StoryBibleError::InvalidStoredData
        | StoryBibleError::Storage(_)
        | StoryBibleError::InvalidSystemTime => CommandError::internal(request_id),
    }
}

fn map_structure_error(request_id: uuid::Uuid, error: &StructureError) -> CommandError {
    match error {
        StructureError::InvalidInput | StructureError::NotFound => {
            CommandError::invalid_request(request_id, "Die Struktur-Anfrage ist ungültig.")
        }
        StructureError::InvalidStoredData
        | StructureError::Storage(_)
        | StructureError::InvalidSystemTime => CommandError::internal(request_id),
    }
}

fn map_continuity_error(request_id: uuid::Uuid, _error: &ContinuityError) -> CommandError {
    CommandError::internal(request_id)
}

fn map_media_error(request_id: uuid::Uuid, error: &MediaError) -> CommandError {
    match error {
        MediaError::InvalidInput | MediaError::NotFound | MediaError::Duplicate => {
            CommandError::invalid_request(request_id, "Die Medienanfrage ist ungültig.")
        }
        MediaError::InvalidStoredData | MediaError::InvalidSystemTime | MediaError::Storage(_) => {
            CommandError::internal(request_id)
        }
    }
}

fn map_visual_error(request_id: uuid::Uuid, error: &VisualError) -> CommandError {
    match error {
        VisualError::InvalidInput | VisualError::NotFound | VisualError::Duplicate => {
            CommandError::invalid_request(request_id, "Die visuelle Profilanfrage ist ungültig.")
        }
        VisualError::InvalidStoredData
        | VisualError::InvalidSystemTime
        | VisualError::Storage(_) => CommandError::internal(request_id),
    }
}

fn map_prompt_error(request_id: uuid::Uuid, error: &PromptError) -> CommandError {
    match error {
        PromptError::Conflict => CommandError::conflict(request_id),
        PromptError::InvalidInput | PromptError::NotFound => {
            CommandError::invalid_request(request_id, "Die Bildprompt-Anfrage ist ungültig.")
        }
        PromptError::InvalidStoredData
        | PromptError::InvalidSystemTime
        | PromptError::Storage(_) => CommandError::internal(request_id),
    }
}

fn map_asset_error(request_id: uuid::Uuid, error: &AssetError) -> CommandError {
    match error {
        AssetError::InvalidName
        | AssetError::UnsupportedFormat
        | AssetError::InvalidImage
        | AssetError::TooLarge => {
            CommandError::invalid_request(request_id, "Die Bilddatei ist ungültig.")
        }
        AssetError::UnsafePath | AssetError::AlreadyExists | AssetError::Io(_) => {
            CommandError::internal(request_id)
        }
    }
}

fn map_git_error(request_id: uuid::Uuid, error: &GitServiceError) -> CommandError {
    match error {
        GitServiceError::InvalidInput | GitServiceError::NothingToCommit => {
            CommandError::invalid_request(request_id, "Die Versionsanfrage ist ungültig.")
        }
        GitServiceError::InvalidSnapshot => {
            CommandError::invalid_request(request_id, "Der Projektsnapshot ist ungültig.")
        }
        GitServiceError::GitFailed
        | GitServiceError::Database(_)
        | GitServiceError::Io(_)
        | GitServiceError::Json(_)
        | GitServiceError::Project(_) => CommandError::internal(request_id),
    }
}

fn map_export_error(request_id: uuid::Uuid, error: &ExportError) -> CommandError {
    match error {
        ExportError::InvalidDocument(_) => CommandError::invalid_request(
            request_id,
            "Das Manuskript enthält eine noch nicht exportierbare Formatierung.",
        ),
        ExportError::UnsafeDestination | ExportError::Internal(_) => {
            CommandError::internal(request_id)
        }
    }
}

fn map_prompt_export_error(request_id: uuid::Uuid, error: &ExportError) -> CommandError {
    match error {
        ExportError::InvalidDocument(_) => CommandError::invalid_request(
            request_id,
            "Der Bildprompt kann nicht exportiert werden.",
        ),
        ExportError::UnsafeDestination | ExportError::Internal(_) => {
            CommandError::internal(request_id)
        }
    }
}

fn map_text_provider_error(request_id: uuid::Uuid, error: &TextProviderError) -> CommandError {
    match error {
        TextProviderError::ContextNotApproved
        | TextProviderError::InvalidRequest
        | TextProviderError::CredentialMissing => {
            CommandError::invalid_request(request_id, "Die Textanfrage ist ungültig.")
        }
        TextProviderError::Unavailable
        | TextProviderError::InvalidResponse
        | TextProviderError::ResponseTooLarge
        | TextProviderError::StreamClosed
        | TextProviderError::CredentialRejected => CommandError::internal(request_id),
    }
}

fn parse_cloud_provider(
    request_id: uuid::Uuid,
    provider_id: &str,
) -> Result<CloudProviderId, CommandError> {
    provider_id
        .parse()
        .map_err(|_| CommandError::invalid_request(request_id, "Der Cloudprovider ist ungültig."))
}

fn map_credential_error(request_id: uuid::Uuid, error: CredentialError) -> CommandError {
    match error {
        CredentialError::Invalid | CredentialError::Missing => {
            CommandError::invalid_request(request_id, "Der API-Schlüssel ist ungültig oder fehlt.")
        }
        CredentialError::Unavailable => CommandError::internal(request_id),
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
            app.manage(AppState {
                project_root,
                ai_releases: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            application_status,
            validate_project_name,
            list_projects,
            create_project,
            create_example_project,
            open_workspace,
            create_document,
            load_document,
            save_document,
            rename_document,
            reorder_documents,
            delete_document,
            export_manuscript,
            open_export_directory,
            list_ollama_models,
            list_cloud_provider_statuses,
            save_cloud_credential,
            delete_cloud_credential,
            list_cloud_models,
            preview_ai_context,
            stream_text_draft,
            analyze_continuity,
            review_text_suggestion,
            list_media_assets,
            import_media_asset,
            update_media_asset,
            replace_media_asset,
            delete_media_asset,
            link_media_asset,
            unlink_media_asset,
            list_visual_profiles,
            save_visual_profile,
            delete_visual_profile,
            list_style_presets,
            create_style_preset,
            update_style_preset,
            delete_style_preset,
            compose_image_prompt,
            list_image_prompt_drafts,
            create_image_prompt_draft,
            update_image_prompt_draft,
            delete_image_prompt_draft,
            export_image_prompt_draft,
            list_story_entities,
            create_story_entity,
            update_story_entity,
            load_story_entity,
            delete_story_entity,
            merge_story_entities,
            create_story_relation,
            delete_story_relation,
            link_story_entity,
            get_plot_board,
            create_storyline,
            delete_storyline,
            update_plot_card,
            reorder_plot_cards,
            list_timelines,
            create_timeline,
            delete_timeline,
            load_timeline,
            create_timeline_event,
            delete_timeline_event,
            reorder_timeline_events,
            link_timeline_event,
            create_project_snapshot,
            validate_project_snapshot,
            get_version_control_status,
            get_version_control_diff,
            commit_project_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("KRAQEL konnte nicht gestartet werden");
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        fs,
        sync::Mutex,
    };

    use base64::Engine as _;
    use kraqel_domain::DocumentKind;
    use kraqel_domain::{
        COMMAND_CONTRACT_VERSION, CommandErrorCode, CommandRequest, EDITOR_DOCUMENT_VERSION,
        EditorDocument, EntityKind,
    };
    use kraqel_export::ExportFormat;
    use serde_json::json;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{
        AiContextRequest, AppState, ApplicationStatusRequest, ComposeImagePromptRequest,
        CreateDocumentRequest, CreateImagePromptDraftRequest, CreateProjectRequest,
        DeleteDocumentRequest, DocumentRequest, ExportProjectRequest, ImagePromptDraftRequest,
        ImportMediaRequest, LinkStoryEntityRequest, ListStoryEntitiesRequest,
        MergeStoryEntitiesRequest, RenameDocumentRequest, ReorderDocumentsRequest,
        ReviewTextSuggestionRequest, SaveDocumentRequest, StoryEntityRequest,
        StreamTextDraftRequest, UpdateImagePromptDraftRequest, ValidateProjectNameRequest,
        WriteStoryEntityRequest, WriteStylePresetRequest, WriteVisualProfileRequest,
        application_status, compose_image_prompt_impl, create_document_impl,
        create_example_project_impl, create_image_prompt_draft_impl, create_project_impl,
        delete_document_impl, delete_image_prompt_draft_impl, export_image_prompt_draft_impl,
        export_manuscript_impl, import_media_asset_impl, link_story_entity_impl,
        list_image_prompt_drafts_impl, list_media_assets_impl, list_projects_impl,
        list_story_entities_impl, list_style_presets_impl, list_visual_profiles_impl,
        load_document_impl, load_story_entity_impl, merge_story_entities_impl, open_workspace_impl,
        prepare_text_generation, preview_ai_context_impl, rename_document_impl,
        reorder_documents_impl, resolve_export_directory, review_text_suggestion_impl,
        save_document_impl, save_visual_profile_impl, update_image_prompt_draft_impl,
        validate_project_name, write_story_entity_impl, write_style_preset_impl,
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
    fn bundled_example_is_created_offline_as_a_new_workspace() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
        };

        let workspace = create_example_project_impl(&state, REQUEST_ID)
            .expect("bundled example should create a workspace");

        assert_eq!(
            workspace.project.name,
            "Die Stadt, die den Dienstag verkaufte (Beispiel)"
        );
        assert_eq!(workspace.documents.len(), 20);
        assert_eq!(list_projects_impl(&state, REQUEST_ID).unwrap().len(), 1);
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
            ai_releases: Mutex::new(HashMap::new()),
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
        let exported = export_manuscript_impl(
            &state,
            REQUEST_ID,
            &ExportProjectRequest {
                project_id: workspace.project.id.to_string(),
                format: ExportFormat::Markdown,
            },
        )
        .expect("manuscript should export");
        assert_eq!(exported.file_name, "manuskript.md");
        let exported_text = std::fs::read_to_string(
            root.path()
                .join(workspace.project.id.to_string())
                .join("exports/manuskript.md"),
        )
        .expect("export should be readable");
        assert!(exported_text.contains("Hallo Welt"));
        let export_directory =
            resolve_export_directory(&state, REQUEST_ID, &workspace.project.id.to_string())
                .expect("export directory should resolve from the project id");
        assert_eq!(
            export_directory,
            fs::canonicalize(
                root.path()
                    .join(workspace.project.id.to_string())
                    .join("exports")
            )
            .expect("export directory should be canonical")
        );
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

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Der vertikale Commandtest hält Konflikt- und Kaskadenablauf zusammen"
    )]
    fn document_management_commands_preserve_conflicts_and_cascade_explicitly() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
        };
        let workspace = create_project_impl(
            &state,
            REQUEST_ID,
            CreateProjectRequest {
                name: "Morgenrot".to_owned(),
            },
        )
        .expect("project should be created");
        let chapter = workspace
            .documents
            .iter()
            .find(|document| document.kind == kraqel_domain::DocumentKind::Chapter)
            .expect("initial chapter")
            .clone();
        let first_scene = workspace
            .documents
            .iter()
            .find(|document| document.kind == kraqel_domain::DocumentKind::Scene)
            .expect("initial scene")
            .clone();
        let renamed = rename_document_impl(
            &state,
            REQUEST_ID,
            RenameDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: first_scene.id.to_string(),
                expected_title: first_scene.title.clone(),
                title: "Ankunft".to_owned(),
            },
        )
        .expect("scene should rename");
        assert_eq!(renamed.title, "Ankunft");
        let stale_rename = rename_document_impl(
            &state,
            REQUEST_ID,
            RenameDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: first_scene.id.to_string(),
                expected_title: first_scene.title,
                title: "Veraltet".to_owned(),
            },
        )
        .expect_err("stale title should conflict");
        assert_eq!(stale_rename.code, CommandErrorCode::Conflict);

        let second_scene = create_document_impl(
            &state,
            REQUEST_ID,
            CreateDocumentRequest {
                project_id: workspace.project.id.to_string(),
                parent_id: Some(chapter.id.to_string()),
                kind: kraqel_domain::DocumentKind::Scene,
                title: "Szene 2".to_owned(),
            },
        )
        .expect("second scene");
        reorder_documents_impl(
            &state,
            REQUEST_ID,
            ReorderDocumentsRequest {
                project_id: workspace.project.id.to_string(),
                parent_id: Some(chapter.id.to_string()),
                expected_order: vec![first_scene.id.to_string(), second_scene.id.to_string()],
                ordered_ids: vec![second_scene.id.to_string(), first_scene.id.to_string()],
            },
        )
        .expect("scenes should reorder");
        let stale_reorder = reorder_documents_impl(
            &state,
            REQUEST_ID,
            ReorderDocumentsRequest {
                project_id: workspace.project.id.to_string(),
                parent_id: Some(chapter.id.to_string()),
                expected_order: vec![first_scene.id.to_string(), second_scene.id.to_string()],
                ordered_ids: vec![second_scene.id.to_string(), first_scene.id.to_string()],
            },
        )
        .expect_err("stale order should conflict");
        assert_eq!(stale_reorder.code, CommandErrorCode::Conflict);

        let deleted = delete_document_impl(
            &state,
            REQUEST_ID,
            DeleteDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: chapter.id.to_string(),
                expected_title: chapter.title,
                expected_revision: chapter.revision,
                expected_child_ids: vec![second_scene.id.to_string(), first_scene.id.to_string()],
            },
        )
        .expect("displayed chapter should delete");
        assert_eq!(deleted.deleted_ids.len(), 3);
        assert_eq!(
            open_workspace_impl(&state, REQUEST_ID, &workspace.project.id.to_string())
                .expect("workspace should reopen")
                .documents,
            Vec::new()
        );
    }

    #[test]
    fn suggestion_review_is_revision_bound_and_does_not_write() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
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
        let request = ReviewTextSuggestionRequest {
            project_id: workspace.project.id.to_string(),
            document_id: scene.id.to_string(),
            expected_revision: 0,
            proposed_text: "Ein Vorschlag.".to_owned(),
        };
        let review = review_text_suggestion_impl(&state, REQUEST_ID, &request)
            .expect("current revision should be reviewable");
        assert!(review.changed);
        let unchanged = load_document_impl(
            &state,
            REQUEST_ID,
            DocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
            },
        )
        .expect("scene should load");
        assert_eq!(unchanged.node.revision, 0);
        assert_eq!(unchanged.plain_text, "");

        save_document_impl(
            &state,
            REQUEST_ID,
            SaveDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: 0,
                content: EditorDocument {
                    version: EDITOR_DOCUMENT_VERSION,
                    document: json!({
                        "type": "doc",
                        "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Neuer Stand"}]}]
                    }),
                },
            },
        )
        .expect("scene should save");
        let conflict = review_text_suggestion_impl(&state, REQUEST_ID, &request)
            .expect_err("stale review should conflict");
        assert_eq!(conflict.code, CommandErrorCode::Conflict);
    }

    #[test]
    fn media_import_is_local_hash_verified_and_listed_with_preview() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
        };
        let workspace = create_project_impl(
            &state,
            REQUEST_ID,
            CreateProjectRequest {
                name: "Bildarchiv".to_owned(),
            },
        )
        .expect("project should be created");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .expect("fixture should decode");
        let imported = import_media_asset_impl(
            &state,
            REQUEST_ID,
            &ImportMediaRequest {
                project_id: workspace.project.id.to_string(),
                file_name: "kolkrabe.png".to_owned(),
                title: "Kolkrabe bei Nacht".to_owned(),
                tags: vec!["Nacht".to_owned()],
                bytes,
            },
        )
        .expect("valid image should import");
        assert!(imported.file_available);
        assert!(
            imported
                .thumbnail_data_url
                .as_deref()
                .is_some_and(|value| value.starts_with("data:image/png;base64,"))
        );
        let listed = list_media_assets_impl(&state, REQUEST_ID, &workspace.project.id.to_string())
            .expect("assets should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].asset.id, imported.asset.id);
    }

    #[test]
    fn visual_profile_and_style_preset_roundtrip_through_commands() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
        };
        let workspace = create_project_impl(
            &state,
            REQUEST_ID,
            CreateProjectRequest {
                name: "Visuelles Archiv".to_owned(),
            },
        )
        .expect("project should be created");
        let entity = write_story_entity_impl(
            &state,
            REQUEST_ID,
            &WriteStoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: None,
                kind: EntityKind::Character,
                name: "Rhea".to_owned(),
                properties: BTreeMap::new(),
            },
            false,
        )
        .expect("entity should be created");
        let profile = save_visual_profile_impl(
            &state,
            REQUEST_ID,
            &WriteVisualProfileRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: entity.id.to_string(),
                traits: BTreeMap::from([("Augen".to_owned(), "bernsteinfarben".to_owned())]),
                canonical_asset_id: None,
                exclusions: vec!["blaue Augen".to_owned()],
            },
        )
        .expect("profile should save");
        assert_eq!(profile.entity_id, entity.id);
        assert_eq!(
            list_visual_profiles_impl(&state, REQUEST_ID, &workspace.project.id.to_string())
                .expect("profiles should list"),
            vec![profile]
        );

        let preset = write_style_preset_impl(
            &state,
            REQUEST_ID,
            &WriteStylePresetRequest {
                project_id: workspace.project.id.to_string(),
                preset_id: None,
                name: "Noir".to_owned(),
                visual_rules: BTreeMap::from([("Palette".to_owned(), "monochrom".to_owned())]),
                applicable_kinds: vec![EntityKind::Character],
            },
            true,
        )
        .expect("preset should save");
        assert_eq!(
            list_style_presets_impl(&state, REQUEST_ID, &workspace.project.id.to_string())
                .expect("presets should list"),
            vec![preset]
        );
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Der vertikale Test hält Auswahl, Bearbeitung, Speicherung und Export als einen Ablauf zusammen"
    )]
    fn image_prompt_roundtrip_stays_local_editable_and_exportable() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
        };
        let workspace = create_project_impl(
            &state,
            REQUEST_ID,
            CreateProjectRequest {
                name: "Prompt-Archiv".to_owned(),
            },
        )
        .expect("project should be created");
        let scene = workspace
            .documents
            .iter()
            .find(|document| document.kind == DocumentKind::Scene)
            .expect("scene should exist");
        save_document_impl(
            &state,
            REQUEST_ID,
            SaveDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: 0,
                content: EditorDocument {
                    version: EDITOR_DOCUMENT_VERSION,
                    document: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Kolkrabe am Turm"}]}]}),
                },
            },
        )
        .expect("scene should save");
        let entity = write_story_entity_impl(
            &state,
            REQUEST_ID,
            &WriteStoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: None,
                kind: EntityKind::Character,
                name: "Rhea".to_owned(),
                properties: BTreeMap::from([("Rolle".to_owned(), "Botin".to_owned())]),
            },
            false,
        )
        .expect("entity should save");
        save_visual_profile_impl(
            &state,
            REQUEST_ID,
            &WriteVisualProfileRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: entity.id.to_string(),
                traits: BTreeMap::from([("Augen".to_owned(), "bernsteinfarben".to_owned())]),
                canonical_asset_id: None,
                exclusions: vec![],
            },
        )
        .expect("profile should save");
        let composition = compose_image_prompt_impl(
            &state,
            REQUEST_ID,
            &ComposeImagePromptRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                entity_ids: vec![entity.id.to_string()],
                style_preset_id: None,
            },
        )
        .expect("prompt should compose");
        assert!(composition.prompt_text.contains("Kolkrabe am Turm"));
        let created = create_image_prompt_draft_impl(
            &state,
            REQUEST_ID,
            &CreateImagePromptDraftRequest {
                project_id: workspace.project.id.to_string(),
                name: "Turm".to_owned(),
                composition,
                prompt_text: "Bewusst bearbeiteter Kolkrabe".to_owned(),
            },
        )
        .expect("draft should save");
        let updated = update_image_prompt_draft_impl(
            &state,
            REQUEST_ID,
            &UpdateImagePromptDraftRequest {
                project_id: workspace.project.id.to_string(),
                draft_id: created.id.to_string(),
                name: "Turm bei Nacht".to_owned(),
                prompt_text: "Kolkrabe, Tusche, Abendlicht".to_owned(),
            },
        )
        .expect("draft should update");
        assert_eq!(
            list_image_prompt_drafts_impl(&state, REQUEST_ID, &workspace.project.id.to_string())
                .expect("drafts should list"),
            vec![updated.clone()]
        );
        let request = ImagePromptDraftRequest {
            project_id: workspace.project.id.to_string(),
            draft_id: updated.id.to_string(),
        };
        let exported = export_image_prompt_draft_impl(&state, REQUEST_ID, &request)
            .expect("draft should export");
        let path = root
            .path()
            .join(workspace.project.id.to_string())
            .join("exports/prompts")
            .join(exported.file_name);
        assert_eq!(
            fs::read_to_string(path).expect("export should be readable"),
            "Kolkrabe, Tusche, Abendlicht\n"
        );
        delete_image_prompt_draft_impl(&state, REQUEST_ID, &request).expect("draft should delete");
    }

    #[test]
    fn ai_context_requires_preview_release_and_current_revision() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
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
            .expect("scene");
        save_document_impl(
            &state,
            REQUEST_ID,
            SaveDocumentRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: 0,
                content: EditorDocument {
                    version: EDITOR_DOCUMENT_VERSION,
                    document: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Nur dieser Kontext"}]}]}),
                },
            },
        )
        .expect("scene save");
        let preview = preview_ai_context_impl(
            &state,
            REQUEST_ID,
            &AiContextRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                provider_id: "ollama".to_owned(),
                model: "rabe:7b".to_owned(),
                instruction: "Weiterschreiben".to_owned(),
            },
        )
        .expect("preview");
        assert_eq!(preview.text, "Nur dieser Kontext");
        assert_eq!(preview.revision, 1);
        let error = prepare_text_generation(
            &state,
            REQUEST_ID,
            &StreamTextDraftRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                expected_revision: preview.revision,
                model: "rabe:7b".to_owned(),
                instruction: "Weiterschreiben".to_owned(),
                context_approved: false,
                provider_id: "ollama".to_owned(),
                release_id: preview.release_id.to_string(),
            },
        )
        .expect_err("release should be required");
        assert_eq!(error.code, CommandErrorCode::InvalidRequest);

        let approved = StreamTextDraftRequest {
            project_id: workspace.project.id.to_string(),
            document_id: scene.id.to_string(),
            expected_revision: preview.revision,
            model: "rabe:7b".to_owned(),
            instruction: "Weiterschreiben".to_owned(),
            context_approved: true,
            provider_id: "ollama".to_owned(),
            release_id: preview.release_id.to_string(),
        };
        let generation = prepare_text_generation(&state, REQUEST_ID, &approved)
            .expect("matching release should work once");
        assert_eq!(generation.context.text, "Nur dieser Kontext");
        let reused = prepare_text_generation(&state, REQUEST_ID, &approved)
            .expect_err("release must be single use");
        assert_eq!(reused.code, CommandErrorCode::InvalidRequest);
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Der vertikale Command-Test belegt Suche, Backlinks und den atomaren Mergevertrag"
    )]
    fn story_bible_commands_roundtrip_search_backlink_and_merge() {
        let root = tempdir().expect("temporary root should exist");
        let state = AppState {
            project_root: root.path().to_path_buf(),
            ai_releases: Mutex::new(HashMap::new()),
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
            .expect("scene should exist");
        let entity = write_story_entity_impl(
            &state,
            REQUEST_ID,
            &WriteStoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: None,
                kind: EntityKind::Character,
                name: "Rhea".to_owned(),
                properties: BTreeMap::from([("Rolle".to_owned(), "Pilotin".to_owned())]),
            },
            false,
        )
        .expect("entity should be created");

        let matches = list_story_entities_impl(
            &state,
            REQUEST_ID,
            &ListStoryEntitiesRequest {
                project_id: workspace.project.id.to_string(),
                query: Some("pilotin".to_owned()),
                kind: Some(EntityKind::Character),
            },
        )
        .expect("entity should be found");
        assert_eq!(matches, vec![entity.clone()]);

        let linked = link_story_entity_impl(
            &state,
            REQUEST_ID,
            &LinkStoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                document_id: scene.id.to_string(),
                entity_id: entity.id.to_string(),
            },
        )
        .expect("scene should link");
        assert_eq!(linked.backlinks.len(), 1);
        assert_eq!(linked.backlinks[0].document_title, "Szene 1");

        let loaded = load_story_entity_impl(
            &state,
            REQUEST_ID,
            &StoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: entity.id.to_string(),
            },
        )
        .expect("entity should load");
        assert_eq!(loaded.entity, entity);

        let duplicate = write_story_entity_impl(
            &state,
            REQUEST_ID,
            &WriteStoryEntityRequest {
                project_id: workspace.project.id.to_string(),
                entity_id: None,
                kind: EntityKind::Character,
                name: "Rhea doppelt".to_owned(),
                properties: BTreeMap::from([("Herkunft".to_owned(), "Veyra".to_owned())]),
            },
            false,
        )
        .expect("duplicate should be created");
        let merged = merge_story_entities_impl(
            &state,
            REQUEST_ID,
            &MergeStoryEntitiesRequest {
                project_id: workspace.project.id.to_string(),
                source_entity_id: duplicate.id.to_string(),
                target_entity_id: entity.id.to_string(),
            },
        )
        .expect("duplicates should merge");
        assert_eq!(merged.entity.id, entity.id);
        assert_eq!(merged.entity.properties["Rolle"], "Pilotin");
        assert_eq!(merged.entity.properties["Herkunft"], "Veyra");
        assert!(
            load_story_entity_impl(
                &state,
                REQUEST_ID,
                &StoryEntityRequest {
                    project_id: workspace.project.id.to_string(),
                    entity_id: duplicate.id.to_string(),
                },
            )
            .is_err()
        );
    }
}
