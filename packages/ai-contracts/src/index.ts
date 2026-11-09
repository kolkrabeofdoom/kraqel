export type AiCapability =
  | "text.generate"
  | "text.analyze"
  | "structured.extract"
  | "image.generate"
  | "image.edit"
  | "image.reference"
  | "image.map_render";

export interface ProviderDescriptor {
  readonly id: string;
  readonly displayName: string;
  readonly location: "local" | "cloud";
  readonly capabilities: readonly AiCapability[];
}

export interface ContextRelease {
  readonly approvedAt: string;
  readonly textIds: readonly string[];
  readonly assetIds: readonly string[];
  readonly metadataKeys: readonly string[];
}
