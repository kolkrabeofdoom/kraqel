import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AiContextPreview,
  CloudProviderStatus,
  TextModel,
  TextProviderStatus,
  TextStreamEvent,
} from "@kraqel/ai-contracts";
import {
  CURRENT_SCHEMA_VERSION,
  parseUuid,
  type DocumentKind,
  type DocumentNode,
  type EntityId,
  type EntityKind,
  type PlotBoard,
  type PlotStatus,
  type Project,
  type ProjectId,
  type SchemaVersion,
  type Uuid,
  type StoryEntity,
  type StoryEntityDetails,
  type StoryRelation,
  type Storyline,
  type Timeline,
  type TimelineDetails,
  type TimelineEvent,
  type TimelineMode,
} from "@kraqel/domain";
import type { EditorDocument } from "@kraqel/editor";

export const COMMAND_CONTRACT_VERSION = 1 as const;

export type CommandErrorCode =
  "INVALID_REQUEST" | "UNSUPPORTED_CONTRACT_VERSION" | "CONFLICT" | "INTERNAL";

export interface CommandRequest<T> {
  readonly contractVersion: typeof COMMAND_CONTRACT_VERSION;
  readonly requestId: Uuid;
  readonly payload: T;
}

export interface CommandResponse<T> {
  readonly contractVersion: typeof COMMAND_CONTRACT_VERSION;
  readonly requestId: Uuid;
  readonly data: T;
}

export interface CommandErrorContract {
  readonly contractVersion: typeof COMMAND_CONTRACT_VERSION;
  readonly requestId: Uuid;
  readonly code: CommandErrorCode;
  readonly message: string;
  readonly retryable: boolean;
}

export class KraqelCommandError extends Error {
  readonly code: CommandErrorCode;
  readonly requestId: Uuid;
  readonly retryable: boolean;

  constructor(error: CommandErrorContract) {
    super(safeMessageFor(error.code));
    this.name = "KraqelCommandError";
    this.code = error.code;
    this.requestId = error.requestId;
    this.retryable = error.retryable;
  }
}

export interface ApplicationStatus {
  readonly version: string;
  readonly schemaVersion: SchemaVersion;
  readonly offlineReady: boolean;
}

export interface ProjectNameValidation {
  readonly valid: boolean;
}

export interface ProjectOverview {
  readonly id: ProjectId;
  readonly name: string;
}

export interface Workspace {
  readonly project: Project;
  readonly documents: readonly DocumentNode[];
}

export interface StoredDocument {
  readonly node: DocumentNode;
  readonly content: EditorDocument;
  readonly plainText: string;
}

export interface SnapshotSummary {
  readonly formatVersion: number;
  readonly projectId: ProjectId;
  readonly fileCount: number;
  readonly byteCount: number;
}

export interface VersionControlStatus {
  readonly initialized: boolean;
  readonly clean: boolean;
  readonly changedFiles: readonly string[];
  readonly head: string | null;
}

export interface CommitResult {
  readonly commitId: string;
}

export type ExportFormat = "markdown" | "html" | "docx" | "pdf";

export interface ExportResult {
  readonly format: ExportFormat;
  readonly fileName: string;
  readonly byteCount: number;
}

export interface CreateDocumentInput {
  readonly projectId: ProjectId;
  readonly parentId: EntityId | null;
  readonly kind: DocumentKind;
  readonly title: string;
}

export async function getApplicationStatus(): Promise<ApplicationStatus> {
  const response = await callCommand<Record<string, never>, ApplicationStatus>(
    "application_status",
    {},
  );

  if (response.data.schemaVersion !== CURRENT_SCHEMA_VERSION) {
    throw internalContractError(response.requestId);
  }

  return response.data;
}

export async function validateProjectName(name: string): Promise<ProjectNameValidation> {
  const response = await callCommand<{ readonly name: string }, ProjectNameValidation>(
    "validate_project_name",
    { name },
  );
  return response.data;
}

export async function listProjects(): Promise<readonly ProjectOverview[]> {
  const response = await callCommand<Record<string, never>, readonly ProjectOverview[]>(
    "list_projects",
    {},
  );
  return response.data;
}

export async function createProject(name: string): Promise<Workspace> {
  const response = await callCommand<{ readonly name: string }, Workspace>("create_project", {
    name,
  });
  return response.data;
}

export async function openWorkspace(projectId: ProjectId): Promise<Workspace> {
  const response = await callCommand<{ readonly projectId: ProjectId }, Workspace>(
    "open_workspace",
    { projectId },
  );
  return response.data;
}

export async function createDocument(input: CreateDocumentInput): Promise<DocumentNode> {
  const response = await callCommand<CreateDocumentInput, DocumentNode>("create_document", input);
  return response.data;
}

