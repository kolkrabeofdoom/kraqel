import type { JSONContent } from "@tiptap/core";
import type { DocumentNode, EntityId } from "@kraqel/domain";

export const EDITOR_DOCUMENT_VERSION = 1 as const;

export interface EditorDocument {
  readonly version: typeof EDITOR_DOCUMENT_VERSION;
  readonly document: JSONContent;
}

export const EMPTY_EDITOR_DOCUMENT: EditorDocument = {
  version: EDITOR_DOCUMENT_VERSION,
  document: { type: "doc", content: [{ type: "paragraph" }] },
};

export interface DocumentTreeNode extends DocumentNode {
  readonly children: readonly DocumentTreeNode[];
}

export type AutosaveStatus = "saved" | "dirty" | "saving" | "error";

export interface AutosaveSnapshot {
  readonly status: AutosaveStatus;
  readonly revision: number;
}

export type AutosaveListener = (snapshot: AutosaveSnapshot) => void;

/** Serialisiert entprellte Speichervorgänge und bewahrt neuere schmutzige Stände. */
export class AutosaveController<T> {
  readonly #delay: number;
  readonly #save: (value: T, expectedRevision: number) => Promise<number>;
  readonly #listeners = new Set<AutosaveListener>();
  #status: AutosaveStatus = "saved";
  #revision: number;
  #pending: T | undefined;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #inFlight: Promise<void> | undefined;
  #disposed = false;

  constructor(
    revision: number,
    save: (value: T, expectedRevision: number) => Promise<number>,
    delay = 700,
  ) {
    this.#revision = revision;
    this.#save = save;
    this.#delay = delay;
  }

  get snapshot(): AutosaveSnapshot {
    return { status: this.#status, revision: this.#revision };
  }

  subscribe(listener: AutosaveListener): () => void {
    this.#listeners.add(listener);
    listener(this.snapshot);
    return () => this.#listeners.delete(listener);
  }

  schedule(value: T): void {
    if (this.#disposed) return;
    this.#pending = value;
    this.#setStatus("dirty");
    this.#armTimer();
  }

  async flush(): Promise<AutosaveSnapshot> {
    if (this.#disposed) return this.snapshot;
    this.#clearTimer();

    if (this.#inFlight) {
      await this.#inFlight;
      if (this.#pending !== undefined) return this.flush();
      return this.snapshot;
    }
    if (this.#pending === undefined) return this.snapshot;

    const value = this.#pending;
    this.#pending = undefined;
    this.#setStatus("saving");
    this.#inFlight = this.#performSave(value);
    await this.#inFlight;
    this.#inFlight = undefined;

    if (this.#pending !== undefined && this.#status !== "error") {
      this.#setStatus("dirty");
      this.#armTimer();
    }
    return this.snapshot;
  }

  dispose(): void {
    this.#disposed = true;
    this.#clearTimer();
    this.#listeners.clear();
  }

  async #performSave(value: T): Promise<void> {
    try {
      this.#revision = await this.#save(value, this.#revision);
      this.#setStatus(this.#pending === undefined ? "saved" : "dirty");
    } catch {
      if (this.#pending === undefined) this.#pending = value;
      this.#setStatus("error");
    }
  }

  #armTimer(): void {
    this.#clearTimer();
    this.#timer = setTimeout(() => {
      void this.flush();
    }, this.#delay);
  }

  #clearTimer(): void {
    if (this.#timer !== undefined) clearTimeout(this.#timer);
    this.#timer = undefined;
  }

  #setStatus(status: AutosaveStatus): void {
    this.#status = status;
    const snapshot = this.snapshot;
    for (const listener of this.#listeners) listener(snapshot);
  }
}

export function isEditorDocument(value: unknown): value is EditorDocument {
  if (!isRecord(value) || value.version !== EDITOR_DOCUMENT_VERSION) return false;
  if (!isRecord(value.document) || value.document.type !== "doc") return false;
  return value.document.content === undefined || Array.isArray(value.document.content);
}

export function buildDocumentTree(nodes: readonly DocumentNode[]): readonly DocumentTreeNode[] {
  const sorted = [...nodes].sort(compareNodes);
  const children = new Map<EntityId | null, DocumentNode[]>();

  for (const node of sorted) {
    const siblings = children.get(node.parentId) ?? [];
    siblings.push(node);
    children.set(node.parentId, siblings);
  }

  const expand = (node: DocumentNode, ancestors: ReadonlySet<EntityId>): DocumentTreeNode => {
    if (ancestors.has(node.id)) return { ...node, children: [] };
    const nextAncestors = new Set(ancestors).add(node.id);
    return {
      ...node,
      children: (children.get(node.id) ?? []).map((child) => expand(child, nextAncestors)),
    };
  };

  return (children.get(null) ?? []).map((node) => expand(node, new Set()));
}

export function plainTextFromDocument(document: EditorDocument): string {
  const fragments: string[] = [];
  collectText(document.document, fragments);
  return fragments.join(" ").replace(/\s+/g, " ").trim();
}

export function countWords(text: string): number {
  const normalized = text.trim();
  return normalized === "" ? 0 : normalized.split(/\s+/u).length;
}

function compareNodes(left: DocumentNode, right: DocumentNode): number {
  return left.position - right.position || left.id.localeCompare(right.id);
}

function collectText(node: JSONContent, fragments: string[]): void {
  if (typeof node.text === "string") fragments.push(node.text);
  for (const child of node.content ?? []) collectText(child, fragments);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
