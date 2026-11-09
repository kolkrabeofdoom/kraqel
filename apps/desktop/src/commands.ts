import { invoke } from "@tauri-apps/api/core";
import { CURRENT_SCHEMA_VERSION, parseUuid, type SchemaVersion, type Uuid } from "@kraqel/domain";

export const COMMAND_CONTRACT_VERSION = 1 as const;

export type CommandErrorCode = "INVALID_REQUEST" | "UNSUPPORTED_CONTRACT_VERSION" | "INTERNAL";

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
    value === "INVALID_REQUEST" || value === "UNSUPPORTED_CONTRACT_VERSION" || value === "INTERNAL"
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
