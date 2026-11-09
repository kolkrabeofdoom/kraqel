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

export interface TextModel {
  readonly id: string;
}

export interface TextProviderStatus {
  readonly provider: ProviderDescriptor;
  readonly available: boolean;
  readonly models: readonly TextModel[];
}

export interface AiContextPreview {
  readonly documentId: string;
  readonly documentTitle: string;
  readonly revision: number;
  readonly text: string;
}

export type TextStreamEvent =
  | {
      readonly type: "started";
      readonly providerId: string;
      readonly model: string;
    }
  | {
      readonly type: "delta";
      readonly text: string;
    }
  | {
      readonly type: "completed";
      readonly provenance: string;
    };
