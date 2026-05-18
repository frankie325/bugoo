import { invoke } from "@tauri-apps/api/core";

export interface TranslationResult {
  translation: string;
  detected_source_lang: string | null;
}

export async function translate(
  text: string,
  sourceLang: string,
  targetLang: string,
): Promise<TranslationResult> {
  return invoke("translate_text", {
    text,
    sourceLang,
    targetLang,
  });
}