export async function loadDocument(
  projectId: ProjectId,
  documentId: EntityId,
): Promise<StoredDocument> {
  const response = await callCommand<
    { readonly projectId: ProjectId; readonly documentId: EntityId },
    StoredDocument
  >("load_document", { projectId, documentId });
  return response.data;
}

export async function saveDocument(
  projectId: ProjectId,
  documentId: EntityId,
  expectedRevision: number,
  content: EditorDocument,
): Promise<number> {
  const response = await callCommand<
    {
      readonly projectId: ProjectId;
      readonly documentId: EntityId;
      readonly expectedRevision: number;
      readonly content: EditorDocument;
    },
    { readonly revision: number }
  >("save_document", { projectId, documentId, expectedRevision, content });
  return response.data.revision;
}

export async function exportManuscript(
  projectId: ProjectId,
  format: ExportFormat,
): Promise<ExportResult> {
  return (
    await callCommand<
      { readonly projectId: ProjectId; readonly format: ExportFormat },
      ExportResult
    >("export_manuscript", { projectId, format })
  ).data;
}

export async function listOllamaModels(): Promise<TextProviderStatus> {
  return (await callCommand<Record<string, never>, TextProviderStatus>("list_ollama_models", {}))
    .data;
}

export async function previewAiContext(
  projectId: ProjectId,
  documentId: EntityId,
  providerId: string,
  model: string,
  instruction: string,
): Promise<AiContextPreview> {
  return (
    await callCommand<
      {
        readonly projectId: ProjectId;
        readonly documentId: EntityId;
        readonly providerId: string;
        readonly model: string;
        readonly instruction: string;
      },
      AiContextPreview
    >("preview_ai_context", { projectId, documentId, providerId, model, instruction })
  ).data;
}

export async function listCloudProviderStatuses(): Promise<readonly CloudProviderStatus[]> {
  return (
    await callCommand<Record<string, never>, readonly CloudProviderStatus[]>(
      "list_cloud_provider_statuses",
      {},
    )
  ).data;
}

export async function saveCloudCredential(providerId: string, apiKey: string): Promise<void> {
  await callCommand<{ readonly providerId: string; readonly apiKey: string }, null>(
    "save_cloud_credential",
    { providerId, apiKey },
  );
}

export async function deleteCloudCredential(providerId: string): Promise<void> {
  await callCommand<{ readonly providerId: string }, null>("delete_cloud_credential", {
    providerId,
  });
}

export async function listCloudModels(providerId: string): Promise<readonly TextModel[]> {
  return (
    await callCommand<{ readonly providerId: string }, readonly TextModel[]>("list_cloud_models", {
      providerId,
    })
  ).data;
}

export interface StreamTextDraftInput {
  readonly projectId: ProjectId;
  readonly documentId: EntityId;
  readonly expectedRevision: number;
  readonly model: string;
  readonly instruction: string;
  readonly contextApproved: boolean;
  readonly providerId: string;
  readonly releaseId: string;
}

export async function streamTextDraft(
  input: StreamTextDraftInput,
  onEvent: (event: TextStreamEvent) => void,
): Promise<void> {
  const requestId = newRequestId();
  const channel = new Channel<TextStreamEvent>();
  channel.onmessage = onEvent;
  const request: CommandRequest<StreamTextDraftInput> = {
    contractVersion: COMMAND_CONTRACT_VERSION,
    requestId,
    payload: input,
  };
  try {
    const response = await invoke<CommandResponse<null>>("stream_text_draft", {
      request,
      onEvent: channel,
    });
    if (!isCommandResponse(response, requestId)) throw internalContractError(requestId);
  } catch (error: unknown) {
    if (error instanceof KraqelCommandError) throw error;
    if (isCommandErrorContract(error, requestId)) throw new KraqelCommandError(error);
    throw internalContractError(requestId);
  }
}

export async function listStoryEntities(
  projectId: ProjectId,
  query = "",
  kind: EntityKind | null = null,
): Promise<readonly StoryEntity[]> {
  const response = await callCommand<
    { readonly projectId: ProjectId; readonly query: string; readonly kind: EntityKind | null },
    readonly StoryEntity[]
  >("list_story_entities", { projectId, query, kind });
  return response.data;
}

export interface WriteStoryEntityInput {
  readonly projectId: ProjectId;
  readonly entityId: EntityId | null;
  readonly kind: EntityKind;
  readonly name: string;
  readonly properties: Readonly<Record<string, string>>;
}

export async function createStoryEntity(
  input: Omit<WriteStoryEntityInput, "entityId">,
): Promise<StoryEntity> {
  const response = await callCommand<WriteStoryEntityInput, StoryEntity>("create_story_entity", {
    ...input,
    entityId: null,
  });
  return response.data;
}

