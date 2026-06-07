import { Button, Chip, ScrollShadow, Surface, Tooltip } from "@heroui/react";
import { Tag } from "lucide-react";
import type { TagItem } from "../../../types/tag";
import { CircleFill } from "@gravity-ui/icons";

type TagChipListProps = {
  tags: TagItem[];
  selectedTags: TagItem[];
  selectedTagIds: string[];
  onToggleTag: (tagId: string) => void;
};

export function TagChipList({
  tags,
  selectedTags,
  selectedTagIds,
  onToggleTag,
}: TagChipListProps) {
  return (
    <div className="flex items-center gap-1.5 mb-1">
      <ScrollShadow
        orientation="horizontal"
        className="flex min-w-0 items-center gap-1.5"
        hideScrollBar
      >
        {selectedTags.map((tag) => (
          <Chip
            key={tag.id}
            size="md"
            variant="soft"
            className="shrink-0 gap-1 rounded-md px-2"
            style={{
              backgroundColor: `${tag.color}1A`,
              color: tag.color,
              borderColor: `${tag.color}33`,
            }}
          >
            <CircleFill width={8} />
            <Chip.Label>{tag.name}</Chip.Label>
          </Chip>
        ))}
      </ScrollShadow>
      <Tooltip delay={0}>
        <Tooltip.Trigger>
          <Button
            size="sm"
            variant="ghost"
            isIconOnly
            aria-label="选择标签"
            className="shrink-0"
          >
            <Tag size={16} aria-hidden="true" />
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Content placement="top end" showArrow>
          <Tooltip.Arrow />
          <Surface className="w-60 bg-surface p-1 pt-0">
            <h3 className="mb-3 flex items-center gap-2 text-base font-semibold text-foreground">
              <Tag size={14} />
              选择标签
            </h3>
            <div className="flex flex-wrap gap-2">
              {tags.map((tag) => {
                const selected = selectedTagIds.includes(tag.id);
                return (
                  <Chip
                    key={tag.id}
                    size="md"
                    variant="soft"
                    className="cursor-pointer gap-1 rounded-md px-2 text-muted"
                    style={
                      selected
                        ? {
                            backgroundColor: `${tag.color}1A`,
                            color: tag.color,
                            borderColor: `${tag.color}33`,
                          }
                        : {}
                    }
                    onClick={() => onToggleTag(tag.id)}
                  >
                    <CircleFill width={8} />
                    <Chip.Label>{tag.name}</Chip.Label>
                  </Chip>
                );
              })}
            </div>
          </Surface>
        </Tooltip.Content>
      </Tooltip>
    </div>
  );
}
