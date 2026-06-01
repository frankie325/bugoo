import { invoke } from "@tauri-apps/api/core";

export type TranslationLanguage = {
  code: string;
  name: string;
  names?: Record<string, string>;
};

export type SourceToTargetMapping = Record<string, string[]>;

export type TranslationLanguages = {
  sourceLanguages: TranslationLanguage[];
  targetLanguages: TranslationLanguage[];
  sourceToTargetMapping?: SourceToTargetMapping;
};

type RustTranslationLanguages = {
  sourceLanguages?: TranslationLanguage[];
  targetLanguages?: TranslationLanguage[];
  source_languages?: TranslationLanguage[];
  target_languages?: TranslationLanguage[];
  sourceToTargetMapping?: SourceToTargetMapping;
  source_to_target_mapping?: SourceToTargetMapping;
};

export async function getTranslationLanguages(
  engine: string,
): Promise<TranslationLanguages> {
  const result = await invoke<RustTranslationLanguages>(
    "get_translation_languages",
    { engine },
  );
  
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
    sourceToTargetMapping: result.sourceToTargetMapping ?? result.source_to_target_mapping ?? {},
  };
}
