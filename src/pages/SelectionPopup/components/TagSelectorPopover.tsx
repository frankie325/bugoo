import { Button, Popover } from "@heroui/react";
import { X } from "lucide-react";
import type { TagItem } from "../../../types/tag";

type TagSelectorPopoverProps = {
  isOpen: boolean;
  tags: TagItem[];
  selectedTagIds: string[];
  onOpenChange: (isOpen: boolean) => void;
  onToggleTag: (tagId: string) => void;
  onCreateTag: () => void;
};

export function TagSelectorPopover({
  isOpen,
  tags,
  selectedTagIds,
  onOpenChange,
  onToggleTag,
  onCreateTag,
}: TagSelectorPopoverProps) {
  return (
    <Popover isOpen={isOpen} onOpenChange={onOpenChange}>
      <Popover.Trigger>
        <span />
      </Popover.Trigger>
      <Popover.Content placement="right">
        <Popover.Dialog className="w-64 rounded-xl bg-white p-3 shadow-xl">
          <div className="mb-3 flex items-center justify-between">
            <p className="text-sm font-semibold text-[#111827]">选择标签</p>
            <Button variant="ghost" size="sm" isIconOnly aria-label="关闭标签选择" onPress={() => onOpenChange(false)}>
              <X className="size-4" aria-hidden="true" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2">
            {tags.map((tag) => {
              const selected = selectedTagIds.includes(tag.id);
              return (
                <button
                  key={tag.id}
                  type="button"
                  onClick={() => onToggleTag(tag.id)}
                  className={selected ? "rounded-full bg-[#DCFCE7] px-2.5 py-1 text-xs font-medium text-[#16A34A]" : "rounded-full bg-[#F3F4F6] px-2.5 py-1 text-xs font-medium text-[#6B7280]"}
                >
                  {tag.name}
                </button>
              );
            })}
          </div>
          <Button className="mt-3" variant="ghost" size="sm" onPress={onCreateTag}>
            新建标签
          </Button>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
