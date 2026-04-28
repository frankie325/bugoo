import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Input, Card, CardBody, CardFooter, Chip, Spinner } from '@heroui/react';

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
    <div className="space-y-4">
      <Input
        placeholder="搜索单词..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        isClearable
      />

      {isLoading && <Spinner />}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {words.map((w) => (
          <Card
            key={w.id}
            isPressable
            onPress={() => onSelectWord(w)}
            className={selectedWordId === w.id ? 'border-primary' : ''}
          >
            <CardBody>
              <p className="text-lg font-bold">{w.word}</p>
              <p className="text-default-500">{w.translation}</p>
            </CardBody>
            <CardFooter className="gap-2">
              <Chip
                size="sm"
                color={w.status === 'learning' ? 'warning' : 'success'}
              >
                {w.status === 'learning' ? '复习中' : '已记住'}
              </Chip>
              <p className="text-xs text-default-400">
                下次: {formatNextReview(w.next_review_at)}
              </p>
            </CardFooter>
          </Card>
        ))}
      </div>

      {words.length === 0 && !isLoading && (
        <div className="text-center py-12 text-default-400">暂无单词</div>
      )}
    </div>
  );
}