import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getWords, addWord, deleteWord, updateWord } from '../lib/api';
import type { AddWordInput, Word, WordDetail, WordUpdate } from '../lib/api';
import { useWordStore } from '../stores/wordStore';

export function useWords(search?: string) {
  const setWords = useWordStore((state) => state.setWords);

  return useQuery({
    queryKey: ['words', search],
    queryFn: async () => {
      const words = await getWords(search);
      setWords(words);
      return words;
    },
  });
}

export function useAddWord() {
  const queryClient = useQueryClient();
  const addWordToStore = useWordStore((state) => state.addWord);

  return useMutation({
    mutationFn: (input: AddWordInput) => addWord(input),
    onSuccess: (saved: WordDetail) => {
      const word: Word = {
        id: saved.wordId,
        word: saved.word,
        translation: saved.translation,
        phonetic: saved.phonetic ?? undefined,
        source_lang: 'en',
        target_lang: 'zh',
        status: 'new',
        tags: '',
        notes: '',
        audio_url: undefined,
        ease_factor: 2.5,
        interval: 0,
        repetitions: 0,
        next_review_at: 0,
        created_at: saved.createdAt,
        updated_at: saved.updatedAt,
      };
      addWordToStore(word);
      queryClient.invalidateQueries({ queryKey: ['words'] });
    },
  });
}

export function useDeleteWord() {
  const queryClient = useQueryClient();
  const removeWordFromStore = useWordStore((state) => state.removeWord);

  return useMutation({
    mutationFn: (wordId: string) => deleteWord(wordId),
    onSuccess: (_: void, wordId: string) => {
      removeWordFromStore(wordId);
      queryClient.invalidateQueries({ queryKey: ['words'] });
    },
  });
}

export function useUpdateWord() {
  const queryClient = useQueryClient();
  const updateWordInStore = useWordStore((state) => state.updateWord);

  return useMutation({
    mutationFn: ({ wordId, updates }: { wordId: string; updates: WordUpdate }) =>
      updateWord(wordId, updates),
    onSuccess: (updatedWord: Word) => {
      updateWordInStore(updatedWord);
      queryClient.invalidateQueries({ queryKey: ['words'] });
    },
  });
}
