import { Button } from '@heroui/react';
import { useTranslation } from 'react-i18next';
import type { Word } from '../../../lib/api';

interface DetailPanelProps {
  word: Word;
  onClose: () => void;
}

export default function DetailPanel({ word, onClose }: DetailPanelProps) {
  const { t } = useTranslation();
  return (
    <aside className="w-80 border-l border-divider bg-background p-4 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-medium">{t("home.detail.title")}</h2>
        <Button size="sm" variant="ghost" onPress={onClose}>
          {t("home.detail.close")}
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
        <span className="text-sm font-medium">{t("home.detail.memoryProgress")}</span>
        <div className="flex justify-between text-sm">
          <span>{t("home.detail.memoryStrength")}</span>
          <span>{Math.round(word.ease_factor * 100)}%</span>
        </div>
        <div className="flex justify-between text-sm">
          <span>{t("home.detail.interval")}</span>
          <span>{t("home.detail.days", { days: word.interval })}</span>
        </div>
        <div className="flex justify-between text-sm">
          <span>{t("home.detail.reviewCount")}</span>
          <span>{word.repetitions}</span>
        </div>
      </div>

      <div className="mt-auto flex flex-col gap-2">
        <Button variant="ghost" className="text-success">
          {t("home.detail.remember")}
        </Button>
        <Button variant="ghost" className="text-warning">
          {t("home.detail.fuzzy")}
        </Button>
        <Button variant="ghost" className="text-danger">
          {t("home.detail.forgot")}
        </Button>
      </div>
    </aside>
  );
}
