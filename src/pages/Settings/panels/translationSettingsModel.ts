import type { TranslationLanguage, TranslationLanguages } from "../../../lib/api";

export type { TranslationLanguage, TranslationLanguages };

export type TranslationEngine =
  | "local"
  | "google"
  | "deepl"
  | "microsoft"
  | "baidu"
  | "tencent"
  | "youdao"
  | "custom";

export const emptyTranslationLanguages: TranslationLanguages = {
  sourceLanguages: [],
  targetLanguages: [],
};

export function getTranslationFieldVisibility(engine: TranslationEngine) {
  return {
    needsEndpoint: ["google", "deepl", "microsoft", "custom"].includes(engine),
    endpointOptional: ["google", "deepl", "microsoft"].includes(engine),
    needsApiKey: [
      "google",
      "deepl",
      "microsoft",
      "baidu",
      "tencent",
      "youdao",
      "custom",
    ].includes(engine),
    needsApiSecret: ["baidu", "tencent", "youdao"].includes(engine),
    needsApiRegion: ["microsoft", "tencent"].includes(engine),
    isCustomEngine: engine === "custom",
  };
}

export function hasLanguage(languages: TranslationLanguage[], code: string) {
  return languages.some((language) => language.code === code);
}