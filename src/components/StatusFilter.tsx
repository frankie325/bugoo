import { ListBox, ListBoxItem } from '@heroui/react';
import type { Word } from '../lib/api';
import type { FilterStatus } from '../stores/wordStore';

interface StatusFilterProps {
  words: Word[];
  currentFilter: FilterStatus;
  onFilterChange: (status: FilterStatus) => void;
}

const statusOptions: { key: FilterStatus; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'new', label: '新添加' },
  { key: 'learning', label: '复习中' },
  { key: 'reviewing', label: '今天待复习' },
  { key: 'mastered', label: '已记住' },
];

export function StatusFilter({ words, currentFilter, onFilterChange }: StatusFilterProps) {
  const counts = {
    all: words.length,
    new: words.filter((w) => w.status === 'new').length,
    learning: words.filter((w) => w.status === 'learning').length,
    reviewing: words.filter((w) => {
      const nextReview = w.next_review_at;
      if (!nextReview) return false;
      return nextReview <= Date.now();
    }).length,
    mastered: words.filter((w) => w.status === 'mastered').length,
  };

  return (
    <ListBox
      aria-label="状态筛选"
      selectionMode="single"
      selectedKeys={[currentFilter]}
      onSelectionChange={(keys) => {
        const key = Array.from(keys)[0] as FilterStatus;
        if (key) onFilterChange(key);
      }}
    >
      {statusOptions.map((option) => (
        <ListBoxItem key={option.key}>
          {option.label} ({counts[option.key]})
        </ListBoxItem>
      ))}
    </ListBox>
  );
}
