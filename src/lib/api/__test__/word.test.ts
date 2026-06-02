import { describe, expect, it, vi } from "vitest";
import { addWord } from "../word";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("addWord", () => {
  it("sends a single input object to the add_word command", async () => {
    invokeMock.mockResolvedValueOnce({
      word_id: "word-1",
      word: "hello",
      translation: "你好",
      phonetic: null,
      meanings: [
        { partOfSpeech: "interjection", translations: ["你好"] },
      ],
      english_definitions: [],
      examples: [{ sentence: "Hi.", translation: "嗨。" }],
      word_forms: [],
      memory_tip: "问候语",
      created_at: 1,
      updated_at: 1,
    });

    const result = await addWord({
      word: "hello",
      translation: "你好",
      sourceLang: "en",
      targetLang: "zh",
      phonetic: null,
      meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
      englishDefinitions: [],
      examples: [{ sentence: "Hi.", translation: "嗨。" }],
      wordForms: [],
      memoryTip: "问候语",
      tags: "",
    });

    expect(invokeMock).toHaveBeenCalledWith("add_word", {
      input: expect.objectContaining({
        word: "hello",
        translation: "你好",
        sourceLang: "en",
        targetLang: "zh",
        meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
      }),
    });
    expect(result.wordId).toBe("word-1");
    expect(result.memoryTip).toBe("问候语");
  });
});
