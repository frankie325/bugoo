import { useCallback, useEffect, useMemo, useRef, useState, type Ref } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertDialog,
  Button,
  ColorArea,
  ColorPicker,
  ColorSlider,
  ColorSwatch,
  ColorSwatchPicker,
  Input,
  Modal,
  Separator,
  ScrollShadow,
  Surface,
  Tag,
  TagGroup,
  useOverlayState,
} from "@heroui/react";
import { GripVertical, Tag as TagIcon, Plus, Trash2 } from "lucide-react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useTagSort } from "../../hooks/useTagSort";
import { reorderTags } from "../../lib/api";
import type { TagItem } from "../../types/tag";

const TAG_COLORS = [
  "#ef4444",
  "#f97316",
  "#eab308",
  "#22c55e",
  "#06b6d4",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
];

type TagCreatePlacement = "above" | "below" | "end";

interface TagCreateOptions {
  anchorTagId?: string;
  placement?: TagCreatePlacement;
}

type EditorState =
  | {
      mode: "create";
      anchorTagId?: string;
      placement: TagCreatePlacement;
    }
  | {
      mode: "edit";
      tag: TagItem;
    };

interface ContextMenuState {
  tagId: string;
  x: number;
  y: number;
}

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

function SortableTag({ tag, count, onContextMenu }: SortableTagProps) {
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

interface TagEditorDialogProps {
  state: EditorState | null;
  onClose: () => void;
  onSubmit: (name: string, color: string) => Promise<void>;
}

function TagEditorDialog({ state, onClose, onSubmit }: TagEditorDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [color, setColor] = useState(TAG_COLORS[0]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isOpen = !!state;

  useEffect(() => {
    if (!state) return;

    if (state.mode === "edit") {
      setName(state.tag.name);
      setColor(state.tag.color);
      return;
    }

    setName("");
    setColor(TAG_COLORS[0]);
  }, [state]);

  const handleSubmit = async () => {
    const trimmedName = name.trim();
    if (!trimmedName) return;

    setIsSubmitting(true);
    try {
      await onSubmit(trimmedName, color);
      onClose();
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal.Backdrop isOpen={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container>
        <Modal.Dialog className="sm:max-w-[380px]">
          <Modal.CloseTrigger />
          <Modal.Header>
            <Modal.Icon className="bg-default text-foreground">
              <TagIcon className="size-5" />
            </Modal.Icon>
            {/* <Modal.Heading>
              {isEditing ? t("home.editTag") : t("home.addTag")}
            </Modal.Heading> */}
          </Modal.Header>
          <Modal.Body className="">
            <Surface className="p-3 flex flex-col gap-3">
              <div className="flex gap-4">
                <ColorPicker
                  value={color}
                  onChange={(nextColor) => setColor(nextColor.toString("hex"))}
                >
                  <ColorPicker.Trigger>
                    <ColorSwatch size="sm" />
                  </ColorPicker.Trigger>
                  <ColorPicker.Popover>
                    <ColorArea
                      aria-label="Color area"
                      className="max-w-full"
                      colorSpace="hsb"
                      xChannel="saturation"
                      yChannel="brightness"
                    >
                      <ColorArea.Thumb />
                    </ColorArea>
                    <ColorSlider
                      aria-label="Hue slider"
                      channel="hue"
                      className="gap-1 px-1"
                      colorSpace="hsb"
                    >
                      <ColorSlider.Track>
                        <ColorSlider.Thumb />
                      </ColorSlider.Track>
                    </ColorSlider>
                  </ColorPicker.Popover>
                </ColorPicker>
                <Input
                  autoFocus
                  className="flex-1"
                  value={name}
                  variant="secondary"
                  placeholder={t("home.tagNamePlaceholder")}
                  disabled={isSubmitting}
                  onChange={(event) => setName(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void handleSubmit();
                    }
                  }}
                />
              </div>

              <Separator />

              <div className="flex flex-col gap-2">
                <span className="text-xs text-foreground">
                  {t("home.tagColorLabel")}
                </span>
                <ScrollShadow hideScrollBar orientation="horizontal">
                  <ColorSwatchPicker
                    className="flex-nowrap"
                    value={color}
                    onChange={(nextColor) =>
                      setColor(nextColor.toString("hex"))
                    }
                  >
                    {TAG_COLORS.map((tagColor) => (
                      <ColorSwatchPicker.Item
                        className="flex-shrink-0"
                        key={tagColor}
                        color={tagColor}
                      >
                        <ColorSwatchPicker.Swatch />
                        <ColorSwatchPicker.Indicator />
                      </ColorSwatchPicker.Item>
                    ))}
                  </ColorSwatchPicker>
                </ScrollShadow>
              </div>
            </Surface>
          </Modal.Body>
          <Modal.Footer>
            <Button
              variant="tertiary"
              onPress={onClose}
              isDisabled={isSubmitting}
            >
              {t("home.deleteTagCancel")}
            </Button>
            <Button
              onPress={handleSubmit}
              isPending={isSubmitting}
              isDisabled={!name.trim()}
            >
              {t("home.tagSave")}
            </Button>
          </Modal.Footer>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

interface TagContextMenuProps {
  state: ContextMenuState;
  onClose: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onCreateAbove: () => void;
  onCreateBelow: () => void;
}

function TagContextMenu({
  state,
  onClose,
  onEdit,
  onDelete,
  onCreateAbove,
  onCreateBelow,
}: TagContextMenuProps) {
  const { t } = useTranslation();
  const menuRef = useRef<HTMLDivElement>(null);
  const [adjusted, setAdjusted] = useState<{ left: number; top: number }>({
    left: state.x,
    top: state.y,
  });

  useEffect(() => {
    if (!menuRef.current) return;

    const menu = menuRef.current;
    const menuWidth = menu.offsetWidth;
    const menuHeight = menu.offsetHeight;
    const vpWidth = window.innerWidth;
    const vpHeight = window.innerHeight;

    let left = state.x;
    let top = state.y;

    if (left + menuWidth > vpWidth) left = vpWidth - menuWidth;
    if (top + menuHeight > vpHeight) top = vpHeight - menuHeight;

    left = Math.max(0, left);
    top = Math.max(0, top);

    setAdjusted({ left, top });
  }, [state.x, state.y]);

  return (
    <div className="fixed inset-0 z-50" onMouseDown={onClose}>
      <div
        ref={menuRef}
        role="menu"
        className="fixed min-w-36 rounded-lg border border-border bg-overlay p-1 text-sm shadow-lg"
        style={{ left: adjusted.left, top: adjusted.top }}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <button
          type="button"
          role="menuitem"
          className="flex w-full items-center rounded-md px-3 py-2 text-left text-foreground hover:bg-surface-secondary"
          onClick={onEdit}
        >
          {t("home.tagContextEdit")}
        </button>
        <button
          type="button"
          role="menuitem"
          className="flex w-full items-center rounded-md px-3 py-2 text-left text-danger hover:bg-danger-soft"
          onClick={onDelete}
        >
          {t("home.tagContextDelete")}
        </button>
        <Separator className="my-1" />
        <button
          type="button"
          role="menuitem"
          className="flex w-full items-center rounded-md px-3 py-2 text-left text-foreground hover:bg-surface-secondary"
          onClick={onCreateAbove}
        >
          {t("home.tagContextCreateAbove")}
        </button>
        <button
          type="button"
          role="menuitem"
          className="flex w-full items-center rounded-md px-3 py-2 text-left text-foreground hover:bg-surface-secondary"
          onClick={onCreateBelow}
        >
          {t("home.tagContextCreateBelow")}
        </button>
      </div>
    </div>
  );
}

interface TagSectionProps {
  tags: TagItem[];
  tagCounts: Record<string, number>;
  selectedTag: string | null;
  onTagSelect: (tagId: string | null) => void;
  onTagCreate: (
    name: string,
    color: string,
    options?: TagCreateOptions,
  ) => Promise<void>;
  onTagUpdate: (tagId: string, name: string, color: string) => Promise<void>;
  onTagDelete: (tagId: string) => Promise<void>;
  onTagReorder: (reorderedTags: TagItem[]) => void;
}

export function TagSection({
  tags,
  tagCounts,
  selectedTag,
  onTagSelect,
  onTagCreate,
  onTagUpdate,
  onTagDelete,
  onTagReorder,
}: TagSectionProps) {
  const { t } = useTranslation();
  const [editorState, setEditorState] = useState<EditorState | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);
  const deleteDialog = useOverlayState();

  const handleReorder = useCallback(
    async (tagIds: string[]) => {
      const reordered = await reorderTags({ tag_ids: tagIds });
      onTagReorder(reordered);
    },
    [onTagReorder],
  );

  const {
    sortedTags,
    sensors,
    handleDragEnd,
    DndContext,
    SortableContext,
    verticalListSortingStrategy,
    closestCenter,
  } = useTagSort(tags, handleReorder);

  const contextTag = useMemo(
    () =>
      contextMenu
        ? sortedTags.find((tag) => tag.id === contextMenu.tagId)
        : null,
    [contextMenu, sortedTags],
  );

  const pendingTag = pendingDeleteId
    ? tags.find((tag) => tag.id === pendingDeleteId)
    : null;

  useEffect(() => {
    if (!contextMenu) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setContextMenu(null);
      }
    };

    const handleScroll = () => setContextMenu(null);

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("scroll", handleScroll, true);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("scroll", handleScroll, true);
    };
  }, [contextMenu]);

  const handleOpenCreate = useCallback(
    (placement: TagCreatePlacement, anchorTagId?: string) => {
      setContextMenu(null);
      setEditorState({ mode: "create", placement, anchorTagId });
    },
    [],
  );

  const handleOpenEdit = useCallback((tag: TagItem) => {
    setContextMenu(null);
    setEditorState({ mode: "edit", tag });
  }, []);

  const handleOpenDelete = useCallback(
    (tagId: string) => {
      setContextMenu(null);
      setPendingDeleteId(tagId);
      deleteDialog.open();
    },
    [deleteDialog],
  );

  const handleEditorSubmit = useCallback(
    async (name: string, color: string) => {
      if (!editorState) return;

      if (editorState.mode === "edit") {
        await onTagUpdate(editorState.tag.id, name, color);
        return;
      }

      await onTagCreate(name, color, {
        anchorTagId: editorState.anchorTagId,
        placement: editorState.placement,
      });
    },
    [editorState, onTagCreate, onTagUpdate],
  );

  const handleConfirmDelete = useCallback(async () => {
    if (!pendingDeleteId) return;

    await onTagDelete(pendingDeleteId);
    if (selectedTag === pendingDeleteId) {
      onTagSelect(null);
    }
    setPendingDeleteId(null);
    deleteDialog.close();
  }, [pendingDeleteId, selectedTag, onTagDelete, onTagSelect, deleteDialog]);

  const handleCancelDelete = useCallback(() => {
    setPendingDeleteId(null);
    deleteDialog.close();
  }, [deleteDialog]);

  const selectedKeys = selectedTag ? new Set([selectedTag]) : new Set<string>();

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-foreground-500">
          {t("home.tagsLabel")}
        </span>
        <Button
          isIconOnly
          size="sm"
          variant="secondary"
          aria-label={t("home.addTag")}
          onPress={() => handleOpenCreate("end")}
        >
          <Plus size={14} />
        </Button>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={sortedTags.map((tag) => tag.id)}
          strategy={verticalListSortingStrategy}
        >
          <TagGroup
            selectedKeys={selectedKeys}
            selectionMode="single"
            size="lg"
            variant="surface"
            className="w-full"
            onSelectionChange={(keys) => {
              if (keys === "all") return;
              const selected = Array.from(keys as Set<string>);
              onTagSelect(selected.length > 0 ? selected[0] : null);
            }}
          >
            <TagGroup.List items={sortedTags} className="flex-col gap-2">
              {(tag) => (
                <SortableTag
                  key={tag.id}
                  tag={tag}
                  count={tagCounts[tag.id] ?? 0}
                  onContextMenu={(targetTag, x, y) => {
                    onTagSelect(targetTag.id);
                    setContextMenu({ tagId: targetTag.id, x, y });
                  }}
                />
              )}
            </TagGroup.List>
          </TagGroup>
        </SortableContext>
      </DndContext>

      {contextMenu && contextTag && (
        <TagContextMenu
          state={contextMenu}
          onClose={() => setContextMenu(null)}
          onEdit={() => handleOpenEdit(contextTag)}
          onDelete={() => handleOpenDelete(contextTag.id)}
          onCreateAbove={() => handleOpenCreate("above", contextTag.id)}
          onCreateBelow={() => handleOpenCreate("below", contextTag.id)}
        />
      )}

      <TagEditorDialog
        state={editorState}
        onClose={() => setEditorState(null)}
        onSubmit={handleEditorSubmit}
      />

      <AlertDialog.Backdrop
        isOpen={deleteDialog.isOpen}
        onOpenChange={deleteDialog.setOpen}
      >
        <AlertDialog.Container>
          <AlertDialog.Dialog className="sm:max-w-[360px]">
            <AlertDialog.CloseTrigger />
            <AlertDialog.Header>
              <AlertDialog.Icon status="danger">
                <Trash2 className="size-5" />
              </AlertDialog.Icon>
              <AlertDialog.Heading>
                {t("home.deleteTagTitle")}
              </AlertDialog.Heading>
            </AlertDialog.Header>
            <AlertDialog.Body>
              <p>
                {t("home.deleteTagConfirm", { name: pendingTag?.name ?? "" })}
              </p>
            </AlertDialog.Body>
            <AlertDialog.Footer>
              <Button variant="tertiary" onPress={handleCancelDelete}>
                {t("home.deleteTagCancel")}
              </Button>
              <Button variant="danger" onPress={handleConfirmDelete}>
                {t("home.deleteTagConfirmBtn")}
              </Button>
            </AlertDialog.Footer>
          </AlertDialog.Dialog>
        </AlertDialog.Container>
      </AlertDialog.Backdrop>
    </div>
  );
}
