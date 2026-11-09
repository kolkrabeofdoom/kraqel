import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  COMMAND_CONTRACT_VERSION,
  KraqelCommandError,
  createStoryEntity,
  getApplicationStatus,
  saveDocument,
  validateProjectName,
} from "./commands";
import type { EntityId, ProjectId } from "@kraqel/domain";
import { EDITOR_DOCUMENT_VERSION } from "@kraqel/editor";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);
const requestId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2";

beforeEach(() => {
  vi.stubGlobal("crypto", { randomUUID: () => requestId });
  mockedInvoke.mockReset();
});

describe("typed Tauri commands", () => {
  it("sends a versioned request and accepts the matching response", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { version: "0.1.0", schemaVersion: 1, offlineReady: true },
    });

    await expect(getApplicationStatus()).resolves.toEqual({
      version: "0.1.0",
      schemaVersion: 1,
      offlineReady: true,
    });
    expect(mockedInvoke).toHaveBeenCalledWith("application_status", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {},
      },
    });
  });

  it("does not echo a supplied project name in the result", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { valid: true },
    });

    const result = await validateProjectName("Geheimes Manuskript");

    expect(result).toEqual({ valid: true });
    expect(JSON.stringify(result)).not.toContain("Geheimes Manuskript");
  });

  it("maps unknown rejections to a sanitized internal error", async () => {
    mockedInvoke.mockRejectedValue(new Error("D:\\private\\manuscript.md"));

    const error = await getApplicationStatus().catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(KraqelCommandError);
    expect(error).toMatchObject({ code: "INTERNAL", retryable: false });
    expect(String(error)).not.toContain("private");
  });

  it("does not trust an error message received across the IPC boundary", async () => {
    mockedInvoke.mockRejectedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      code: "INVALID_REQUEST",
      message: "D:\\private\\manuscript.md",
      retryable: false,
    });

    const error = await getApplicationStatus().catch((reason: unknown) => reason);

    expect(error).toMatchObject({ code: "INVALID_REQUEST", retryable: false });
    expect(String(error)).not.toContain("private");
  });

  it("sends versioned editor content with an optimistic revision", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { revision: 8 },
    });
    const projectId = requestId as ProjectId;
    const documentId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const content = {
      version: EDITOR_DOCUMENT_VERSION,
      document: { type: "doc", content: [{ type: "paragraph" }] },
    };

    await expect(saveDocument(projectId, documentId, 7, content)).resolves.toBe(8);
    expect(mockedInvoke).toHaveBeenCalledWith("save_document", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, documentId, expectedRevision: 7, content },
      },
    });
  });

  it("maps revision conflicts to a stable local message", async () => {
    mockedInvoke.mockRejectedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      code: "CONFLICT",
      message: "D:\\private\\draft.md",
      retryable: false,
    });

    const error = await getApplicationStatus().catch((reason: unknown) => reason);

    expect(error).toMatchObject({ code: "CONFLICT", retryable: false });
    expect(String(error)).toContain("zwischenzeitlich");
    expect(String(error)).not.toContain("private");
  });

  it("sends structured Story Bible properties without inventing an entity id", async () => {
    const projectId = requestId as ProjectId;
    const entityId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: {
        id: entityId,
        projectId,
        kind: "character",
        name: "Rhea",
        properties: { Rolle: "Pilotin" },
      },
    });

    await createStoryEntity({
      projectId,
      kind: "character",
      name: "Rhea",
      properties: { Rolle: "Pilotin" },
    });

    expect(mockedInvoke).toHaveBeenCalledWith("create_story_entity", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          entityId: null,
          kind: "character",
          name: "Rhea",
          properties: { Rolle: "Pilotin" },
        },
      },
    });
  });
});
