import type { TagItem } from "../types/tag";

export type TagCreatePlacement = "above" | "below" | "end";

export type SortOrderResult =
  | { kind: "insert"; order: number }
  | { kind: "reorder"; newOrder: number; reorderedTags: TagItem[] };

export function getNextSortOrder(
  tags: TagItem[],
  placement: TagCreatePlacement = "end",
  anchorTagId?: string,
): SortOrderResult {
  const sortedTags = [...tags].sort((a, b) => a.sort_order - b.sort_order);

  if (sortedTags.length === 0) {
    return { kind: "insert", order: 0 };
  }

  if (placement === "end" || !anchorTagId) {
    return {
      kind: "insert",
      order: sortedTags.length,
    };
  }

  const anchorIndex = sortedTags.findIndex((tag) => tag.id === anchorTagId);
  if (anchorIndex === -1) {
    return { kind: "insert", order: sortedTags.length };
  }

  // "above": new tag takes anchor's position, anchor and all below shift by +1
  // "below": new tag takes anchor's position + 1, all below shift by +1
  const newOrder = placement === "above" ? anchorIndex : anchorIndex + 1;

  // If new tag goes after the last tag, no shifting needed
  if (newOrder >= sortedTags.length) {
    return { kind: "insert", order: sortedTags.length };
  }

  // Shift all tags at newOrder and below by +1
  const reorderedTags = sortedTags.map((tag, i) => ({
    ...tag,
    sort_order: i < newOrder ? i : i + 1,
  }));

  return { kind: "reorder", newOrder, reorderedTags };
}