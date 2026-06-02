import { Button, Popover } from "@heroui/react";
import { BookOpen, EyeOff, MessageSquareWarning, MoreHorizontal, RefreshCw } from "lucide-react";

type MoreActionsPopoverProps = {
  onRetry: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
};

export function MoreActionsPopover({
  onRetry,
  onOpenMainWindow,
  onHideWord,
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
        <Popover.Dialog className="w-40 rounded-lg bg-white p-1 shadow-xl">
          <div role="menu" className="flex flex-col">
            <button role="menuitem" type="button" onClick={onRetry} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <RefreshCw className="size-3.5" aria-hidden="true" />
              重新翻译
            </button>
            <button role="menuitem" type="button" onClick={onOpenMainWindow} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <BookOpen className="size-3.5" aria-hidden="true" />
              打开主窗口
            </button>
            <button role="menuitem" type="button" onClick={onHideWord} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <EyeOff className="size-3.5" aria-hidden="true" />
              隐藏此词
            </button>
            <button role="menuitem" type="button" onClick={onCopyFeedback} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <MessageSquareWarning className="size-3.5" aria-hidden="true" />
              反馈翻译问题
            </button>
          </div>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
