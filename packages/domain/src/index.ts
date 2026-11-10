declare const uuidBrand: unique symbol;
declare const projectIdBrand: unique symbol;
declare const entityIdBrand: unique symbol;
declare const schemaVersionBrand: unique symbol;

export type Uuid = string & { readonly [uuidBrand]: "Uuid" };
export type ProjectId = Uuid & { readonly [projectIdBrand]: "ProjectId" };
export type EntityId = Uuid & { readonly [entityIdBrand]: "EntityId" };
export type SchemaVersion = number & { readonly [schemaVersionBrand]: "SchemaVersion" };

export const CURRENT_SCHEMA_VERSION = 1 as SchemaVersion;

const canonicalUuidPattern =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function parseUuid(value: string): Uuid | null {
  return canonicalUuidPattern.test(value) ? (value.toLowerCase() as Uuid) : null;
}

export function parseProjectId(value: string): ProjectId | null {
  return parseUuid(value) as ProjectId | null;
}

export function parseEntityId(value: string): EntityId | null {
  return parseUuid(value) as EntityId | null;
}

export function parseSchemaVersion(value: number): SchemaVersion | null {
  return Number.isSafeInteger(value) && value > 0 ? (value as SchemaVersion) : null;
}

export type DocumentKind = "chapter" | "scene";
export type EntityKind = "character" | "location" | "faction" | "item";

export interface StoryEntity {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly kind: EntityKind;
  readonly name: string;
  readonly properties: Readonly<Record<string, string>>;
}

export interface StoryRelation {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly sourceEntityId: EntityId;
  readonly targetEntityId: EntityId;
  readonly kind: string;
  readonly properties: Readonly<Record<string, string>>;
}

export interface StoryBacklink {
  readonly documentId: EntityId;
  readonly documentTitle: string;
  readonly documentKind: DocumentKind;
  readonly role: string;
}

export interface StoryEntityDetails {
  readonly entity: StoryEntity;
  readonly outgoingRelations: readonly StoryRelation[];
  readonly incomingRelations: readonly StoryRelation[];
  readonly backlinks: readonly StoryBacklink[];
}

export type PlotStatus = "planned" | "draft" | "revised" | "done";

export interface Storyline {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly name: string;
  readonly color: string;
  readonly position: number;
}

export interface PlotCard {
  readonly documentId: EntityId;
  readonly title: string;
  readonly storylineId: EntityId | null;
  readonly povEntityId: EntityId | null;
  readonly status: PlotStatus;
  readonly position: number;
}

export interface PlotBoard {
  readonly storylines: readonly Storyline[];
  readonly cards: readonly PlotCard[];
}

export type TimelineMode = "relative" | "absolute";

export interface Timeline {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly name: string;
  readonly mode: TimelineMode;
  readonly position: number;
}

export interface TimelineEvent {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly timelineId: EntityId;
  readonly title: string;
  readonly description: string;
  readonly timeValue: string | null;
  readonly position: number;
  readonly documentIds: readonly EntityId[];
  readonly entityIds: readonly EntityId[];
}

export interface TimelineDetails {
  readonly timeline: Timeline;
  readonly events: readonly TimelineEvent[];
}

export type MediaKind = "image";
export type MediaProvenance = "imported";
export type AssetTargetKind = "document" | "entity" | "event";

export interface AssetLink {
  readonly targetKind: AssetTargetKind;
  readonly targetId: EntityId;
  readonly role: string;
}

export interface MediaAsset {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly kind: MediaKind;
  readonly relativePath: string;
  readonly thumbnailPath: string;
  readonly originalName: string;
  readonly title: string;
  readonly mimeType: string;
  readonly checksum: string;
  readonly byteCount: number;
  readonly width: number;
  readonly height: number;
  readonly provenance: MediaProvenance;
  readonly tags: readonly string[];
  readonly links: readonly AssetLink[];
}

export interface VisualProfile {
  readonly entityId: EntityId;
  readonly projectId: ProjectId;
  readonly traits: Readonly<Record<string, string>>;
  readonly canonicalAssetId: EntityId | null;
  readonly exclusions: readonly string[];
}

export interface StylePreset {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly name: string;
  readonly visualRules: Readonly<Record<string, string>>;
  readonly applicableKinds: readonly EntityKind[];
}

export interface ImagePromptComposition {
  readonly projectId: ProjectId;
  readonly documentId: EntityId;
  readonly documentTitle: string;
  readonly documentRevision: number;
  readonly entityIds: readonly EntityId[];
  readonly stylePresetId: EntityId | null;
  readonly promptText: string;
}

export interface ImagePromptDraft {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly name: string;
  readonly sourceDocumentId: EntityId;
  readonly sourceDocumentRevision: number;
  readonly entityIds: readonly EntityId[];
  readonly stylePresetId: EntityId | null;
  readonly promptText: string;
}

export interface Project {
  readonly id: ProjectId;
  readonly name: string;
  readonly schemaVersion: SchemaVersion;
}

export interface DocumentNode {
  readonly id: EntityId;
  readonly projectId: ProjectId;
  readonly parentId: EntityId | null;
  readonly kind: DocumentKind;
  readonly title: string;
  readonly position: number;
  readonly revision: number;
}

export function isValidProjectName(name: string): boolean {
  const trimmed = name.trim();
  return trimmed.length > 0 && trimmed.length <= 120;
}
