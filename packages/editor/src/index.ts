export const EDITOR_DOCUMENT_VERSION = 1;

export interface EditorSnapshot {
  readonly version: typeof EDITOR_DOCUMENT_VERSION;
  readonly document: Readonly<Record<string, unknown>>;
}
