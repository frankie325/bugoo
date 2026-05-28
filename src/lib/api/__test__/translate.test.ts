import { describe, expect, it, vi } from "vitest";
import { translate } from "../translate";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("translate", () => {
  it("sends only text to the translate_text command", async () => {
    invokeMock.mockResolvedValueOnce({
      translation: "你好，世界",
      detected_source_lang: "en",
      phonetic: null,
      part_of_speech: ["interjection"],
      definitions: ["hello world"],
      examples: [{ sentence: "Hello world", translation: "你好，世界" }],
    });

    await expect(translate("Hello world")).resolves.toEqual({
      translation: "你好，世界",
      detectedSourceLang: "en",
      phonetic: null,
      partOfSpeech: ["interjection"],
      definitions: ["hello world"],
      examples: [{ sentence: "Hello world", translation: "你好，世界" }],
    });
    expect(invokeMock.mock.calls).toStrictEqual([
      [
        "translate_text",
        {
          text: "Hello world",
        },
      ],
    ]);
  });
});
