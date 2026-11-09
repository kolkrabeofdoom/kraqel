import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  COMMAND_CONTRACT_VERSION,
  KraqelCommandError,
  commitProjectSnapshot,
  createStoryEntity,
  reorderPlotCards,
  getApplicationStatus,
  saveDocument,
  streamTextDraft,
  validateProjectName,
} from "./commands";
import type { EntityId, ProjectId } from "@kraqel/domain";
import { EDITOR_DOCUMENT_VERSION } from "@kraqel/editor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class MockChannel<T> {
    onmessage: (message: T) => void = () => undefined;
  },
}));

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

  it("sends the complete Plotboard order through the versioned boundary", async () => {
    const projectId = requestId as ProjectId;
    const first = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const second = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4" as EntityId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });

    await reorderPlotCards(projectId, [second, first]);

    expect(mockedInvoke).toHaveBeenCalledWith("reorder_plot_cards", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, parentId: null, orderedIds: [second, first] },
      },
    });
  });

  it("sends a commit message only through the versioned Git command", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { commitId: "abc123" },
    });

    await expect(commitProjectSnapshot(projectId, "Kapitel überarbeitet")).resolves.toEqual({
      commitId: "abc123",
    });
    expect(mockedInvoke).toHaveBeenCalledWith("commit_project_snapshot", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, message: "Kapitel überarbeitet" },
      },
    });
  });

  it("streams text events through a channel without changing editor content", async () => {
    const projectId = requestId as ProjectId;
    const documentId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const events: unknown[] = [];
    mockedInvoke.mockImplementation(async (_command, arguments_) => {
      const channel = (arguments_ as { onEvent: { onmessage: (message: unknown) => void } })
        .onEvent;
      channel.onmessage({ type: "delta", text: "Kolkrabe" });
      return { contractVersion: COMMAND_CONTRACT_VERSION, requestId, data: null };
    });

    await streamTextDraft(
      {
        projectId,
        documentId,
        expectedRevision: 3,
        model: "rabe:7b",
        instruction: "Weiterschreiben",
        contextApproved: true,
      },
      (event) => events.push(event),
    );

    expect(events).toEqual([{ type: "delta", text: "Kolkrabe" }]);
    expect(mockedInvoke).toHaveBeenCalledWith("stream_text_draft", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          documentId,
          expectedRevision: 3,
          model: "rabe:7b",
          instruction: "Weiterschreiben",
          contextApproved: true,
        },
      },
      onEvent: expect.any(Object),
    });
  });
});
