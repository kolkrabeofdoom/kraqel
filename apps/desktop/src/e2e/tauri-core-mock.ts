import type {
  DocumentKind,
  DocumentNode,
  EntityId,
  EntityKind,
  PlotBoard,
  PlotCard,
  PlotStatus,
  Project,
  StoryEntity,
  StoryEntityDetails,
  StoryRelation,
  Storyline,
  Timeline,
  TimelineDetails,
  TimelineEvent,
  TimelineMode,
} from "@kraqel/domain";
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
  storyEntities: StoryEntity[];
  storyRelations: StoryRelation[];
  storyLinks: StoryLink[];
  storylines: Storyline[];
  plotCards: PlotCard[];
  timelines: Timeline[];
  timelineEvents: TimelineEvent[];
}

interface StoryLink {
  documentId: EntityId;
  entityId: EntityId;
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
        storyEntities: [],
        storyRelations: [],
        storyLinks: [],
        storylines: [],
        plotCards: [newPlotCard(scene, 0)],
        timelines: [],
        timelineEvents: [],
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
      if (node.kind === "scene")
        current.plotCards.push(newPlotCard(node, current.plotCards.length));
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
      current.storyLinks = current.storyLinks.filter(({ documentId }) => !deleted.has(documentId));
      current.plotCards = current.plotCards.filter(({ documentId }) => !deleted.has(documentId));
      current.timelineEvents = current.timelineEvents.map((item) => ({
        ...item,
        documentIds: item.documentIds.filter((id) => !deleted.has(id)),
      }));
      compactPositions(current);
      writeState(state);
      return { deletedIds };
    }
    case "list_story_entities": {
      const current = findProject(state, payload.projectId);
      const query = String(payload.query).trim().toLocaleLowerCase();
      const kind = payload.kind as EntityKind | null;
      return current.storyEntities
        .filter((entity) => kind === null || entity.kind === kind)
        .filter((entity) => {
          if (!query) return true;
          const values = [entity.name, ...Object.entries(entity.properties).flat()];
          return values.some((value) => value.toLocaleLowerCase().includes(query));
        })
        .sort(compareStoryEntities);
    }
    case "create_story_entity": {
      const current = findProject(state, payload.projectId);
      const entity: StoryEntity = {
        id: crypto.randomUUID() as EntityId,
        projectId: current.project.id,
        kind: payload.kind as EntityKind,
        name: String(payload.name).trim(),
        properties: structuredClone(payload.properties) as Readonly<Record<string, string>>,
      };
      current.storyEntities.push(entity);
      writeState(state);
      return entity;
    }
    case "update_story_entity": {
      const current = findProject(state, payload.projectId);
      const index = current.storyEntities.findIndex(({ id }) => id === payload.entityId);
      if (index < 0) throw new Error("Unknown E2E story entity");
      const entity: StoryEntity = {
        id: payload.entityId as EntityId,
        projectId: current.project.id,
        kind: payload.kind as EntityKind,
        name: String(payload.name).trim(),
        properties: structuredClone(payload.properties) as Readonly<Record<string, string>>,
      };
      current.storyEntities[index] = entity;
      writeState(state);
      return entity;
    }
    case "load_story_entity":
      return storyEntityDetails(findProject(state, payload.projectId), payload.entityId);
    case "delete_story_entity": {
      const current = findProject(state, payload.projectId);
      current.storyEntities = current.storyEntities.filter(({ id }) => id !== payload.entityId);
      current.storyRelations = current.storyRelations.filter(
        ({ sourceEntityId, targetEntityId }) =>
          sourceEntityId !== payload.entityId && targetEntityId !== payload.entityId,
      );
      current.storyLinks = current.storyLinks.filter(
        ({ entityId }) => entityId !== payload.entityId,
      );
      writeState(state);
      return null;
    }
    case "create_story_relation": {
      const current = findProject(state, payload.projectId);
      findStoryEntity(current, payload.sourceEntityId);
      findStoryEntity(current, payload.targetEntityId);
      const relation: StoryRelation = {
        id: crypto.randomUUID() as EntityId,
        projectId: current.project.id,
        sourceEntityId: payload.sourceEntityId as EntityId,
        targetEntityId: payload.targetEntityId as EntityId,
        kind: String(payload.kind).trim(),
        properties: structuredClone(payload.properties) as Readonly<Record<string, string>>,
      };
      current.storyRelations.push(relation);
      writeState(state);
      return relation;
    }
    case "delete_story_relation": {
      const current = findProject(state, payload.projectId);
      current.storyRelations = current.storyRelations.filter(({ id }) => id !== payload.relationId);
      writeState(state);
      return null;
    }
    case "link_story_entity": {
      const current = findProject(state, payload.projectId);
      findStoredDocument(current, payload.documentId);
      findStoryEntity(current, payload.entityId);
      if (
        !current.storyLinks.some(
          ({ documentId, entityId }) =>
            documentId === payload.documentId && entityId === payload.entityId,
        )
      ) {
        current.storyLinks.push({
          documentId: payload.documentId as EntityId,
          entityId: payload.entityId as EntityId,
        });
      }
      writeState(state);
      return storyEntityDetails(current, payload.entityId);
    }
    case "get_plot_board":
      return plotBoard(findProject(state, payload.projectId));
    case "create_storyline": {
      const current = findProject(state, payload.projectId);
      const storyline: Storyline = {
        id: crypto.randomUUID() as EntityId,
        projectId: current.project.id,
        name: String(payload.name).trim(),
        color: String(payload.color),
        position: current.storylines.length,
      };
      current.storylines.push(storyline);
      writeState(state);
      return storyline;
    }
    case "delete_storyline": {
      const current = findProject(state, payload.projectId);
      current.storylines = current.storylines.filter(({ id }) => id !== payload.storylineId);
      current.plotCards = current.plotCards.map((card) =>
        card.storylineId === payload.storylineId ? { ...card, storylineId: null } : card,
      );
      writeState(state);
      return null;
    }
    case "update_plot_card": {
      const current = findProject(state, payload.projectId);
      const index = current.plotCards.findIndex(
        ({ documentId }) => documentId === payload.documentId,
      );
      if (index < 0) throw new Error("Unknown E2E plot card");
      current.plotCards[index] = {
        ...current.plotCards[index]!,
        status: payload.status as PlotStatus,
        storylineId: (payload.storylineId ?? null) as EntityId | null,
        povEntityId: (payload.povEntityId ?? null) as EntityId | null,
      };
      writeState(state);
      return null;
    }
    case "reorder_plot_cards": {
      const current = findProject(state, payload.projectId);
      reorderByIds(current.plotCards, payload.orderedIds as EntityId[], (item) => item.documentId);
      writeState(state);
      return null;
    }
    case "list_timelines":
      return [...findProject(state, payload.projectId).timelines].sort(byPosition);
    case "create_timeline": {
      const current = findProject(state, payload.projectId);
      const timeline: Timeline = {
        id: crypto.randomUUID() as EntityId,
        projectId: current.project.id,
        name: String(payload.name).trim(),
        mode: payload.mode as TimelineMode,
        position: current.timelines.length,
      };
      current.timelines.push(timeline);
      writeState(state);
      return timeline;
    }
    case "delete_timeline": {
      const current = findProject(state, payload.projectId);
      current.timelines = current.timelines.filter(({ id }) => id !== payload.timelineId);
      current.timelineEvents = current.timelineEvents.filter(
        ({ timelineId }) => timelineId !== payload.timelineId,
      );
      writeState(state);
      return null;
    }
    case "load_timeline":
      return timelineDetails(findProject(state, payload.projectId), payload.timelineId);
    case "create_timeline_event": {
      const current = findProject(state, payload.projectId);
      const timelineId = payload.timelineId as EntityId;
      findTimeline(current, timelineId);
      const event: TimelineEvent = {
        id: crypto.randomUUID() as EntityId,
        projectId: current.project.id,
        timelineId,
        title: String(payload.title).trim(),
        description: String(payload.description),
        timeValue: (payload.timeValue ?? null) as string | null,
        position: current.timelineEvents.filter((item) => item.timelineId === timelineId).length,
        documentIds: [],
        entityIds: [],
      };
      current.timelineEvents.push(event);
      writeState(state);
      return event;
    }
    case "delete_timeline_event": {
      const current = findProject(state, payload.projectId);
      current.timelineEvents = current.timelineEvents.filter(({ id }) => id !== payload.eventId);
      writeState(state);
      return null;
    }
    case "reorder_timeline_events": {
      const current = findProject(state, payload.projectId);
      const timelineId = payload.parentId as EntityId;
      const events = current.timelineEvents.filter((item) => item.timelineId === timelineId);
      reorderByIds(events, payload.orderedIds as EntityId[], (item) => item.id);
      current.timelineEvents = current.timelineEvents.map(
        (item) => events.find(({ id }) => id === item.id) ?? item,
      );
      writeState(state);
      return null;
    }
    case "link_timeline_event": {
      const current = findProject(state, payload.projectId);
      const index = current.timelineEvents.findIndex(({ id }) => id === payload.eventId);
      if (index < 0) throw new Error("Unknown E2E timeline event");
      const item = current.timelineEvents[index]!;
      const documentId = (payload.documentId ?? null) as EntityId | null;
      const entityId = (payload.entityId ?? null) as EntityId | null;
      if (documentId) findStoredDocument(current, documentId);
      if (entityId) findStoryEntity(current, entityId);
      current.timelineEvents[index] = {
        ...item,
        documentIds:
          documentId && !item.documentIds.includes(documentId)
            ? [...item.documentIds, documentId]
            : item.documentIds,
        entityIds:
          entityId && !item.entityIds.includes(entityId)
            ? [...item.entityIds, entityId]
            : item.entityIds,
      };
      writeState(state);
      return timelineDetails(current, payload.timelineId);
    }
    default:
      throw new Error(`Command ${command} is not available in the browser E2E adapter`);
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

