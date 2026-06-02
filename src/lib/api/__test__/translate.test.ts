import { describe, expect, it, vi } from "vitest";
import { translate } from "../translate";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("translate", () => {
  it("maps backend unified translation result to frontend fields", async () => {
    invokeMock.mockResolvedValueOnce({
      translation: "你好，世界",
      detected_source_lang: "en",
      phonetic: null,
      meanings: [
        { partOfSpeech: "interjection", translations: ["你好"] },
      ],
      english_definitions: [],
      examples: [{ sentence: "Hello world", translation: "你好，世界" }],
      word_forms: [],
      memory_tip: "",
    });

    await expect(translate("Hello world")).resolves.toEqual({
      translation: "你好，世界",
      detectedSourceLang: "en",
      phonetic: null,
      meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
      englishDefinitions: [],
      examples: [{ sentence: "Hello world", translation: "你好，世界" }],
      wordForms: [],
      memoryTip: "",
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
