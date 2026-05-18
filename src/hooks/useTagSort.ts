import { useMemo, useCallback } from "react";
import type { TagItem } from "../types/tag";
import {
  DndContext,
  DragEndEvent,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";

export function useTagSort(
  tags: TagItem[],
  onReorder: (tagIds: string[]) => Promise<void>,
) {
  const sortedTags = useMemo(() => {
    return [...tags].sort((a, b) => a.sort_order - b.sort_order);
  }, [tags]);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  const handleDragEnd = useCallback(
    async (event: DragEndEvent) => {
      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const activeIndex = sortedTags.findIndex((t) => t.id === active.id);
      const overIndex = sortedTags.findIndex((t) => t.id === over.id);
      if (activeIndex === -1 || overIndex === -1) return;

      const newOrder = arrayMove(
        sortedTags.map((t) => t.id),
        activeIndex,
        overIndex,
      );

      await onReorder(newOrder);
    },
    [sortedTags, onReorder],
  );

  return {
    sortedTags,
    sensors,
    handleDragEnd,
    DndContext,
    SortableContext,
    verticalListSortingStrategy,
    closestCenter,
  };
}
