import { create } from 'zustand';
import type { Word } from '../lib/api';

export type FilterStatus = 'all' | 'new' | 'learning' | 'reviewing' | 'mastered';

interface WordState {
  words: Word[];
  searchQuery: string;
  filterStatus: FilterStatus;
  selectedWord: Word | null;

  setWords: (words: Word[]) => void;
  addWord: (word: Word) => void;
  removeWord: (id: string) => void;
  updateWord: (word: Word) => void;
  setSearchQuery: (query: string) => void;
  setFilterStatus: (status: FilterStatus) => void;
  setSelectedWord: (word: Word | null) => void;
}

export const useWordStore = create<WordState>((set) => ({
  words: [],
  searchQuery: '',
  filterStatus: 'all',
  selectedWord: null,

  setWords: (words) => set({ words }),

  addWord: (word) => set((state) => ({
    words: [word, ...state.words]
  })),

  removeWord: (id) => set((state) => ({
    words: state.words.filter((w) => w.id !== id),
    selectedWord: state.selectedWord?.id === id ? null : state.selectedWord,
  })),

  updateWord: (word) => set((state) => ({
    words: state.words.map((w) => w.id === word.id ? word : w),
    selectedWord: state.selectedWord?.id === word.id ? word : state.selectedWord,
  })),

  setSearchQuery: (searchQuery) => set({ searchQuery }),

  setFilterStatus: (filterStatus) => set({ filterStatus }),

  setSelectedWord: (selectedWord) => set({ selectedWord }),
}));
