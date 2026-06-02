import { Button } from "@heroui/react";
import { Star, Volume2 } from "lucide-react";
import { MoreActionsPopover } from "./MoreActionsPopover";

type PopupHeaderProps = {
  word: string;
  phonetic: string | null;
  onSpeak: () => void;
  onRetry: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
};

export function PopupHeader({
  word,
  phonetic,
  onSpeak,
  onRetry,
  onOpenMainWindow,
  onHideWord,
  onCopyFeedback,
}: PopupHeaderProps) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <h2 className="truncate text-base font-semibold leading-6 text-[#111827]">{word}</h2>
          <Button variant="ghost" size="sm" isIconOnly aria-label="播放发音" onPress={onSpeak}>
            <Volume2 className="size-4 text-[#22C55E]" aria-hidden="true" />
          </Button>
        </div>
        {phonetic && (
          <p className="mt-0.5 text-xs text-[#6B7280]">{phonetic}</p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-0.5">
        <Button variant="ghost" size="sm" isIconOnly aria-label="收藏">
          <Star className="size-4 text-[#6B7280]" aria-hidden="true" />
        </Button>
        <MoreActionsPopover
          onRetry={onRetry}
          onOpenMainWindow={onOpenMainWindow}
          onHideWord={onHideWord}
          onCopyFeedback={onCopyFeedback}
        />
      </div>
    </div>
  );
}
