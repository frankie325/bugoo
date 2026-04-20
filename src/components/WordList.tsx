import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface Word {
  id: string;
  word: string;
  translation: string;
  status: string;
  tags: string;
  notes: string;
  source_lang: string;
  target_lang: string;
  audio_url: string;
  ease_factor: number;
  interval: number;
  repetitions: number;
  next_review_at: number;
  created_at: number;
  updated_at: number;
}

interface WordListProps {
  onSelectWord: (word: Word) => void;
  selectedWordId: string | null;
}

export function WordList({ onSelectWord, selectedWordId }: WordListProps) {
  const [words, setWords] = useState<Word[]>([]);
  const [search, setSearch] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const loadWords = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<Word[]>('get_words', { search: search || null });
      setWords(result);
    } catch (e) {
      console.error('Failed to load words:', e);
    } finally {
      setIsLoading(false);
    }
  }, [search]);

  useEffect(() => {
    loadWords();
  }, [loadWords]);

  const formatNextReview = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="word-list">
      <input
        className="search-input"
        type="text"
        placeholder="搜索单词..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

      {isLoading && <div className="loading-indicator">加载中...</div>}

      <div className="word-items">
        {words.length === 0 && !isLoading && (
          <div className="empty-state">暂无单词</div>
        )}
        {words.map((w) => (
          <div
            key={w.id}
            className={`word-item ${selectedWordId === w.id ? 'selected' : ''}`}
            onClick={() => onSelectWord(w)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                onSelectWord(w);
              }
            }}
          >
            <div className="word-main">
              <span className="word-text">{w.word}</span>
              <span className="word-translation">{w.translation}</span>
            </div>
            <div className="word-meta">
              <span className={`word-status status-${w.status}`}>
                {w.status === 'learning' ? '复习中' : '已记住'}
              </span>
              <span className="word-next-review">
                下次: {formatNextReview(w.next_review_at)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}