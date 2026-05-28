import { invoke } from "@tauri-apps/api/core";

export type TranslationLanguage = {
  code: string;
  name: string;
};

export type TranslationLanguages = {
  sourceLanguages: TranslationLanguage[];
  targetLanguages: TranslationLanguage[];
};

type RustTranslationLanguages = {
  sourceLanguages?: TranslationLanguage[];
  targetLanguages?: TranslationLanguage[];
  source_languages?: TranslationLanguage[];
  target_languages?: TranslationLanguage[];
};

export async function getTranslationLanguages(): Promise<TranslationLanguages> {
  const result = await invoke<RustTranslationLanguages>("get_translation_languages");

  return {
    sourceLanguages: Array.isArray(result.sourceLanguages)
      ? result.sourceLanguages
      : Array.isArray(result.source_languages)
        ? result.source_languages
      : [],
    targetLanguages: Array.isArray(result.targetLanguages)
      ? result.targetLanguages
      : Array.isArray(result.target_languages)
        ? result.target_languages
      : [],
  };
}
