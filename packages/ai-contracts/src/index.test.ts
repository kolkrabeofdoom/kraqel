import { describe, expect, it } from "vitest";
import {
  IMAGE_CAPABILITIES,
  IMAGE_GENERATION_CONTRACT_VERSION,
  isTerminalImageGenerationStatus,
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

  it("distinguishes active and terminal job states", () => {
    expect(isTerminalImageGenerationStatus("queued")).toBe(false);
    expect(isTerminalImageGenerationStatus("running")).toBe(false);
    expect(isTerminalImageGenerationStatus("succeeded")).toBe(true);
    expect(isTerminalImageGenerationStatus("failed")).toBe(true);
    expect(isTerminalImageGenerationStatus("cancelled")).toBe(true);
  });
});
