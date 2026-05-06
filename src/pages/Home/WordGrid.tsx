import { Card } from '@heroui/react';
import type { Word } from '../../lib/api';

interface WordGridProps {
  words: Word[];
  onWordClick: (word: Word) => void;
}

export function WordGrid({ words, onWordClick }: WordGridProps) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {words.map((word) => (
        <Card
          key={word.id}
          onClick={() => onWordClick(word)}
          className="border border-divider hover:border-primary transition-colors cursor-pointer"
        >
          <div className="p-4">
            <h3 className="font-medium text-lg">{word.word}</h3>
            {word.phonetic && (
              <p className="text-sm text-foreground-400">{word.phonetic}</p>
            )}
            <p className="text-sm text-foreground-600 mt-1 line-clamp-1">
              {word.translation}
            </p>
            {word.tags && (
              <div className="flex flex-wrap gap-1 mt-2">
                {word.tags.split(',').slice(0, 2).map((tag, i) => (
                  <span
                    key={i}
                    className="text-xs px-1 py-0.5 bg-foreground-100 rounded"
                  >
                    {tag.trim()}
                  </span>
                ))}
              </div>
            )}
          </div>
        </Card>
      ))}
    </div>
  );
}
