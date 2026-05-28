import { describe, expect, it } from "vitest";
import {
  getTranslationFieldVisibility,
  hasLanguage,
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

  it("checks language membership from config data", () => {
    expect(hasLanguage([{ code: "en", name: "English" }], "en")).toBe(true);
  });

});
