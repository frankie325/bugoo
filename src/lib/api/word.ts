import { invoke } from "@tauri-apps/api/core";
import type {
  EnglishDefinitionGroup,
  WordDetail,
  WordFormItem,
} from "./wordDetails";
import { toWordDetail } from "./wordDetails";
import type { TranslationExample } from "./translate";

export interface WordMeaning {
  partOfSpeech: string;
  translations: string[];
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

export interface AddWordInput {
  word: string;
  translation: string;
  sourceLang: string;
  targetLang: string;
  phonetic?: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
  tags?: string;
}

export async function addWord(input: AddWordInput): Promise<WordDetail> {
  const detail = await invoke<Parameters<typeof toWordDetail>[0]>("add_word", {
    input,
  });
  return toWordDetail(detail);
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
