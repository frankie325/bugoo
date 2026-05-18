import { useTranslation } from "react-i18next";
import { Tag } from "@heroui/react";
import { GripVertical } from "lucide-react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { TagItem } from "@src/types/tag";
import type { Ref } from "react";

interface SortableTagProps {
  tag: TagItem;
  count: number;
  onContextMenu: (tag: TagItem, x: number, y: number) => void;
}

function setRef<T>(ref: Ref<T> | undefined, value: T | null) {
  if (!ref) return;

  if (typeof ref === "function") {
    ref(value);
    return;
  }

  ref.current = value;
}

export function SortableTag({ tag, count, onContextMenu }: SortableTagProps) {
  const { t } = useTranslation();
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: tag.id });

  const sortableStyle = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.56 : 1,
  };

  const rowClassName = [
    "group relative flex min-h-10 w-full cursor-pointer items-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors",
    "data-[focus-visible=true]:outline-none data-[focus-visible=true]:ring-2 data-[focus-visible=true]:ring-accent/20",
    "data-[selected=true]:!border-border data-[selected=true]:!bg-surface-secondary data-[selected=true]:!text-foreground data-[selected=true]:shadow-sm data-[selected=true]:ring-1 data-[selected=true]:ring-border/60",
    "border-border/70 bg-surface text-foreground/85 shadow-none hover:bg-surface-secondary",
  ].join(" ");

  return (
    <Tag
      id={tag.id}
      textValue={tag.name}
      render={(props) => (
        <div
          {...props}
          ref={(node) => {
            setRef(props.ref, node);
            setNodeRef(node);
          }}
          className={`${props.className ?? ""} ${rowClassName}`}
          style={{ ...props.style, ...sortableStyle }}
          onContextMenu={(event) => {
            props.onContextMenu?.(event);
            if (event.defaultPrevented) return;
            event.preventDefault();
            onContextMenu(tag, event.clientX, event.clientY);
          }}
        />
      )}
    >
      <span
        className="size-2.5 shrink-0 rounded-full transition-transform group-data-[selected=true]:scale-110 group-data-[selected=true]:shadow-xs"
        style={{ backgroundColor: tag.color }}
      />
      <span className="min-w-0 flex-1 truncate font-medium">{tag.name}</span>
      <div className="flex justify-center items-center w-5">
        <span className="group-hover:hidden rounded-md border border-border/60 bg-default px-2 py-0.5 text-xs font-medium tabular-nums text-muted">
          {count}
        </span>
        <span
          ref={setActivatorNodeRef}
          className="hidden group-hover:flex size-5 shrink-0 cursor-grab items-center justify-center rounded-md text-muted opacity-0 transition-opacity hover:bg-default hover:text-foreground group-hover:opacity-100 active:cursor-grabbing"
          aria-label={t("home.tagDragHandle", { defaultValue: "拖动排序" })}
          {...attributes}
          {...listeners}
          onPointerDown={(event) => {
            listeners?.onPointerDown?.(event);
            event.stopPropagation();
          }}
          onMouseDown={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            listeners?.onKeyDown?.(event);
            event.stopPropagation();
          }}
          onClick={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.stopPropagation()}
        >
          <GripVertical className="size-3.5" />
        </span>
      </div>
    </Tag>
  );
}