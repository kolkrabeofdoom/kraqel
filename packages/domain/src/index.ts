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
