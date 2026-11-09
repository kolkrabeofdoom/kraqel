export type EntityId = string;
export type ProjectId = string;

export type DocumentKind = "chapter" | "scene";

export interface Project {
  readonly id: ProjectId;
  readonly name: string;
  readonly schemaVersion: number;
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
