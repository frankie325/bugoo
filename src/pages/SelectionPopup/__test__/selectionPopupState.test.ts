import { describe, expect, it } from "vitest";
import { getSelectionPopupState } from "../selectionPopupState";
import type { ResolvedWord } from "../../../lib/api";

const resolvedWord: ResolvedWord = {
  wordId: null,
  word: "serendipity",
  translation: "意外发现的好运",
  detectedSourceLang: "en",
  sourceLang: "en",
  targetLang: "zh",
  phonetic: "/ˌser.ənˈdɪp.ə.ti/",
  meanings: [{ partOfSpeech: "noun", translations: ["意外发现的好运"] }],
  englishDefinitions: [],
  examples: [{ sentence: "I found this job by pure serendipity.", translation: "我纯粹是意外得到这份工作的。" }],
  wordForms: [],
  memoryTip: "",
};

describe("getSelectionPopupState", () => {
  it("returns empty when selected text is blank", () => {
    expect(getSelectionPopupState({ text: " ", resolvedWord: null, isResolving: false, resolveError: null })).toBe("empty");
  });

  it("returns tooLong when selected text exceeds the popup limit", () => {
    expect(getSelectionPopupState({ text: "a".repeat(51), resolvedWord: null, isResolving: false, resolveError: null })).toBe("tooLong");
  });

  it("returns loading while resolving without a result", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: null, isResolving: true, resolveError: null })).toBe("loading");
  });

  it("returns saved when the resolved word already has wordId", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: { ...resolvedWord, wordId: "word_1" }, isResolving: false, resolveError: null })).toBe("saved");
  });

  it("returns success when the resolved word is not saved", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord, isResolving: false, resolveError: null })).toBe("success");
  });

  it("returns error when resolveError exists", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: null, isResolving: false, resolveError: "翻译失败" })).toBe("error");
  });
});
