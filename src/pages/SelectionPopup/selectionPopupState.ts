import type { ResolvedWord } from "../../lib/api";

export type SelectionPopupState =
  | "empty"
  | "tooLong"
  | "loading"
  | "success"
  | "saved"
  | "error";

type GetSelectionPopupStateInput = {
  text: string;
  resolvedWord: ResolvedWord | null;
  isResolving: boolean;
  resolveError: string | null;
};

const MAX_SELECTION_LENGTH = 50;

export function getSelectionPopupState({
  text,
  resolvedWord,
  isResolving,
  resolveError,
}: GetSelectionPopupStateInput): SelectionPopupState {
  const trimmed = text.trim();

  if (!trimmed) {
    return "empty";
  }

  if (trimmed.length > MAX_SELECTION_LENGTH) {
    return "tooLong";
  }

  if (isResolving && !resolvedWord) {
    return "loading";
  }

  if (resolveError) {
    return "error";
  }

  if (resolvedWord?.wordId) {
    return "saved";
  }

  return "success";
}
