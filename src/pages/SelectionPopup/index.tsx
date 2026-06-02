import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { SelectionText } from "./SelectionText";
import {
  addWord,
  createTag,
  getTags,
  resolveWord,
  speakText,
  type ResolvedWord,
} from "../../lib/api";
import type { TagItem } from "../../types/tag";
import { getSelectionPopupState } from "./selectionPopupState";
import { TagSelectorPopover } from "./components/TagSelectorPopover";

const TEXT_UPDATED_EVENT = "selection-popup://text-updated";
const CLOSE_POPUP_COMMAND = "close_selection_popup";
const GET_POPUP_TEXT_COMMAND = "get_selection_popup_text";
const CURSOR_INSIDE_POPUP_COMMAND = "is_cursor_inside_selection_popup";
const OPEN_FLOAT_WINDOW_COMMAND = "open_float_window";
const AUTO_CLOSE_DELAY_MS = 2000;

export function SelectionPopupPage() {
  const location = useLocation();
  const initialText = useMemo(() => {
    const params = new URLSearchParams(location.search);
    return params.get("text") ?? "";
  }, [location.search]);
  const [text, setText] = useState(initialText);
  const [resolvedWord, setResolvedWord] = useState<ResolvedWord | null>(null);
  const [isResolving, setIsResolving] = useState(false);
  const [isSavingWord, setIsSavingWord] = useState(false);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [tags, setTags] = useState<TagItem[]>([]);
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);
  const [isTagSelectorOpen, setIsTagSelectorOpen] = useState(false);
  const resolveRequestIdRef = useRef(0);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearCloseTimer = useCallback(() => {
    if (closeTimerRef.current) {
      clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }
  }, []);

  const closePopup = useCallback(() => {
    invoke(CLOSE_POPUP_COMMAND).catch((error) => {
      console.warn("Failed to close selection popup", error);
    });
  }, []);

  const isCursorInsidePopup = useCallback(async () => {
    try {
      return await invoke<boolean>(CURSOR_INSIDE_POPUP_COMMAND);
    } catch (error) {
      console.warn("Failed to read cursor position for selection popup", error);
      return false;
    }
  }, []);

  const hydrateLatestText = useCallback(async () => {
    try {
      const latestText = await invoke<string | null>(GET_POPUP_TEXT_COMMAND);
      if (latestText) {
        setText(latestText);
      }
    } catch (error) {
      console.warn("Failed to hydrate selection popup text", error);
    }
  }, []);

  const scheduleAutoClose = useCallback((delayMs = AUTO_CLOSE_DELAY_MS) => {
    clearCloseTimer();
    closeTimerRef.current = setTimeout(async () => {
      const isInside = await isCursorInsidePopup();
      if (isInside) {
        scheduleAutoClose(AUTO_CLOSE_DELAY_MS);
        return;
      }
      closePopup();
    }, delayMs);
  }, [clearCloseTimer, closePopup, isCursorInsidePopup]);

  useEffect(() => {
    setText(initialText);
  }, [initialText]);

  useEffect(() => {
    setSelectedTagIds([]);
    setIsTagSelectorOpen(false);
  }, [text]);

  useEffect(() => {
    getTags()
      .then((loaded) => {
        setTags(loaded);
      })
      .catch((error) => {
        console.warn("Failed to load selection popup tags", error);
      });
  }, []);

  useEffect(() => {
    const trimmed = text.trim();
    if (!trimmed) {
      setResolvedWord(null);
      setResolveError(null);
      return;
    }

    const requestId = resolveRequestIdRef.current + 1;
    resolveRequestIdRef.current = requestId;
    setResolvedWord(null);
    setIsResolving(true);
    setResolveError(null);

    resolveWord(trimmed)
      .then((result) => {
        if (resolveRequestIdRef.current === requestId) {
          setResolvedWord(result);
        }
      })
      .catch((error) => {
        if (resolveRequestIdRef.current === requestId) {
          setResolvedWord(null);
          setResolveError(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (resolveRequestIdRef.current === requestId) {
          setIsResolving(false);
        }
      });
  }, [text]);

  const popupState = getSelectionPopupState({
    text,
    resolvedWord,
    isResolving,
    resolveError,
  });

  const selectedTags = useMemo(
    () => tags.filter((tag) => selectedTagIds.includes(tag.id)),
    [selectedTagIds, tags],
  );

  const handleAddWord = useCallback(async () => {
    if (!resolvedWord || resolvedWord.wordId) {
      return;
    }

    setIsSavingWord(true);
    try {
      const selectedTagNames = selectedTags.map((tag) => tag.name).join(",");
      const saved = await addWord({
        word: resolvedWord.word,
        translation: resolvedWord.translation,
        sourceLang: resolvedWord.sourceLang,
        targetLang: resolvedWord.targetLang,
        phonetic: resolvedWord.phonetic,
        meanings: resolvedWord.meanings,
        englishDefinitions: resolvedWord.englishDefinitions,
        examples: resolvedWord.examples,
        wordForms: resolvedWord.wordForms,
        memoryTip: resolvedWord.memoryTip,
        tags: selectedTagNames,
      });
      setResolvedWord({
        wordId: saved.wordId,
        word: saved.word,
        translation: saved.translation,
        detectedSourceLang: resolvedWord.detectedSourceLang,
        sourceLang: resolvedWord.sourceLang,
        targetLang: resolvedWord.targetLang,
        phonetic: saved.phonetic,
        meanings: saved.meanings,
        englishDefinitions: saved.englishDefinitions,
        examples: saved.examples,
        wordForms: saved.wordForms,
        memoryTip: saved.memoryTip,
      });
    } finally {
      setIsSavingWord(false);
    }
  }, [resolvedWord, selectedTags]);

  const handleRetry = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed) {
      return;
    }
    setText(trimmed);
    resolveRequestIdRef.current += 1;
    const requestId = resolveRequestIdRef.current;
    setResolvedWord(null);
    setIsResolving(true);
    setResolveError(null);

    resolveWord(trimmed)
      .then((result) => {
        if (resolveRequestIdRef.current === requestId) {
          setResolvedWord(result);
        }
      })
      .catch((error) => {
        if (resolveRequestIdRef.current === requestId) {
          setResolvedWord(null);
          setResolveError(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (resolveRequestIdRef.current === requestId) {
          setIsResolving(false);
        }
      });
  }, [text]);

  const handleCopy = useCallback(async () => {
    const content = resolvedWord
      ? `${resolvedWord.word}\n${resolvedWord.translation}`
      : text.trim();
    if (content) {
      await writeText(content);
    }
  }, [resolvedWord, text]);

  const handleSpeak = useCallback(() => {
    const speechText = resolvedWord?.word || text.trim();
    if (speechText) {
      void speakText(speechText, resolvedWord?.sourceLang);
    }
  }, [resolvedWord, text]);

  const handleOpenMainWindow = useCallback(() => {
    invoke(OPEN_FLOAT_WINDOW_COMMAND).catch((error) => {
      console.warn("Failed to open main window from selection popup", error);
    });
  }, []);

  const handleHideWord = useCallback(() => {
    const trimmed = text.trim();
    if (trimmed) {
      sessionStorage.setItem(`bugoo:hidden-selection:${trimmed}`, String(Date.now()));
    }
    closePopup();
  }, [closePopup, text]);

  const handleCopyFeedback = useCallback(async () => {
    const payload = JSON.stringify(
      {
        text: text.trim(),
        resolvedWord,
        error: resolveError,
      },
      null,
      2,
    );
    await writeText(payload);
  }, [resolveError, resolvedWord, text]);

  const handleToggleTag = useCallback((tagId: string) => {
    setSelectedTagIds((current) =>
      current.includes(tagId)
        ? current.filter((id) => id !== tagId)
        : [...current, tagId],
    );
  }, []);

  const handleCreateTag = useCallback(async () => {
    const nextTag = await createTag({
      name: `标签 ${tags.length + 1}`,
      color: "#22C55E",
      sort_order: tags.length,
    });
    setTags((current) => [...current, nextTag]);
    setSelectedTagIds((current) => [...current, nextTag.id]);
  }, [tags.length]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    listen<string>(TEXT_UPDATED_EVENT, (event) => {
      setText(event.payload);
      scheduleAutoClose();
    })
      .then((dispose) => {
        if (disposed) {
          dispose();
        } else {
          unlisten = dispose;
        }
        void hydrateLatestText();
      })
      .catch((error) => {
        console.warn("Failed to listen for selection popup updates", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
      clearCloseTimer();
    };
  }, [clearCloseTimer, hydrateLatestText, scheduleAutoClose]);

  useEffect(() => {
    scheduleAutoClose();
    return clearCloseTimer;
  }, [clearCloseTimer, scheduleAutoClose]);

  return (
    <main className="flex min-h-screen items-center justify-center bg-transparent p-2">
      <SelectionText
        text={text}
        state={popupState}
        resolvedWord={resolvedWord}
        selectedTags={selectedTags}
        isSavingWord={isSavingWord}
        onRetry={handleRetry}
        onCopy={handleCopy}
        onSpeak={handleSpeak}
        onAddWord={handleAddWord}
        onOpenMainWindow={handleOpenMainWindow}
        onHideWord={handleHideWord}
        onCopyFeedback={handleCopyFeedback}
        onOpenTagSelector={() => setIsTagSelectorOpen(true)}
      />
      <TagSelectorPopover
        isOpen={isTagSelectorOpen}
        tags={tags}
        selectedTagIds={selectedTagIds}
        onOpenChange={setIsTagSelectorOpen}
        onToggleTag={handleToggleTag}
        onCreateTag={handleCreateTag}
      />
    </main>
  );
}
