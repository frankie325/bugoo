import { Chip } from '@heroui/react';

interface TagSectionProps {
  tags: string[];
  selectedTag: string | null;
  onTagSelect: (tag: string | null) => void;
}

export function TagSection({ tags, selectedTag, onTagSelect }: TagSectionProps) {
  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium text-foreground-500">标签</span>
      <div className="flex flex-wrap gap-1">
        {tags.map((tag) => (
          <Chip
            key={tag}
            variant={selectedTag === tag ? 'primary' : 'soft'}
            onClick={() => onTagSelect(selectedTag === tag ? null : tag)}
          >
            {tag}
          </Chip>
        ))}
      </div>
    </div>
  );
}
