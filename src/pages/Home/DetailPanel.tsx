import { Button } from '@heroui/react';
import type { Word } from '../../lib/api';

interface DetailPanelProps {
  word: Word;
  onClose: () => void;
}

export function DetailPanel({ word, onClose }: DetailPanelProps) {
  return (
    <aside className="w-80 border-l border-divider bg-background p-4 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-medium">单词详情</h2>
        <Button size="sm" variant="ghost" onPress={onClose}>
          关闭
        </Button>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-2xl font-bold">{word.word}</h3>
        {word.phonetic && (
          <p className="text-sm text-foreground-400">{word.phonetic}</p>
        )}
        <p className="text-foreground-600">{word.translation}</p>
      </div>

      {word.tags && (
        <div className="flex flex-wrap gap-1">
          {word.tags.split(',').map((tag, i) => (
            <span
              key={i}
              className="text-xs px-2 py-1 bg-foreground-100 rounded"
            >
              {tag.trim()}
            </span>
          ))}
        </div>
      )}

      <div className="flex flex-col gap-2">
        <span className="text-sm font-medium">记忆进度</span>
        <div className="flex justify-between text-sm">
          <span>记忆强度</span>
          <span>{Math.round(word.ease_factor * 100)}%</span>
        </div>
        <div className="flex justify-between text-sm">
          <span>间隔</span>
          <span>{word.interval}天</span>
        </div>
        <div className="flex justify-between text-sm">
          <span>复习次数</span>
          <span>{word.repetitions}</span>
        </div>
      </div>

      <div className="mt-auto flex flex-col gap-2">
        <Button variant="ghost" className="text-success">
          😄 我记住了
        </Button>
        <Button variant="ghost" className="text-warning">
          🤔 有点模糊
        </Button>
        <Button variant="ghost" className="text-danger">
          😵 完全不会
        </Button>
      </div>
    </aside>
  );
}
