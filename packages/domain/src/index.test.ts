import { describe, expect, it } from "vitest";
import { isValidProjectName } from "./index";

describe("isValidProjectName", () => {
  it("accepts a concise project name", () => {
    expect(isValidProjectName("Die Stadt unter dem Meer")).toBe(true);
  });

  it("rejects blank names", () => {
    expect(isValidProjectName("   ")).toBe(false);
  });
});
