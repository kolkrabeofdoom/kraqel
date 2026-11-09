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
  readonly traits: Readonly<Record<string, string>>;
  readonly canonicalAssetId: string | null;
  readonly exclusions: readonly string[];
}
