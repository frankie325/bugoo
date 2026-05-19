import { invoke } from "@tauri-apps/api/core";

export interface TranslationExample {
  sentence: string;
  translation: string;
}

export interface TranslationResult {
  translation: string;
  detectedSourceLang: string | null;
  phonetic: string | null;
  partOfSpeech: string[];
  definitions: string[];
  examples: TranslationExample[];
}

interface RustTranslationResult {
  translation: string;
  detected_source_lang: string | null;
  phonetic: string | null;
  part_of_speech: string[];
  definitions: string[];
  examples: TranslationExample[];
}

export async function translate(
  text: string,
  sourceLang: string,
  targetLang: string,
): Promise<TranslationResult> {
  const result = await invoke<RustTranslationResult>("translate_text", {
    text,
    sourceLang,
    targetLang,
  });

  return {
    translation: result.translation,
    detectedSourceLang: result.detected_source_lang,
    phonetic: result.phonetic,
    partOfSpeech: Array.isArray(result.part_of_speech)
      ? result.part_of_speech
      : [],
    definitions: Array.isArray(result.definitions) ? result.definitions : [],
    examples: Array.isArray(result.examples) ? result.examples : [],
  };
}
