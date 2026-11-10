export const IMAGE_GENERATION_CONTRACT_VERSION = 1 as const;

export const IMAGE_CAPABILITIES = [
  "image.generate",
  "image.edit",
  "image.reference",
  "image.map_render",
] as const;

export type ImageCapability = (typeof IMAGE_CAPABILITIES)[number];
export type AiCapability =
  "text.generate" | "text.analyze" | "structured.extract" | ImageCapability;

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
  readonly providerId: string;
  readonly providerDisplayName: string;
  readonly providerLocation: "local" | "cloud";
  readonly model: string;
  readonly instruction: string;
  readonly metadata: Readonly<Record<string, string>>;
  readonly releaseId: string;
}

export interface CloudProviderStatus {
  readonly provider: ProviderDescriptor;
  readonly credentialConfigured: boolean;
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

export interface ImageModelLimits {
  readonly maxPromptCharacters: number;
  readonly maxReferenceImages: number;
  readonly supportedWidths: readonly number[];
  readonly supportedHeights: readonly number[];
  readonly maxOutputs: number;
  readonly supportsSeed: boolean;
}

export interface ImageModelDescriptor {
  readonly id: string;
  readonly displayName: string;
  readonly capabilities: readonly ImageCapability[];
  readonly limits: ImageModelLimits;
}

export interface ImageProviderDescriptor {
  readonly id: string;
  readonly displayName: string;
  readonly location: "local" | "cloud";
  readonly available: boolean;
  readonly models: readonly ImageModelDescriptor[];
  readonly unavailableReason: string | null;
}

export interface ImageContextRelease {
  readonly promptDraftId: string;
  readonly promptText: string;
  readonly referenceAssetIds: readonly string[];
  readonly approved: boolean;
}

export interface ImageGenerationSettings {
  readonly width: number;
  readonly height: number;
  readonly outputCount: number;
  readonly seed: number | null;
}

export interface ImageGenerationRequest {
  readonly contractVersion: typeof IMAGE_GENERATION_CONTRACT_VERSION;
  readonly jobId: string;
  readonly projectId: string;
  readonly providerId: string;
  readonly modelId: string;
  readonly capability: ImageCapability;
  readonly context: ImageContextRelease;
  readonly settings: ImageGenerationSettings;
}

export type ImageGenerationJobStatus = "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface ImageGenerationProvenance {
  readonly providerId: string;
  readonly modelId: string;
  readonly capability: ImageCapability;
  readonly seed: number | null;
  readonly generatedAt: string;
}

export type ImageGenerationErrorCode =
  | "INVALID_REQUEST"
  | "UNSUPPORTED_CAPABILITY"
  | "PROVIDER_UNAVAILABLE"
  | "MODEL_UNAVAILABLE"
  | "CONTEXT_NOT_APPROVED"
  | "INVALID_STATE"
  | "OUTPUT_REJECTED"
  | "INTERNAL";

export interface ImageGenerationError {
  readonly code: ImageGenerationErrorCode;
  readonly message: string;
  readonly retryable: boolean;
}

export interface ImageGenerationJob {
  readonly contractVersion: typeof IMAGE_GENERATION_CONTRACT_VERSION;
  readonly id: string;
  readonly projectId: string;
  readonly providerId: string;
  readonly modelId: string;
  readonly capability: ImageCapability;
  readonly promptDraftId: string;
  readonly promptText: string;
  readonly referenceAssetIds: readonly string[];
  readonly settings: ImageGenerationSettings;
  readonly status: ImageGenerationJobStatus;
  readonly error: ImageGenerationError | null;
  readonly provenance: ImageGenerationProvenance | null;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export function isTerminalImageGenerationStatus(status: ImageGenerationJobStatus): boolean {
  return status === "succeeded" || status === "failed" || status === "cancelled";
}
