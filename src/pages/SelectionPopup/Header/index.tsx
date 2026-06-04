import { Button } from "@heroui/react";
import { Star, Volume2, X } from "lucide-react";
import { MoreActionsPopover } from "./MoreAction";

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
          <h2 className="text-foreground truncate text-base font-semibold leading-6">{word}</h2>
          <Button variant="ghost" size="sm" isIconOnly aria-label="播放发音" onPress={onSpeak}>
            <Volume2 className="text-accent size-4" aria-hidden="true" />
          </Button>
        </div>
        {phonetic && (
          <p className="text-muted mt-0.5 text-xs">{phonetic}</p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-0.5">
        <Button variant="ghost" size="sm" isIconOnly aria-label="收藏">
          <Star className="text-muted size-4" aria-hidden="true" />
        </Button>
        <MoreActionsPopover
          onRetry={onRetry}
          onOpenMainWindow={onOpenMainWindow}
          onCopyFeedback={onCopyFeedback}
        />
        <Button variant="ghost" size="sm" isIconOnly aria-label="关闭" onPress={onHideWord}>
          <X className="text-muted size-4" aria-hidden="true" />
        </Button>
      </div>
    </div>
  );
}

