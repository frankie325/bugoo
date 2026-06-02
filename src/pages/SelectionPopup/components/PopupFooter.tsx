import { Button } from "@heroui/react";
import { Check, Clipboard, Plus, Volume2 } from "lucide-react";

type PopupFooterProps = {
  isSaved: boolean;
  isSavingWord: boolean;
  canAddWord: boolean;
  onCopy: () => void;
  onSpeak: () => void;
  onAddWord: () => void;
};

export function PopupFooter({
  isSaved,
  isSavingWord,
  canAddWord,
  onCopy,
  onSpeak,
  onAddWord,
}: PopupFooterProps) {
  return (
    <div className="flex items-center justify-between gap-2 border-t border-[#E5E7EB] pt-2">
      <div className="flex items-center gap-1">
        <Button variant="ghost" size="sm" onPress={onCopy}>
          <Clipboard className="size-3.5" aria-hidden="true" />
          复制
        </Button>
        <Button variant="ghost" size="sm" onPress={onSpeak}>
          <Volume2 className="size-3.5 text-[#22C55E]" aria-hidden="true" />
          发音
        </Button>
      </div>
      <Button
        size="sm"
        className={isSaved ? "bg-[#DCFCE7] text-[#16A34A]" : "bg-[#22C55E] text-white"}
        isPending={isSavingWord}
        isDisabled={isSaved || !canAddWord}
        onPress={onAddWord}
      >
        {isSaved ? <Check className="size-3.5" aria-hidden="true" /> : <Plus className="size-3.5" aria-hidden="true" />}
        {isSaved ? "已加入" : "加入生词本"}
      </Button>
    </div>
  );
}
