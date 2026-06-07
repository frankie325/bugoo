import { invoke } from "@tauri-apps/api/core";
import type { WordMeaning } from "./word";
import type { TranslationExample } from "./translate";

export interface EnglishDefinitionGroup {
  partOfSpeech: string;
  definitions: string[];
}

export interface WordFormItem {
  type: string;
  words: string[];
}

export interface WordDetail {
  wordId: string;
  word: string;
  translation: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
  createdAt: number;
  updatedAt: number;
}

export interface ResolvedWord {
  wordId: string | null;
  word: string;
  translation: string;
  detectedSourceLang: string | null;
  sourceLang: string;
  targetLang: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
}

interface RustWordDetail {
  word_id: string;
  word: string;
  translation: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  english_definitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  word_forms: WordFormItem[];
  memory_tip: string;
  created_at: number;
  updated_at: number;
}

interface RustResolvedWord {
  word_id: string | null;
  word: string;
  translation: string;
  detected_source_lang: string | null;
  source_lang: string;
  target_lang: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  english_definitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  word_forms: WordFormItem[];
  memory_tip: string;
}

export function toWordDetail(detail: RustWordDetail): WordDetail {
  return {
    wordId: detail.word_id,
    word: detail.word,
    translation: detail.translation,
    phonetic: detail.phonetic,
    meanings: Array.isArray(detail.meanings) ? detail.meanings : [],
    englishDefinitions: Array.isArray(detail.english_definitions)
      ? detail.english_definitions
      : [],
    examples: Array.isArray(detail.examples) ? detail.examples : [],
    wordForms: Array.isArray(detail.word_forms) ? detail.word_forms : [],
    memoryTip: detail.memory_tip,
    createdAt: detail.created_at,
    updatedAt: detail.updated_at,
  };
}

function toResolvedWord(result: RustResolvedWord): ResolvedWord {
  return {
    wordId: result.word_id,
    word: result.word,
    translation: result.translation,
    detectedSourceLang: result.detected_source_lang,
    sourceLang: result.source_lang,
    targetLang: result.target_lang,
    phonetic: result.phonetic,
    meanings: Array.isArray(result.meanings) ? result.meanings : [],
    englishDefinitions: Array.isArray(result.english_definitions)
      ? result.english_definitions
      : [],
    examples: Array.isArray(result.examples) ? result.examples : [],
    wordForms: Array.isArray(result.word_forms) ? result.word_forms : [],
    memoryTip: result.memory_tip ?? "",
  };
}

export async function getWordDetail(
  wordId: string,
): Promise<WordDetail | null> {
  const detail = await invoke<RustWordDetail | null>("get_word_detail", {
    wordId,
  });

  return detail ? toWordDetail(detail) : null;
}

export async function resolveWord(text: string): Promise<ResolvedWord> {
  const result = await invoke<RustResolvedWord>("resolve_word", { text });
  return toResolvedWord(result);
}
