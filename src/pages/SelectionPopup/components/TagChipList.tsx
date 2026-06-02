import { Button } from "@heroui/react";
import { Plus } from "lucide-react";
import type { TagItem } from "../../../types/tag";

type TagChipListProps = {
  selectedTags: TagItem[];
  onAddPress: () => void;
};

export function TagChipList({ selectedTags, onAddPress }: TagChipListProps) {
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {selectedTags.map((tag) => (
        <span key={tag.id} className="rounded-full bg-[#EFF6FF] px-2 py-0.5 text-[11px] font-medium text-[#2563EB]">
          {tag.name}
        </span>
      ))}
      <Button variant="ghost" size="sm" isIconOnly aria-label="选择标签" onPress={onAddPress}>
        <Plus className="size-3.5" aria-hidden="true" />
      </Button>
    </div>
  );
}
