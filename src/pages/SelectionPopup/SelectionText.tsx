import type { ResolvedWord } from "../../lib/api";
import type { TagItem } from "../../types/tag";
import type { SelectionPopupState } from "./selectionPopupState";
import { SelectionPopupCard } from "./components/SelectionPopupCard";

type SelectionTextProps = {
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

export function SelectionText(props: SelectionTextProps) {
  return <SelectionPopupCard {...props} />;
}