function newPlotCard(node: DocumentNode, position: number): PlotCard {
  return {
    documentId: node.id,
    title: node.title,
    storylineId: null,
    povEntityId: null,
    status: "planned",
    position,
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

function findStoryEntity(current: TestProject, id: unknown): StoryEntity {
  const entity = current.storyEntities.find((candidate) => candidate.id === id);
  if (!entity) throw new Error("Unknown E2E story entity");
  return entity;
}

function findTimeline(current: TestProject, id: unknown): Timeline {
  const timeline = current.timelines.find((candidate) => candidate.id === id);
  if (!timeline) throw new Error("Unknown E2E timeline");
  return timeline;
}

function plotBoard(current: TestProject): PlotBoard {
  const titles = new Map(current.documents.map(({ node }) => [node.id, node.title]));
  return {
    storylines: [...current.storylines].sort(byPosition),
    cards: [...current.plotCards]
      .filter(({ documentId }) => titles.has(documentId))
      .sort(byPosition)
      .map((card) => ({ ...card, title: titles.get(card.documentId)! })),
  };
}

function timelineDetails(current: TestProject, id: unknown): TimelineDetails {
  const timeline = findTimeline(current, id);
  return {
    timeline,
    events: [...current.timelineEvents]
      .filter(({ timelineId }) => timelineId === timeline.id)
      .sort(byPosition),
  };
}

function byPosition(left: { readonly position: number }, right: { readonly position: number }) {
  return left.position - right.position;
}

function reorderByIds<T extends { readonly position: number }>(
  items: T[],
  orderedIds: readonly EntityId[],
  idOf: (item: T) => EntityId,
): void {
  if (
    items.length !== orderedIds.length ||
    items.some((item) => !orderedIds.includes(idOf(item)))
  ) {
    throw new Error("Invalid E2E order");
  }
  const positions = new Map(orderedIds.map((id, position) => [id, position]));
  items.forEach((item, index) => {
    items[index] = { ...item, position: positions.get(idOf(item))! };
  });
}

function storyEntityDetails(current: TestProject, id: unknown): StoryEntityDetails {
  const entity = findStoryEntity(current, id);
  return {
    entity,
    outgoingRelations: current.storyRelations.filter(
      ({ sourceEntityId }) => sourceEntityId === entity.id,
    ),
    incomingRelations: current.storyRelations.filter(
      ({ targetEntityId }) => targetEntityId === entity.id,
    ),
    backlinks: current.storyLinks
      .filter(({ entityId }) => entityId === entity.id)
      .map(({ documentId }) => {
        const document = findStoredDocument(current, documentId).node;
        return {
          documentId: document.id,
          documentTitle: document.title,
          documentKind: document.kind,
          role: "reference",
        };
      }),
  };
}

function compareStoryEntities(left: StoryEntity, right: StoryEntity): number {
  return left.name.localeCompare(right.name, "de") || left.id.localeCompare(right.id);
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
