import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  COMMAND_CONTRACT_VERSION,
  KraqelCommandError,
  getApplicationStatus,
  validateProjectName,
} from "./commands";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);
const requestId = "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2";

beforeEach(() => {
  vi.stubGlobal("crypto", { randomUUID: () => requestId });
  mockedInvoke.mockReset();
});

describe("typed Tauri commands", () => {
  it("sends a versioned request and accepts the matching response", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { version: "0.1.0", schemaVersion: 1, offlineReady: true },
    });

    await expect(getApplicationStatus()).resolves.toEqual({
      version: "0.1.0",
      schemaVersion: 1,
      offlineReady: true,
    });
    expect(mockedInvoke).toHaveBeenCalledWith("application_status", {
      request: {
        contractVersion: COMMAND_CONTRACT_VERSION,
        requestId,
        payload: {},
      },
    });
  });

  it("does not echo a supplied project name in the result", async () => {
    mockedInvoke.mockResolvedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      data: { valid: true },
    });

    const result = await validateProjectName("Geheimes Manuskript");

    expect(result).toEqual({ valid: true });
    expect(JSON.stringify(result)).not.toContain("Geheimes Manuskript");
  });

  it("maps unknown rejections to a sanitized internal error", async () => {
    mockedInvoke.mockRejectedValue(new Error("D:\\private\\manuscript.md"));

    const error = await getApplicationStatus().catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(KraqelCommandError);
    expect(error).toMatchObject({ code: "INTERNAL", retryable: false });
    expect(String(error)).not.toContain("private");
  });

  it("does not trust an error message received across the IPC boundary", async () => {
    mockedInvoke.mockRejectedValue({
      contractVersion: COMMAND_CONTRACT_VERSION,
      requestId,
      code: "INVALID_REQUEST",
      message: "D:\\private\\manuscript.md",
      retryable: false,
    });

    const error = await getApplicationStatus().catch((reason: unknown) => reason);

    expect(error).toMatchObject({ code: "INVALID_REQUEST", retryable: false });
    expect(String(error)).not.toContain("private");
  });
});
