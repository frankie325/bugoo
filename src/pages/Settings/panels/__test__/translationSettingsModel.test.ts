import { describe, expect, it } from "vitest";
import type { TranslationLanguages } from "../../../../lib/api";
import {
  getFilteredTargetLanguages,
  getTranslationCredentialFieldHints,
  getTranslationFieldVisibility,
  hasLanguage,
  localizeTranslationLanguageName,
} from "../translationSettingsModel";

describe("translationSettingsModel", () => {
  it("hides credential fields for local engine", () => {
    expect(getTranslationFieldVisibility("local")).toEqual({
      needsEndpoint: false,
      endpointOptional: false,
      needsApiKey: false,
      needsApiSecret: false,
      needsApiRegion: false,
      isCustomEngine: false,
    });
  });

  it("hides endpoint for vendor engines while keeping custom endpoint visible", () => {
    expect(getTranslationFieldVisibility("google").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("deepl").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("microsoft").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("baidu").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("tencent").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("youdao").needsEndpoint).toBe(false);
    expect(getTranslationFieldVisibility("custom").needsEndpoint).toBe(true);
  });

  it("shows only credential fields required by each vendor engine", () => {
    expect(getTranslationFieldVisibility("google")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: false,
      needsApiRegion: false,
    });
    expect(getTranslationFieldVisibility("deepl")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: false,
      needsApiRegion: false,
    });
    expect(getTranslationFieldVisibility("microsoft")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: false,
      needsApiRegion: true,
    });
    expect(getTranslationFieldVisibility("baidu")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: true,
      needsApiRegion: false,
    });
    expect(getTranslationFieldVisibility("tencent")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: true,
      needsApiRegion: true,
    });
    expect(getTranslationFieldVisibility("youdao")).toMatchObject({
      needsApiKey: true,
      needsApiSecret: true,
      needsApiRegion: false,
    });
  });

  it("returns vendor-specific credential field hints", () => {
    expect(getTranslationCredentialFieldHints("tencent")).toEqual({
      apiKey: "SecretId",
      apiSecret: "SecretKey",
      apiRegion: "Region",
    });
    expect(getTranslationCredentialFieldHints("google")).toEqual({});
  });

  it("checks language membership from config data", () => {
    expect(hasLanguage([{ code: "en", name: "English" }], "en")).toBe(true);
  });

  it("localizes standard language names with Intl.DisplayNames", () => {
    expect(
      localizeTranslationLanguageName({ code: "en", name: "English" }, "zh-CN"),
    ).toBe("英语");
    expect(
      localizeTranslationLanguageName({ code: "ja", name: "Japanese" }, "en"),
    ).toBe("Japanese");
  });

  it("localizes vendor language aliases with Intl.DisplayNames", () => {
    expect(
      localizeTranslationLanguageName({ code: "jp", name: "Japanese" }, "zh-CN"),
    ).toBe("日语");
    expect(
      localizeTranslationLanguageName({ code: "kor", name: "Korean" }, "zh-CN"),
    ).toBe("韩语");
    expect(
      localizeTranslationLanguageName(
        { code: "cht", name: "Chinese Traditional" },
        "zh-CN",
      ),
    ).toBe("繁体中文");
    expect(
      localizeTranslationLanguageName(
        { code: "pb", name: "Portuguese Brazilian" },
        "zh-CN",
      ),
    ).toBe("巴西葡萄牙语");
    expect(
      localizeTranslationLanguageName(
        {
          code: "wyw",
          name: "Classical Chinese",
          names: { "zh-CN": "文言文" },
        },
        "zh-CN",
      ),
    ).toBe("文言文");
  });

  it("uses the provided auto-detect label instead of Intl.DisplayNames", () => {
    expect(
      localizeTranslationLanguageName(
        { code: "auto", name: "Auto Detect" },
        "zh-CN",
        { autoDetectName: "自动检测" },
      ),
    ).toBe("自动检测");
  });

  it("falls back to name and then code when localized names are missing", () => {
    expect(
      localizeTranslationLanguageName(
        {
          code: "xx",
          name: "Example",
        },
        "zh-CN",
      ),
    ).toBe("Example");

    expect(
      localizeTranslationLanguageName(
        {
          code: "xx",
          name: "",
        },
        "zh-CN",
      ),
    ).toBe("xx");
  });

  it("getFilteredTargetLanguages: Youdao-style config without mapping returns target list without auto", () => {
    const youdao: TranslationLanguages = {
      sourceLanguages: [
        { code: "auto", name: "自动识别" },
        { code: "en", name: "英语" },
        { code: "zh-CHS", name: "简体中文" },
      ],
      targetLanguages: [
        { code: "en", name: "英语" },
        { code: "zh-CHS", name: "简体中文" },
        { code: "ja", name: "日语" },
      ],
    };

    const result = getFilteredTargetLanguages("en", youdao);

    expect(result.map((t) => t.code)).toEqual(["en", "zh-CHS", "ja"]);
    expect(result.some((t) => t.code === "auto")).toBe(false);
  });

  it("getFilteredTargetLanguages: returns empty when source not in mapping", () => {
    const langs: TranslationLanguages = {
      sourceLanguages: [
        { code: "en", name: "English" },
        { code: "zh", name: "中文" },
      ],
      targetLanguages: [
        { code: "en", name: "English" },
        { code: "zh", name: "中文" },
      ],
      sourceToTargetMapping: {
        en: ["zh"],
        zh: ["en"],
      },
    };

    expect(getFilteredTargetLanguages("fr", langs)).toEqual([]);
  });

  it("getFilteredTargetLanguages: filters targets by mapping", () => {
    const langs: TranslationLanguages = {
      sourceLanguages: [
        { code: "en", name: "English" },
        { code: "zh", name: "中文" },
      ],
      targetLanguages: [
        { code: "en", name: "English" },
        { code: "zh", name: "中文" },
        { code: "ja", name: "日语" },
      ],
      sourceToTargetMapping: {
        en: ["zh", "ja"],
        zh: ["en"],
      },
    };

    expect(getFilteredTargetLanguages("en", langs).map((t) => t.code)).toEqual([
      "zh",
      "ja",
    ]);
    expect(getFilteredTargetLanguages("zh", langs).map((t) => t.code)).toEqual([
      "en",
    ]);
  });
});
