import { Button, Popover } from "@heroui/react";
import { BookOpen, MessageSquareWarning, MoreHorizontal, RefreshCw } from "lucide-react";

type MoreActionsPopoverProps = {
  onRetry: () => void;
  onOpenMainWindow: () => void;
  onCopyFeedback: () => void;
};

export function MoreActionsPopover({
  onRetry,
  onOpenMainWindow,
  onCopyFeedback,
}: MoreActionsPopoverProps) {
  return (
    <Popover>
      <Popover.Trigger>
        <Button variant="ghost" size="sm" isIconOnly aria-label="更多操作">
          <MoreHorizontal className="size-4" aria-hidden="true" />
        </Button>
      </Popover.Trigger>
      <Popover.Content placement="bottom end">
        <Popover.Dialog className="bg-background w-40 rounded-lg p-1 shadow-xl">
          <div role="menu" className="flex flex-col">
            <button role="menuitem" type="button" onClick={onRetry} className="text-foreground hover:bg-surface-secondary flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs">
              <RefreshCw className="size-3.5" aria-hidden="true" />
              重新翻译
            </button>
            <button role="menuitem" type="button" onClick={onOpenMainWindow} className="text-foreground hover:bg-surface-secondary flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs">
              <BookOpen className="size-3.5" aria-hidden="true" />
              打开主窗口
            </button>
            <button role="menuitem" type="button" onClick={onCopyFeedback} className="text-foreground hover:bg-surface-secondary flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs">
              <MessageSquareWarning className="size-3.5" aria-hidden="true" />
              反馈翻译问题
            </button>
          </div>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
