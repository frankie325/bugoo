import { Card } from "@heroui/react";
import type { ResolvedWord } from "../../../lib/api";
import type { TagItem } from "../../../types/tag";
import type { SelectionPopupState } from "../selectionPopupState";
import { ErrorState } from "./ErrorState";
import { ExamplePreview } from "./ExamplePreview";
import { LoadingState } from "./LoadingState";
import { MeaningList } from "./MeaningList";
import { PopupFooter } from "./PopupFooter";
import { PopupHeader } from "./PopupHeader";
import { ReviewStatusCard } from "./ReviewStatusCard";
import { TagChipList } from "./TagChipList";

type SelectionPopupCardProps = {
  text: string;
  state: SelectionPopupState;
  resolvedWord: ResolvedWord | null;
  selectedTags: TagItem[];
  isSavingWord: boolean;
  onRetry: () => void;
  onCopy: () => void;
  onSpeak: () => void;
  onAddWord: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
  onOpenTagSelector: () => void;
};

export function SelectionPopupCard({
  text,
  state,
  resolvedWord,
  selectedTags,
  isSavingWord,
  onRetry,
  onCopy,
  onSpeak,
  onAddWord,
  onOpenMainWindow,
  onHideWord,
  onCopyFeedback,
  onOpenTagSelector,
}: SelectionPopupCardProps) {
  const displayText = resolvedWord?.word || text.trim() || "未读取到选中文本";
  const isSaved = state === "saved";
  const canAddWord = state === "success" && Boolean(resolvedWord);

  return (
    <Card className="w-[320px] max-w-[320px] rounded-[12px] border border-[#E5E7EB] bg-white shadow-xl">
      <Card.Content className="flex max-h-[420px] flex-col gap-3 overflow-y-auto p-3">
        {state === "loading" ? (
          <LoadingState word={displayText} />
        ) : state === "empty" ? (
          <ErrorState title="无选区结果" description="请重新选择需要翻译的内容" actionLabel="关闭" onAction={onHideWord} />
        ) : state === "tooLong" ? (
          <ErrorState title="内容过长" description="划词弹窗适合翻译 50 个字符以内的短文本" actionLabel="重新翻译" onAction={onRetry} />
        ) : state === "error" ? (
          <ErrorState title="翻译失败" description="请检查网络或稍后重试" actionLabel="重试" onAction={onRetry} />
        ) : (
          <>
            <PopupHeader
              word={displayText}
              phonetic={resolvedWord?.phonetic ?? null}
              onSpeak={onSpeak}
              onRetry={onRetry}
              onOpenMainWindow={onOpenMainWindow}
              onHideWord={onHideWord}
              onCopyFeedback={onCopyFeedback}
            />
            {resolvedWord && (
              <>
                <p className="text-sm font-medium leading-6 text-[#111827]">
                  {resolvedWord.translation}
                </p>
                <MeaningList meanings={resolvedWord.meanings} />
                <ExamplePreview examples={resolvedWord.examples} />
                {isSaved ? (
                  <ReviewStatusCard nextReviewText="明天 18:00" />
                ) : (
                  <TagChipList selectedTags={selectedTags} onAddPress={onOpenTagSelector} />
                )}
              </>
            )}
            <PopupFooter
              isSaved={isSaved}
              isSavingWord={isSavingWord}
              canAddWord={canAddWord}
              onCopy={onCopy}
              onSpeak={onSpeak}
              onAddWord={onAddWord}
            />
          </>
        )}
      </Card.Content>
    </Card>
  );
}
