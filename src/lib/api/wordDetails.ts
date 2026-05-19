import { invoke } from "@tauri-apps/api/core";

import type { TranslationExample } from "./translate";

export interface WordDetail {
  wordId: string;
  word: string;
  translation: string;
  phonetic: string | null;
  partOfSpeech: string[];
  definitions: string[];
  examples: TranslationExample[];
  memoryTip: string;
  detail: string;
  provider: string;
  rawJson: string;
  createdAt: number;
  updatedAt: number;
}

interface RustWordDetail {
  word_id: string;
  word: string;
  translation: string;
  phonetic: string | null;
  part_of_speech: string[];
  definitions: string[];
  examples: TranslationExample[];
  memory_tip: string;
  detail: string;
  provider: string;
  raw_json: string;
  created_at: number;
  updated_at: number;
}

function toWordDetail(detail: RustWordDetail): WordDetail {
  return {
    wordId: detail.word_id,
    word: detail.word,
    translation: detail.translation,
    phonetic: detail.phonetic,
    partOfSpeech: Array.isArray(detail.part_of_speech)
      ? detail.part_of_speech
      : [],
    definitions: Array.isArray(detail.definitions) ? detail.definitions : [],
    examples: Array.isArray(detail.examples) ? detail.examples : [],
    memoryTip: detail.memory_tip,
    detail: detail.detail,
    provider: detail.provider,
    rawJson: detail.raw_json,
    createdAt: detail.created_at,
    updatedAt: detail.updated_at,
  };
}

export async function getWordDetail(
  wordId: string,
): Promise<WordDetail | null> {
  const detail = await invoke<RustWordDetail | null>("get_word_detail", {
    wordId,
  });

  return detail ? toWordDetail(detail) : null;
}

export async function generateWordDetail(wordId: string): Promise<WordDetail> {
  const detail = await invoke<RustWordDetail>("generate_word_detail", {
    wordId,
  });

  return toWordDetail(detail);
}
