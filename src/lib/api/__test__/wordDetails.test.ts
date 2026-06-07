import { describe, expect, it, vi } from "vitest";
import { getWordDetail, resolveWord } from "../wordDetails";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("getWordDetail", () => {
  it("maps backend fields into the unified frontend shape", async () => {
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
      updated_at: 2,
    });

    const result = await getWordDetail("word-1");

    expect(result).toEqual({
      wordId: "word-1",
      word: "hello",
      translation: "你好",
      phonetic: null,
      meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
      englishDefinitions: [],
      examples: [{ sentence: "Hi.", translation: "嗨。" }],
      wordForms: [],
      memoryTip: "问候语",
      createdAt: 1,
      updatedAt: 2,
    });
    expect(invokeMock).toHaveBeenCalledWith("get_word_detail", {
      wordId: "word-1",
    });
  });
});

describe("resolveWord", () => {
  it("returns null wordId when the word is not in the database", async () => {
    invokeMock.mockResolvedValueOnce({
      word_id: null,
      word: "hello",
      translation: "你好",
      detected_source_lang: "en",
      source_lang: "en",
      target_lang: "zh",
      phonetic: null,
      meanings: [
        { partOfSpeech: "interjection", translations: ["你好"] },
      ],
      english_definitions: [],
      examples: [{ sentence: "Hi.", translation: "嗨。" }],
      word_forms: [],
      memory_tip: "问候语",
    });

    const result = await resolveWord("hello");

    expect(result.wordId).toBeNull();
    expect(result.translation).toBe("你好");
    expect(result.meanings).toEqual([
      { partOfSpeech: "interjection", translations: ["你好"] },
    ]);
  });

  it("returns saved wordId when the word is already in the database", async () => {
    invokeMock.mockResolvedValueOnce({
      word_id: "word-1",
      word: "hello",
      translation: "你好",
      detected_source_lang: "en",
      source_lang: "en",
      target_lang: "zh",
      phonetic: null,
      meanings: [
        { partOfSpeech: "interjection", translations: ["你好"] },
      ],
      english_definitions: [],
      examples: [{ sentence: "Hi.", translation: "嗨。" }],
      word_forms: [],
      memory_tip: "问候语",
    });

    const result = await resolveWord("hello");

    expect(result.wordId).toBe("word-1");
    expect(invokeMock).toHaveBeenCalledWith("resolve_word", { text: "hello" });
  });
});
