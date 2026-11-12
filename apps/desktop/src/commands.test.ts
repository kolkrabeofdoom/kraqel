import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  COMMAND_CONTRACT_VERSION,
  KraqelCommandError,
  analyzeContinuity,
  commitProjectSnapshot,
  composeImagePrompt,
  createExampleProject,
  createImagePromptDraft,
  createStoryEntity,
  deleteDocument,
  deleteProject,
  reorderPlotCards,
  reorderDocuments,
  getApplicationStatus,
  saveDocument,
  renameDocument,
  reviewTextSuggestion,
  importMediaAsset,
  importSnapshotProject,
  exportProjectArchive,
  listMediaAssets,
  listVisualProfiles,
  mergeStoryEntities,
  openExportDirectory,
  saveStylePreset,
  saveVisualProfile,
  streamTextDraft,
  validateProjectName,
  exportImagePromptDraft,
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

  it("requests an offline example copy without a source path or network location", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: {
        project: {
          id: projectId,
          name: "Die Stadt, die den Dienstag verkaufte (Beispiel)",
          schemaVersion: 1,
        },
        documents: [],
      },
    });

    await expect(createExampleProject()).resolves.toMatchObject({
      project: { id: projectId },
    });
    expect(mockedInvoke).toHaveBeenCalledWith("create_example_project", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {},
      },
    });
  });

  it("opens native snapshot selection without exposing a path or archive bytes", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });

    await expect(importSnapshotProject()).resolves.toBeNull();
    expect(mockedInvoke).toHaveBeenCalledWith("import_snapshot_project", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {},
      },
    });
  });

  it("exports a project archive through native selection without exposing a destination", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });

    await expect(exportProjectArchive(projectId)).resolves.toBeNull();
    expect(mockedInvoke).toHaveBeenCalledWith("export_project_archive", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId },
      },
    });
  });

  it("sends an explicit deletion confirmation for exactly one project", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });

    await expect(deleteProject(projectId)).resolves.toBeUndefined();
    expect(mockedInvoke).toHaveBeenCalledWith("delete_project", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, confirmed: true },
      },
    });
  });

  it("opens only the export directory derived from a project id", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });

    await expect(openExportDirectory(projectId)).resolves.toBeUndefined();
    expect(mockedInvoke).toHaveBeenCalledWith("open_export_directory", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId },
      },
    });
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

  it("sends conflict-bound document tree mutations", async () => {
    const projectId = requestId as ProjectId;
    const documentId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const siblingId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4" as EntityId;
    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: {
        id: documentId,
        projectId,
        parentId: null,
        kind: "chapter",
        title: "Ankunft",
        position: 0,
        revision: 0,
      },
    });
    await renameDocument({
      projectId,
      documentId,
      expectedTitle: "Kapitel 1",
      title: "Ankunft",
    });
    expect(mockedInvoke).toHaveBeenLastCalledWith("rename_document", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          documentId,
          expectedTitle: "Kapitel 1",
          title: "Ankunft",
        },
      },
    });

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: null,
    });
    await reorderDocuments({
      projectId,
      parentId: null,
      expectedOrder: [documentId, siblingId],
      orderedIds: [siblingId, documentId],
    });
    expect(mockedInvoke).toHaveBeenLastCalledWith("reorder_documents", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          parentId: null,
          expectedOrder: [documentId, siblingId],
          orderedIds: [siblingId, documentId],
        },
      },
    });

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { deletedIds: [documentId, siblingId] },
    });
    await expect(
      deleteDocument({
        projectId,
        documentId,
        expectedTitle: "Ankunft",
        expectedRevision: 0,
        expectedChildIds: [siblingId],
      }),
    ).resolves.toEqual([documentId, siblingId]);
    expect(mockedInvoke).toHaveBeenLastCalledWith("delete_document", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          documentId,
          expectedTitle: "Ankunft",
          expectedRevision: 0,
          expectedChildIds: [siblingId],
        },
      },
    });
  });

  it("requests a local continuity analysis through the versioned boundary", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: [],
    });

    await expect(analyzeContinuity(projectId)).resolves.toEqual([]);
    expect(mockedInvoke).toHaveBeenCalledWith("analyze_continuity", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId },
      },
    });
  });

  it("binds a suggestion review to the scene revision", async () => {
    const projectId = requestId as ProjectId;
    const documentId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { documentId, revision: 4, changed: true, lines: [] },
    });

    await reviewTextSuggestion(projectId, documentId, 4, "Vorschlag");
    expect(mockedInvoke).toHaveBeenCalledWith("review_text_suggestion", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, documentId, expectedRevision: 4, proposedText: "Vorschlag" },
      },
    });
  });

  it("sends bounded media bytes and metadata through the versioned boundary", async () => {
    const projectId = requestId as ProjectId;
    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: [],
    });
    await expect(listMediaAssets(projectId)).resolves.toEqual([]);

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { asset: {}, thumbnailDataUrl: null, fileAvailable: false },
    });
    await importMediaAsset(projectId, "rabe.png", "Kolkrabe", ["Nacht"], [137, 80]);
    expect(mockedInvoke).toHaveBeenLastCalledWith("import_media_asset", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          fileName: "rabe.png",
          title: "Kolkrabe",
          tags: ["Nacht"],
          bytes: [137, 80],
        },
      },
    });
  });

  it("sends structured visual profiles and style presets through the versioned boundary", async () => {
    const projectId = requestId as ProjectId;
    const entityId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: [],
    });
    await expect(listVisualProfiles(projectId)).resolves.toEqual([]);

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { entityId, projectId, traits: {}, canonicalAssetId: null, exclusions: [] },
    });
    await saveVisualProfile(projectId, entityId, { Augen: "bernsteinfarben" }, null, [
      "blaue Augen",
    ]);
    expect(mockedInvoke).toHaveBeenLastCalledWith("save_visual_profile", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          entityId,
          traits: { Augen: "bernsteinfarben" },
          canonicalAssetId: null,
          exclusions: ["blaue Augen"],
        },
      },
    });

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { id: entityId, projectId, name: "Noir", visualRules: {}, applicableKinds: [] },
    });
    await saveStylePreset(projectId, null, "Noir", { Palette: "monochrom" }, ["character"]);
    expect(mockedInvoke).toHaveBeenLastCalledWith("create_style_preset", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          presetId: null,
          name: "Noir",
          visualRules: { Palette: "monochrom" },
          applicableKinds: ["character"],
        },
      },
    });
  });

  it("keeps image prompt composition, saving and export behind local typed commands", async () => {
    const projectId = requestId as ProjectId;
    const documentId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const entityId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4" as EntityId;
    const composition = {
      projectId,
      documentId,
      documentTitle: "Turm",
      documentRevision: 3,
      entityIds: [entityId],
      stylePresetId: null,
      promptText: "Kolkrabe im Abendlicht",
    };
    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: composition,
    });
    await expect(composeImagePrompt(projectId, documentId, [entityId], null)).resolves.toEqual(
      composition,
    );
    expect(mockedInvoke).toHaveBeenLastCalledWith("compose_image_prompt", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, documentId, entityIds: [entityId], stylePresetId: null },
      },
    });

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { id: entityId, name: "Turm", promptText: composition.promptText },
    });
    await createImagePromptDraft(projectId, "Turm", composition, composition.promptText);
    expect(mockedInvoke).toHaveBeenLastCalledWith("create_image_prompt_draft", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {
          projectId,
          name: "Turm",
          composition,
          promptText: composition.promptText,
        },
      },
    });

    mockedInvoke.mockResolvedValueOnce({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { fileName: `bildprompt-${entityId}.txt`, byteCount: 24 },
    });
    await exportImagePromptDraft(projectId, entityId);
    expect(mockedInvoke).toHaveBeenLastCalledWith("export_image_prompt_draft", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, draftId: entityId },
      },
    });
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

  it("sends source and target explicitly when merging Story Bible duplicates", async () => {
    const projectId = requestId as ProjectId;
    const sourceEntityId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
    const targetEntityId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4" as EntityId;
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: {
        entity: {
          id: targetEntityId,
          projectId,
          kind: "character",
          name: "Mara",
          properties: {},
        },
        outgoingRelations: [],
        incomingRelations: [],
        backlinks: [],
      },
    });

    await mergeStoryEntities(projectId, sourceEntityId, targetEntityId);

    expect(mockedInvoke).toHaveBeenCalledWith("merge_story_entities", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: { projectId, sourceEntityId, targetEntityId },
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
        providerId: "ollama",
        releaseId: "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4",
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
          providerId: "ollama",
          releaseId: "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4",
        },
      },
      onEvent: expect.any(Object),
    });
  });
});