export async function updateStoryEntity(input: WriteStoryEntityInput): Promise<StoryEntity> {
  const response = await callCommand<WriteStoryEntityInput, StoryEntity>(
    "update_story_entity",
    input,
  );
  return response.data;
}

export async function loadStoryEntity(
  projectId: ProjectId,
  entityId: EntityId,
): Promise<StoryEntityDetails> {
  const response = await callCommand<
    { readonly projectId: ProjectId; readonly entityId: EntityId },
    StoryEntityDetails
  >("load_story_entity", { projectId, entityId });
  return response.data;
}

export async function deleteStoryEntity(projectId: ProjectId, entityId: EntityId): Promise<void> {
  await callCommand<{ readonly projectId: ProjectId; readonly entityId: EntityId }, null>(
    "delete_story_entity",
    { projectId, entityId },
  );
}

export async function createStoryRelation(
  projectId: ProjectId,
  sourceEntityId: EntityId,
  targetEntityId: EntityId,
  kind: string,
): Promise<StoryRelation> {
  const response = await callCommand<
    {
      readonly projectId: ProjectId;
      readonly sourceEntityId: EntityId;
      readonly targetEntityId: EntityId;
      readonly kind: string;
      readonly properties: Readonly<Record<string, string>>;
    },
    StoryRelation
  >("create_story_relation", {
    projectId,
    sourceEntityId,
    targetEntityId,
    kind,
    properties: {},
  });
  return response.data;
}

export async function deleteStoryRelation(
  projectId: ProjectId,
  relationId: EntityId,
): Promise<void> {
  await callCommand<{ readonly projectId: ProjectId; readonly relationId: EntityId }, null>(
    "delete_story_relation",
    { projectId, relationId },
  );
}

export async function linkStoryEntity(
  projectId: ProjectId,
  documentId: EntityId,
  entityId: EntityId,
): Promise<StoryEntityDetails> {
  const response = await callCommand<
    {
      readonly projectId: ProjectId;
      readonly documentId: EntityId;
      readonly entityId: EntityId;
    },
    StoryEntityDetails
  >("link_story_entity", { projectId, documentId, entityId });
  return response.data;
}

export async function getPlotBoard(projectId: ProjectId): Promise<PlotBoard> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, PlotBoard>("get_plot_board", { projectId })
  ).data;
}

export async function createStoryline(
  projectId: ProjectId,
  name: string,
  color: string,
): Promise<Storyline> {
  return (
    await callCommand<
      { readonly projectId: ProjectId; readonly name: string; readonly color: string },
      Storyline
    >("create_storyline", { projectId, name, color })
  ).data;
}

export async function deleteStoryline(projectId: ProjectId, storylineId: EntityId): Promise<void> {
  await callCommand("delete_storyline", { projectId, storylineId });
}

export async function updatePlotCard(
  projectId: ProjectId,
  documentId: EntityId,
  status: PlotStatus,
  storylineId: EntityId | null,
  povEntityId: EntityId | null,
): Promise<void> {
  await callCommand("update_plot_card", {
    projectId,
    documentId,
    status,
    storylineId,
    povEntityId,
  });
}

export async function reorderPlotCards(
  projectId: ProjectId,
  orderedIds: readonly EntityId[],
): Promise<void> {
  await callCommand("reorder_plot_cards", { projectId, parentId: null, orderedIds });
}

export async function listTimelines(projectId: ProjectId): Promise<readonly Timeline[]> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, readonly Timeline[]>("list_timelines", {
      projectId,
    })
  ).data;
}

export async function createTimeline(
  projectId: ProjectId,
  name: string,
  mode: TimelineMode,
): Promise<Timeline> {
  return (
    await callCommand<
      { readonly projectId: ProjectId; readonly name: string; readonly mode: TimelineMode },
      Timeline
    >("create_timeline", { projectId, name, mode })
  ).data;
}

export async function deleteTimeline(projectId: ProjectId, timelineId: EntityId): Promise<void> {
  await callCommand("delete_timeline", { projectId, timelineId });
}

export async function loadTimeline(
  projectId: ProjectId,
  timelineId: EntityId,
): Promise<TimelineDetails> {
  return (
    await callCommand<
      { readonly projectId: ProjectId; readonly timelineId: EntityId },
      TimelineDetails
    >("load_timeline", { projectId, timelineId })
  ).data;
}

export async function createTimelineEvent(
  projectId: ProjectId,
  timelineId: EntityId,
  title: string,
  description: string,
  timeValue: string | null,
): Promise<TimelineEvent> {
  return (
    await callCommand<
      {
        readonly projectId: ProjectId;
        readonly timelineId: EntityId;
        readonly title: string;
        readonly description: string;
        readonly timeValue: string | null;
      },
      TimelineEvent
    >("create_timeline_event", { projectId, timelineId, title, description, timeValue })
  ).data;
}

