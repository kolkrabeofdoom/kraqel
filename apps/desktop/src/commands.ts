import { invoke } from "@tauri-apps/api/core";
import {
  CURRENT_SCHEMA_VERSION,
  parseUuid,
  type DocumentKind,
  type DocumentNode,
  type EntityId,
  type EntityKind,
  type Project,
  type ProjectId,
  type SchemaVersion,
  type Uuid,
  type StoryEntity,
  type StoryEntityDetails,
  type StoryRelation,
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
