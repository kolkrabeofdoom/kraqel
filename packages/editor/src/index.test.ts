import { afterEach, describe, expect, it, vi } from "vitest";
import type { DocumentNode, EntityId, ProjectId } from "@kraqel/domain";
import {
  AutosaveController,
  EDITOR_DOCUMENT_VERSION,
  buildDocumentTree,
  countWords,
  type EditorDocument,
  isEditorDocument,
  plainTextFromDocument,
} from "./index";

const projectId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2" as ProjectId;
const chapterId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3" as EntityId;
const firstSceneId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4" as EntityId;
const secondSceneId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a5" as EntityId;

afterEach(() => vi.useRealTimers());

describe("editor document contract", () => {
  it("accepts versioned Tiptap JSON and derives plain text", () => {
    const document: EditorDocument = {
      version: EDITOR_DOCUMENT_VERSION,
      document: {
        type: "doc",
        content: [
          { type: "paragraph", content: [{ type: "text", text: "Die Stadt" }] },
          { type: "paragraph", content: [{ type: "text", text: "erwacht langsam." }] },
        ],
      },
    };

    expect(isEditorDocument(document)).toBe(true);
    expect(plainTextFromDocument(document)).toBe("Die Stadt erwacht langsam.");
    expect(countWords(plainTextFromDocument(document))).toBe(4);
    expect(isEditorDocument({ version: 99, document: { type: "doc" } })).toBe(false);
  });
});

describe("document tree", () => {
  it("sorts siblings by position and stable id", () => {
    const nodes: DocumentNode[] = [
      node(secondSceneId, chapterId, "Zweite Szene", 1, "scene"),
      node(firstSceneId, chapterId, "Erste Szene", 1, "scene"),
      node(chapterId, null, "Kapitel", 0, "chapter"),
    ];

    const tree = buildDocumentTree(nodes);

    expect(tree).toHaveLength(1);
    expect(tree[0]?.children.map(({ id }) => id)).toEqual([firstSceneId, secondSceneId]);
  });
});

describe("autosave controller", () => {
  it("debounces changes and exposes visible state transitions", async () => {
    vi.useFakeTimers();
    const states: string[] = [];
    const save = vi.fn(async (_value: string, revision: number) => revision + 1);
    const controller = new AutosaveController(3, save, 500);
    controller.subscribe(({ status }) => states.push(status));

    controller.schedule("eins");
    controller.schedule("zwei");
    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledOnce();
    expect(save).toHaveBeenCalledWith("zwei", 3);
    expect(controller.snapshot).toEqual({ status: "saved", revision: 4 });
    expect(states).toEqual(["saved", "dirty", "dirty", "saving", "saved"]);
  });

  it("keeps failed content dirty for an explicit retry", async () => {
    vi.useFakeTimers();
    const save = vi
      .fn<(value: string, revision: number) => Promise<number>>()
      .mockRejectedValueOnce(new Error("offline storage error"))
      .mockResolvedValueOnce(2);
    const controller = new AutosaveController(1, save, 100);

    controller.schedule("Text");
    await vi.advanceTimersByTimeAsync(100);
    expect(controller.snapshot.status).toBe("error");

    await controller.flush();
    expect(controller.snapshot).toEqual({ status: "saved", revision: 2 });
  });

  it("never replaces a newer edit when an older save fails", async () => {
    let rejectOldSave: ((reason: Error) => void) | undefined;
    const save = vi
      .fn<(value: string, revision: number) => Promise<number>>()
      .mockImplementationOnce(
        () =>
          new Promise((_resolve, reject) => {
            rejectOldSave = reject;
          }),
      )
      .mockResolvedValueOnce(1);
    const controller = new AutosaveController(0, save, 10_000);

    controller.schedule("alter Stand");
    const firstFlush = controller.flush();
    controller.schedule("neuer Stand");
    rejectOldSave?.(new Error("write failed"));
    await firstFlush;
    await controller.flush();

    expect(save).toHaveBeenNthCalledWith(1, "alter Stand", 0);
    expect(save).toHaveBeenNthCalledWith(2, "neuer Stand", 0);
    expect(controller.snapshot).toEqual({ status: "saved", revision: 1 });
  });
});

function node(
  id: EntityId,
  parentId: EntityId | null,
  title: string,
  position: number,
  kind: "chapter" | "scene",
): DocumentNode {
  return { id, projectId, parentId, kind, title, position, revision: 0 };
}
