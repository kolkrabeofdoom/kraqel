import { describe, expect, it } from "vitest";
import {
  CURRENT_SCHEMA_VERSION,
  isValidProjectName,
  parseProjectId,
  parseSchemaVersion,
  parseUuid,
} from "./index";

describe("isValidProjectName", () => {
  it("accepts a concise project name", () => {
    expect(isValidProjectName("Die Stadt unter dem Meer")).toBe(true);
  });

  it("rejects blank names", () => {
    expect(isValidProjectName("   ")).toBe(false);
  });
});

describe("stable identifiers", () => {
  it("accepts and normalizes canonical UUIDs", () => {
    expect(parseUuid("018F47EA-9B2D-7CC1-98E8-B5F4F6B741A2")).toBe(
      "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2",
    );
    expect(parseProjectId("018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2")).not.toBeNull();
  });

  it("rejects arbitrary strings and invalid schema versions", () => {
    expect(parseUuid("chapter-one")).toBeNull();
    expect(parseSchemaVersion(0)).toBeNull();
    expect(parseSchemaVersion(1.5)).toBeNull();
    expect(parseSchemaVersion(1)).toBe(CURRENT_SCHEMA_VERSION);
  });
});
