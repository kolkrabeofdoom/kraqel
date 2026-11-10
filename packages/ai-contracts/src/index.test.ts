import { describe, expect, it } from "vitest";
import {
  IMAGE_CAPABILITIES,
  IMAGE_GENERATION_CONTRACT_VERSION,
  isTerminalImageGenerationStatus,
  type ImageGenerationRequest,
} from "./index";

describe("image generation contracts", () => {
  it("keeps the version and wire capabilities stable", () => {
    expect(IMAGE_GENERATION_CONTRACT_VERSION).toBe(1);
    expect(IMAGE_CAPABILITIES).toEqual([
      "image.generate",
      "image.edit",
      "image.reference",
      "image.map_render",
    ]);
  });

  it("represents an explicitly released request without provider assumptions", () => {
    const request: ImageGenerationRequest = {
      contractVersion: IMAGE_GENERATION_CONTRACT_VERSION,
      jobId: "00000000-0000-0000-0000-000000000001",
      projectId: "00000000-0000-0000-0000-000000000002",
      providerId: "local-mock",
      modelId: "mock-v1",
      capability: "image.generate",
      context: {
        promptDraftId: "00000000-0000-0000-0000-000000000003",
        promptText: "Kolkrabe über der solidarischen Stadt",
        referenceAssetIds: [],
        approved: true,
      },
      settings: { width: 1024, height: 1024, outputCount: 1, seed: 42 },
    };

    expect(request.context.approved).toBe(true);
    expect(request.providerId).toBe("local-mock");
  });

  it("distinguishes active and terminal job states", () => {
    expect(isTerminalImageGenerationStatus("queued")).toBe(false);
    expect(isTerminalImageGenerationStatus("running")).toBe(false);
    expect(isTerminalImageGenerationStatus("succeeded")).toBe(true);
    expect(isTerminalImageGenerationStatus("failed")).toBe(true);
    expect(isTerminalImageGenerationStatus("cancelled")).toBe(true);
  });
});
