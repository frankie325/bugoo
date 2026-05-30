import { describe, expect, it, vi } from "vitest";
import { getTranslationLanguages } from "../translationLanguages";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("getTranslationLanguages", () => {
  it("maps camelCase language fields returned by Tauri", async () => {
    invokeMock.mockResolvedValueOnce({
      sourceLanguages: [{ code: "auto", name: "Auto Detect" }],
      targetLanguages: [{ code: "zh", name: "Chinese" }],
    });

    await expect(getTranslationLanguages("local")).resolves.toEqual({
      sourceLanguages: [{ code: "auto", name: "Auto Detect" }],
      targetLanguages: [{ code: "zh", name: "Chinese" }],
    });
  });
});