export async function deleteTimelineEvent(projectId: ProjectId, eventId: EntityId): Promise<void> {
  await callCommand("delete_timeline_event", { projectId, eventId });
}

export async function reorderTimelineEvents(
  projectId: ProjectId,
  timelineId: EntityId,
  orderedIds: readonly EntityId[],
): Promise<void> {
  await callCommand("reorder_timeline_events", { projectId, parentId: timelineId, orderedIds });
}

export async function linkTimelineEvent(
  projectId: ProjectId,
  timelineId: EntityId,
  eventId: EntityId,
  documentId: EntityId | null,
  entityId: EntityId | null,
): Promise<TimelineDetails> {
  return (
    (await callCommand("link_timeline_event", {
      projectId,
      timelineId,
      eventId,
      documentId,
      entityId,
    })) as CommandResponse<TimelineDetails>
  ).data;
}

export async function createProjectSnapshot(projectId: ProjectId): Promise<SnapshotSummary> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, SnapshotSummary>(
      "create_project_snapshot",
      { projectId },
    )
  ).data;
}

export async function validateProjectSnapshot(projectId: ProjectId): Promise<SnapshotSummary> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, SnapshotSummary>(
      "validate_project_snapshot",
      { projectId },
    )
  ).data;
}

export async function getVersionControlStatus(projectId: ProjectId): Promise<VersionControlStatus> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, VersionControlStatus>(
      "get_version_control_status",
      { projectId },
    )
  ).data;
}

export async function getVersionControlDiff(projectId: ProjectId): Promise<string> {
  return (
    await callCommand<{ readonly projectId: ProjectId }, { readonly diff: string }>(
      "get_version_control_diff",
      { projectId },
    )
  ).data.diff;
}

export async function commitProjectSnapshot(
  projectId: ProjectId,
  message: string,
): Promise<CommitResult> {
  return (
    await callCommand<{ readonly projectId: ProjectId; readonly message: string }, CommitResult>(
      "commit_project_snapshot",
      { projectId, message },
    )
  ).data;
}

async function callCommand<TPayload, TData>(
  command: string,
  payload: TPayload,
): Promise<CommandResponse<TData>> {
  const requestId = newRequestId();
  const request: CommandRequest<TPayload> = {
    contractVersion: COMMAND_CONTRACT_VERSION,
    requestId,
    payload,
  };

  try {
    const response = await invoke<CommandResponse<TData>>(command, { request });
    if (!isCommandResponse(response, requestId)) {
      throw internalContractError(requestId);
    }
    return response;
  } catch (error: unknown) {
    if (error instanceof KraqelCommandError) {
      throw error;
    }
    if (isCommandErrorContract(error, requestId)) {
      throw new KraqelCommandError(error);
    }
    throw internalContractError(requestId);
  }
}

function newRequestId(): Uuid {
  const requestId = parseUuid(crypto.randomUUID());
  if (!requestId) {
    throw new Error("Die Laufzeit konnte keine gültige Korrelationskennung erzeugen.");
  }
  return requestId;
}

function isCommandResponse<T>(value: unknown, requestId: Uuid): value is CommandResponse<T> {
  if (!isRecord(value)) return false;
  return value.contractVersion === COMMAND_CONTRACT_VERSION && value.requestId === requestId;
}

function isCommandErrorContract(value: unknown, requestId: Uuid): value is CommandErrorContract {
  if (!isRecord(value)) return false;
  return (
    value.contractVersion === COMMAND_CONTRACT_VERSION &&
    value.requestId === requestId &&
    isCommandErrorCode(value.code) &&
    typeof value.message === "string" &&
    typeof value.retryable === "boolean"
  );
}

function isCommandErrorCode(value: unknown): value is CommandErrorCode {
  return (
    value === "INVALID_REQUEST" ||
    value === "UNSUPPORTED_CONTRACT_VERSION" ||
    value === "CONFLICT" ||
    value === "INTERNAL"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function safeMessageFor(code: CommandErrorCode): string {
  switch (code) {
    case "INVALID_REQUEST":
      return "Die lokale Anwendung hat die Anfrage abgelehnt.";
    case "UNSUPPORTED_CONTRACT_VERSION":
      return "Die Command-Vertragsversion wird nicht unterstützt.";
    case "CONFLICT":
      return "Das Dokument wurde zwischenzeitlich verändert. Bitte neu laden.";
    case "INTERNAL":
      return "Die lokale Anwendung konnte die Anfrage nicht verarbeiten.";
  }
}

function internalContractError(requestId: Uuid): KraqelCommandError {
  return new KraqelCommandError({
    contractVersion: COMMAND_CONTRACT_VERSION,
    requestId,
    code: "INTERNAL",
    message: "Die lokale Anwendung hat eine ungültige Antwort geliefert.",
    retryable: false,
  });
}
