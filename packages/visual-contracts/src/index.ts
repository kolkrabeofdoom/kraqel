export type AssetKind = "image" | "map" | "symbol" | "reference";

export interface MediaAsset {
  readonly id: string;
  readonly projectId: string;
  readonly kind: AssetKind;
  readonly relativePath: string;
  readonly mimeType: string;
  readonly checksum: string;
  readonly width: number | null;
  readonly height: number | null;
  readonly provenance: "imported" | "generated" | "unknown";
  readonly createdAt: string;
}

export interface VisualProfile {
  readonly entityId: string;
  readonly projectId: string;
  readonly traits: Readonly<Record<string, string>>;
  readonly canonicalAssetId: string | null;
  readonly exclusions: readonly string[];
}

export interface StylePreset {
  readonly id: string;
  readonly projectId: string;
  readonly name: string;
  readonly visualRules: Readonly<Record<string, string>>;
  readonly applicableKinds: readonly ("character" | "location" | "faction" | "item")[];
}

export interface ImagePromptComposition {
  readonly projectId: string;
  readonly documentId: string;
  readonly documentTitle: string;
  readonly documentRevision: number;
  readonly entityIds: readonly string[];
  readonly stylePresetId: string | null;
  readonly promptText: string;
}

export interface ImagePromptDraft {
  readonly id: string;
  readonly projectId: string;
  readonly name: string;
  readonly sourceDocumentId: string;
  readonly sourceDocumentRevision: number;
  readonly entityIds: readonly string[];
  readonly stylePresetId: string | null;
  readonly promptText: string;
}
