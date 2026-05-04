import type { Word } from '../lib/api';

interface WordListProps {
  words: Word[];
  onWordClick: (word: Word) => void;
}

export function WordList({ words, onWordClick }: WordListProps) {
  return (
    <div className="flex flex-col gap-2">
      {words.map((word) => (
        <div
          key={word.id}
          onClick={() => onWordClick(word)}
          className="flex items-center gap-4 p-3 border border-divider rounded hover:border-primary cursor-pointer transition-colors"
        >
          <span className="font-medium">{word.word}</span>
          <span className="text-foreground-600 flex-1">{word.translation}</span>
          <span className="text-sm text-foreground-400">{word.status}</span>
          {word.tags && (
            <span className="text-xs text-foreground-400">{word.tags}</span>
          )}
        </div>
      ))}
    </div>
  );
}
