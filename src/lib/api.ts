import { invoke } from "@tauri-apps/api/core";

export interface TranslationResult {
  translation: string;
  detected_source_lang: string | null;
}

export interface Word {
  id: string;
  word: string;
  translation: string;
  phonetic?: string;
  source_lang: string;
  target_lang: string;
  status: string;
  tags: string;
  notes: string;
  audio_url?: string;
  ease_factor: number;
  interval: number;
  repetitions: number;
  next_review_at: number;
  created_at: number;
  updated_at: number;
}

export interface WordUpdate {
  translation?: string;
  tags?: string;
  notes?: string;
  status?: string;
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

export async function addWord(
  word: string,
  translation: string,
  sourceLang: string = "EN",
  targetLang: string = "ZH",
  tags: string = "",
): Promise<Word> {
  return invoke("add_word", {
    word,
    translation,
    sourceLang,
    targetLang,
    tags,
  });
}

export async function getWords(search?: string): Promise<Word[]> {
  return invoke("get_words", { search });
}

export async function deleteWord(wordId: string): Promise<void> {
  return invoke("delete_word", { wordId });
}

export async function updateWord(
  wordId: string,
  updates: WordUpdate,
): Promise<Word> {
  return invoke("update_word", { wordId, updates });
}

export async function getSettings(): Promise<Record<string, string>> {
  return invoke("get_settings");
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export async function seedSettings(): Promise<Record<string, string>> {
  return invoke("seed_settings");
}
