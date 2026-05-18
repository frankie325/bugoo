import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Separator } from "@heroui/react";

export interface ContextMenuState {
  tagId: string;
  x: number;
  y: number;
}

interface TagContextMenuProps {
  state: ContextMenuState;
  onClose: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onCreateAbove: () => void;
  onCreateBelow: () => void;
}

export function TagContextMenu({
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