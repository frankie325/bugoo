import { invoke } from "@tauri-apps/api/core";
import type { EnglishDefinitionGroup, WordFormItem } from "./wordDetails";
import type { WordMeaning } from "./word";

export interface TranslationExample {
  sentence: string;
  translation: string;
}

export interface TranslationResult {
  translation: string;
  detectedSourceLang: string | null;
  phonetic: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
}

interface RustTranslationResult {
  translation: string;
  detected_source_lang: string | null;
  phonetic: string | null;
  meanings: WordMeaning[];
  english_definitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  word_forms: WordFormItem[];
  memory_tip: string;
}

export async function translate(text: string): Promise<TranslationResult> {
  const result = await invoke<RustTranslationResult>("translate_text", {
    text,
  });

  return {
    translation: result.translation,
    detectedSourceLang: result.detected_source_lang,
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
