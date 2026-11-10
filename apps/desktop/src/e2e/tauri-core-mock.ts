import type { DocumentKind, DocumentNode, Project } from "@kraqel/domain";
import { plainTextFromDocument, type EditorDocument } from "@kraqel/editor";

const STORAGE_KEY = "kraqel-e2e-command-state-v1";

interface StoredDocument {
  node: DocumentNode;
  content: EditorDocument;
  plainText: string;
}

interface TestProject {
  project: Project;
  documents: StoredDocument[];
}

interface TestState {
  projects: TestProject[];
}

interface CommandRequest {
  contractVersion: number;
  requestId: string;
  payload: Record<string, unknown>;
}

interface InvokeArgs {
  request?: CommandRequest;
}

export class Channel<T> {
  onmessage: (message: T) => void = () => undefined;

  toJSON(): string {
    return "__KRAQEL_E2E_CHANNEL__";
  }
}

export async function invoke<T>(command: string, args: InvokeArgs = {}): Promise<T> {
  const request = args.request;
  if (!request || request.contractVersion !== 1) throw new Error("Invalid E2E command request");

  const data = execute(command, request.payload);
  return {
    contractVersion: 1,
    requestId: request.requestId,
    data,
  } as T;
}

function execute(command: string, payload: Record<string, unknown>): unknown {
  const state = readState();

  switch (command) {
    case "list_projects":
      return state.projects.map(({ project }) => ({ id: project.id, name: project.name }));
    case "create_project": {
      const projectId = crypto.randomUUID() as Project["id"];
      const project: Project = {
        id: projectId,
        name: String(payload.name).trim(),
        schemaVersion: 1 as Project["schemaVersion"],
      };
      const chapter = newNode(projectId, null, "chapter", "Kapitel 1", 0);
      const scene = newNode(projectId, chapter.id, "scene", "Szene 1", 0);
      const created: TestProject = {
        project,
        documents: [storedDocument(chapter), storedDocument(scene)],
      };
      state.projects.push(created);
      writeState(state);
      return workspace(created);
    }
    case "open_workspace":
      return workspace(findProject(state, payload.projectId));
    case "create_document": {
      const current = findProject(state, payload.projectId);
      const parentId = (payload.parentId ?? null) as DocumentNode["parentId"];
      const position = current.documents.filter(({ node }) => node.parentId === parentId).length;
      const node = newNode(
        current.project.id,
        parentId,
        payload.kind as DocumentKind,
        String(payload.title),
        position,
      );
      current.documents.push(storedDocument(node));
      writeState(state);
      return node;
    }
    case "load_document": {
      const current = findStoredDocument(findProject(state, payload.projectId), payload.documentId);
      return structuredClone(current);
    }
    case "save_document": {
      const current = findStoredDocument(findProject(state, payload.projectId), payload.documentId);
      if (current.node.revision !== payload.expectedRevision) throw new Error("Revision conflict");
      current.content = structuredClone(payload.content) as EditorDocument;
      current.plainText = plainTextFromDocument(current.content);
      current.node = { ...current.node, revision: current.node.revision + 1 };
      writeState(state);
      return { revision: current.node.revision };
    }
    case "rename_document": {
      const current = findStoredDocument(findProject(state, payload.projectId), payload.documentId);
      if (current.node.title !== payload.expectedTitle) throw new Error("Title conflict");
      current.node = { ...current.node, title: String(payload.title).trim() };
      writeState(state);
      return current.node;
    }
    case "reorder_documents": {
      const current = findProject(state, payload.projectId);
      const expected = payload.expectedOrder as string[];
      const ordered = payload.orderedIds as string[];
      const siblings = current.documents
        .filter(({ node }) => node.parentId === (payload.parentId ?? null))
        .sort((left, right) => left.node.position - right.node.position);
      if (siblings.map(({ node }) => node.id).join("\0") !== expected.join("\0")) {
        throw new Error("Order conflict");
      }
      for (const [position, id] of ordered.entries()) {
        const document = findStoredDocument(current, id);
        document.node = { ...document.node, position };
      }
      writeState(state);
      return null;
    }
    case "delete_document": {
      const current = findProject(state, payload.projectId);
      const target = findStoredDocument(current, payload.documentId);
      const childIds = current.documents
        .filter(({ node }) => node.parentId === target.node.id)
        .sort((left, right) => left.node.position - right.node.position)
        .map(({ node }) => node.id);
      if (
        target.node.title !== payload.expectedTitle ||
        target.node.revision !== payload.expectedRevision ||
        childIds.join("\0") !== (payload.expectedChildIds as string[]).join("\0")
      ) {
        throw new Error("Delete conflict");
      }
      const deletedIds = [target.node.id, ...childIds];
      const deleted = new Set(deletedIds);
      current.documents = current.documents.filter(({ node }) => !deleted.has(node.id));
      compactPositions(current);
      writeState(state);
      return { deletedIds };
    }
    default:
      throw new Error(`Command ${command} is not available in the M1 E2E adapter`);
  }
}

function newNode(
  projectId: Project["id"],
  parentId: DocumentNode["parentId"],
  kind: DocumentKind,
  title: string,
  position: number,
): DocumentNode {
  return {
    id: crypto.randomUUID() as DocumentNode["id"],
    projectId,
    parentId,
    kind,
    title,
    position,
    revision: 0,
  };
}

function storedDocument(node: DocumentNode): StoredDocument {
  return {
    node,
    content: { version: 1, document: { type: "doc", content: [{ type: "paragraph" }] } },
    plainText: "",
  };
}

function workspace(current: TestProject) {
  return {
    project: current.project,
    documents: current.documents.map(({ node }) => node),
  };
}

function findProject(state: TestState, id: unknown): TestProject {
  const current = state.projects.find(({ project }) => project.id === id);
  if (!current) throw new Error("Unknown E2E project");
  return current;
}

function findStoredDocument(current: TestProject, id: unknown): StoredDocument {
  const document = current.documents.find(({ node }) => node.id === id);
  if (!document) throw new Error("Unknown E2E document");
  return document;
}

function compactPositions(current: TestProject): void {
  const parentIds = new Set(current.documents.map(({ node }) => node.parentId));
  for (const parentId of parentIds) {
    current.documents
      .filter(({ node }) => node.parentId === parentId)
      .sort((left, right) => left.node.position - right.node.position)
      .forEach((document, position) => {
        document.node = { ...document.node, position };
      });
  }
}

function readState(): TestState {
  const serialized = localStorage.getItem(STORAGE_KEY);
  return serialized ? (JSON.parse(serialized) as TestState) : { projects: [] };
}

function writeState(state: TestState): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}
