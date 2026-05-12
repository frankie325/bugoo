import { Chip } from '@heroui/react';
import { useTranslation } from 'react-i18next';

interface TagSectionProps {
  tags: string[];
  selectedTag: string | null;
  onTagSelect: (tag: string | null) => void;
}

export function TagSection({ tags, selectedTag, onTagSelect }: TagSectionProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium text-gray-500">{t("home.tagsLabel")}</span>
      <div className="flex flex-wrap gap-1">
        {tags.map((tag) => (
          <Chip
            key={tag}
            variant={selectedTag === tag ? "primary" : "soft"}
            onClick={() => onTagSelect(selectedTag === tag ? null : tag)}
          >
            {tag}
          </Chip>
        ))}
      </div>
    </div>
  );
}
