import type {
  TranslationLanguage,
  TranslationLanguages,
  SourceToTargetMapping,
} from "../../../lib/api";

export type { TranslationLanguage, TranslationLanguages, SourceToTargetMapping };

export type TranslationEngine =
  | "local"
  | "google"
  | "deepl"
  | "microsoft"
  | "baidu"
  | "tencent"
  | "youdao"
  | "custom";

export interface TranslationCredentialFieldHints {
  apiKey?: string;
  apiSecret?: string;
  apiRegion?: string;
}

export const emptyTranslationLanguages: TranslationLanguages = {
  sourceLanguages: [],
  targetLanguages: [],
};

export function getTranslationFieldVisibility(engine: TranslationEngine) {
  return {
    needsEndpoint: engine === "custom",
    endpointOptional: false,
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

export function getTranslationCredentialFieldHints(
  engine: TranslationEngine,
): TranslationCredentialFieldHints {
  switch (engine) {
    case "baidu":
      return {
        apiKey: "APP ID",
        apiSecret: "APP Secret",
      };
    case "tencent":
      return {
        apiKey: "SecretId",
        apiSecret: "SecretKey",
        apiRegion: "Region",
      };
    case "youdao":
      return {
        apiKey: "App ID",
        apiSecret: "APP Secret",
      };
    case "deepl":
      return {
        apiKey: "Auth Key",
      };
    default:
      return {};
  }
}

export function hasLanguage(languages: TranslationLanguage[], code: string) {
  return languages.some((language) => language.code === code);
}

const translationLanguageCodeAliases: Record<string, string> = {
  ara: "ar",
  bul: "bg",
  cht: "zh-Hant",
  dan: "da",
  est: "et",
  fin: "fi",
  fra: "fr",
  jp: "ja",
  jw: "jv",
  kor: "ko",
  pb: "pt-BR",
  rom: "ro",
  slo: "sl",
  spa: "es",
  swe: "sv",
  vie: "vi",
  wyw: "lzh",
  zt: "zh-Hant",
  "zh-chs": "zh-Hans",
  "zh-cht": "zh-Hant",
};

export function normalizeTranslationLanguageDisplayCode(code: string) {
  const trimmedCode = code.trim();
  return (
    translationLanguageCodeAliases[trimmedCode.toLowerCase()] ||
    trimmedCode.replace("_", "-")
  );
}

function displayNameForLocale(code: string, locale: string) {
  try {
    const displayNames = new Intl.DisplayNames([locale], { type: "language" });
    const displayName = displayNames.of(code);
    return displayName && displayName !== code ? displayName : undefined;
  } catch {
    return undefined;
  }
}

export function localizeTranslationLanguageName(
  language: TranslationLanguage,
  locale?: string,
  options: { autoDetectName?: string } = {},
) {
  if (language.code.trim().toLowerCase() === "auto") {
    return options.autoDetectName?.trim() || language.name.trim() || language.code;
  }

  const names = language.names ?? {};
  const normalizedLocale = locale?.trim();
  const baseLocale = normalizedLocale?.split("-")[0];
  const fallbackLocales = [
    normalizedLocale,
    baseLocale,
    baseLocale === "zh" ? "zh-CN" : undefined,
    "en",
  ].filter((value): value is string => Boolean(value));

  for (const fallbackLocale of fallbackLocales) {
    const localizedName = names[fallbackLocale];
    if (localizedName?.trim()) {
      return localizedName;
    }
  }

  const displayCode = normalizeTranslationLanguageDisplayCode(language.code);
  for (const fallbackLocale of fallbackLocales) {
    const displayName = displayNameForLocale(displayCode, fallbackLocale);
    if (displayName?.trim()) {
      return displayName;
    }
  }

  return language.name.trim() || language.code;
}

export function getFilteredTargetLanguages(
  sourceLang: string,
  translationLanguages: TranslationLanguages,
): TranslationLanguage[] {
  if (
    !translationLanguages.sourceToTargetMapping ||
    Object.keys(translationLanguages.sourceToTargetMapping).length === 0
  ) {
    return translationLanguages.targetLanguages;
  }
  const allowed = translationLanguages.sourceToTargetMapping[sourceLang];
  if (!allowed || allowed.length === 0) {
    return [];
  }
  return translationLanguages.targetLanguages.filter((t) =>
    allowed.includes(t.code),
  );
}
