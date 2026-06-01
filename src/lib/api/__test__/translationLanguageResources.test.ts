// @ts-nocheck
import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const translationResourcesDir = path.join(
  process.cwd(),
  "src-tauri",
  "resources",
  "translation",
);

type ResourceLanguage = {
  code?: string;
  name?: string;
  names?: Record<string, string>;
};

type TranslationLanguageResource = {
  sourceLanguages?: ResourceLanguage[];
  targetLanguages?: ResourceLanguage[];
};

describe("translation language resources", () => {
  it("keeps language resources lightweight and fallback-compatible", () => {
    const bloatedEntries: string[] = [];
    const invalidEntries: string[] = [];

    for (const filename of fs
      .readdirSync(translationResourcesDir)
      .filter((file) => file.endsWith("-languages.json"))
      .sort()) {
      const resource = JSON.parse(
        fs.readFileSync(path.join(translationResourcesDir, filename), "utf8"),
      ) as TranslationLanguageResource;

      for (const section of ["sourceLanguages", "targetLanguages"] as const) {
        for (const language of resource[section] ?? []) {
          if (!language.code?.trim() || !language.name?.trim()) {
            invalidEntries.push(`${filename}:${section}:${language.code ?? "<missing>"}`);
          }

          const namesKeys = Object.keys(language.names ?? {});
          if (namesKeys.length > 3) {
            bloatedEntries.push(
              `${filename}:${section}:${language.code}:${namesKeys.length}`,
            );
          }
        }
      }
    }

    expect(invalidEntries).toEqual([]);
    expect(
      bloatedEntries.length,
      `Resources should rely on Intl.DisplayNames instead of full locale maps: ${bloatedEntries
        .slice(0, 10)
        .join(", ")}`,
    ).toBe(0);
  });
});
