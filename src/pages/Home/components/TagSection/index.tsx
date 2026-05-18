import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertDialog, Button, TagGroup, useOverlayState } from "@heroui/react";
import { Plus, Trash2 } from "lucide-react";
import { useTagSort } from "@src/hooks/useTagSort";
import { reorderTags } from "@src/lib/api";
import type { TagCreatePlacement } from "@src//lib/tagSort";
import type { TagItem } from "@src/types/tag";
import { SortableTag } from "./SortableTag";
import { TagContextMenu, type ContextMenuState } from "./TagContextMenu";
import { TagEditorDialog, type EditorState } from "./TagEditorDialog";

interface TagCreateOptions {
  anchorTagId?: string;
  placement?: TagCreatePlacement;
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

export default function TagSection({
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
