import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Card, ScrollShadow } from "@heroui/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import {
  addWord,
  getTags,
  resolveWord,
  speakText,
  type ResolvedWord,
} from "../../lib/api";
import type { TagItem } from "../../types/tag";
import { getSelectionPopupState } from "./selectionPopupState";
import { ErrorState } from "./components/ErrorState";
import { ExamplePreview } from "./components/ExamplePreview";
import { LoadingState } from "./components/LoadingState";
import { MeaningList } from "./components/MeaningList";
import { PopupFooter } from "./Footer";
import { PopupHeader } from "./Header";
import { TagChipList } from "./components/TagChipList";
import { useSelectionPopupResize } from "./useSelectionPopupResize";

const TEXT_UPDATED_EVENT = "selection-popup://text-updated";
const CLOSE_POPUP_COMMAND = "close_selection_popup";
const CONTENT_READY_COMMAND = "selection_popup_content_ready";
const GET_POPUP_TEXT_COMMAND = "get_selection_popup_text";
const OPEN_FLOAT_WINDOW_COMMAND = "open_float_window";

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
  const resolveRequestIdRef = useRef(0);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const headerRef = useRef<HTMLDivElement | null>(null);
  const middleContentRef = useRef<HTMLDivElement | null>(null);
  const footerRef = useRef<HTMLDivElement | null>(null);

  const closePopup = useCallback(() => {
    invoke(CLOSE_POPUP_COMMAND).catch((error) => {
      console.warn("Failed to close selection popup", error);
    });
  }, []);

  const notifyContentReady = useCallback((readyText: string) => {
    invoke(CONTENT_READY_COMMAND, { text: readyText }).catch((error) => {
      console.warn("Failed to notify selection popup content ready", error);
    });
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

  useEffect(() => {
    setText(initialText);
  }, [initialText]);

  useEffect(() => {
    setSelectedTagIds([]);
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
    const currentText = text;
    const trimmed = currentText.trim();
    if (!trimmed) {
      setResolvedWord(null);
      setResolveError(null);
      notifyContentReady(currentText);
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
          notifyContentReady(currentText);
        }
      });
  }, [notifyContentReady, text]);

  const popupState = getSelectionPopupState({
    text,
    resolvedWord,
    isResolving,
    resolveError,
  });

  useSelectionPopupResize({
    popupState,
    resolvedWord,
    resolveError,
    text,
    containerRef,
    headerRef,
    middleContentRef,
    footerRef,
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
    const currentText = text;
    const trimmed = currentText.trim();
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
          notifyContentReady(currentText);
        }
      });
  }, [notifyContentReady, text]);

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

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    listen<string>(TEXT_UPDATED_EVENT, (event) => {
      setText(event.payload);
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
    };
  }, [hydrateLatestText]);

  const displayText = resolvedWord?.word || text.trim() || "未读取到选中文本";
  const isSaved = Boolean(resolvedWord?.wordId);
  // "加入生词本" 按钮可用条件：解析完成且有结果且未保存。
  // 与翻译进度解耦——解析中按钮 disabled，解析完成且未保存则可用。
  const canAddWord = !isResolving && Boolean(resolvedWord) && !isSaved;

  return (
    <main className="h-full w-full p-0 bg-surface">
      <Card className="flex h-full w-full flex-col rounded-[12px] bg-white p-0 shadow-xl">
        <Card.Content className="flex h-full flex-col p-0">
          <div ref={containerRef} className="flex h-full flex-col gap-0 p-3">
            <div ref={headerRef} className="shrink-0">
              <PopupHeader
                word={displayText}
                phonetic={resolvedWord?.phonetic ?? null}
                onSpeak={handleSpeak}
                onRetry={handleRetry}
                onOpenMainWindow={handleOpenMainWindow}
                onHideWord={handleHideWord}
                onCopyFeedback={handleCopyFeedback}
              />
            </div>

            <ScrollShadow
              className="flex min-h-0 flex-1 flex-col my-2"
              hideScrollBar
            >
              <div
                ref={middleContentRef}
                className="flex flex-col"
                data-testid="selection-popup-middle-content"
              >
                {popupState === "loading" ? (
                  <LoadingState />
                ) : popupState === "empty" ? (
                  <ErrorState
                    title="无选区结果"
                    description="请重新选择需要翻译的内容"
                    actionLabel="关闭"
                    onAction={handleHideWord}
                  />
                ) : popupState === "tooLong" ? (
                  <ErrorState
                    title="内容过长"
                    description="划词弹窗适合翻译 50 个字符以内的短文本"
                    actionLabel="重新翻译"
                    onAction={handleRetry}
                  />
                ) : popupState === "error" ? (
                  <ErrorState
                    title="翻译失败"
                    description="请检查网络或稍后重试"
                    actionLabel="重试"
                    onAction={handleRetry}
                  />
                ) : resolvedWord ? (
                  <>
                    <p className="text-foreground text-sm font-medium leading-6">
                      {resolvedWord.translation}
                    </p>
                    <MeaningList meanings={resolvedWord.meanings} />
                    <ExamplePreview
                      examples={resolvedWord.examples}
                      highlightText={text.trim()}
                    />
                  </>
                ) : null}
              </div>
            </ScrollShadow>

            <div ref={footerRef} className="shrink-0">
              {resolvedWord ? (
                <TagChipList
                  tags={tags}
                  selectedTags={selectedTags}
                  selectedTagIds={selectedTagIds}
                  onToggleTag={handleToggleTag}
                />
              ) : null}

              <PopupFooter
                isSaved={isSaved}
                isSavingWord={isSavingWord}
                canAddWord={canAddWord}
                onCopy={handleCopy}
                onSpeak={handleSpeak}
                onAddWord={handleAddWord}
              />
            </div>
          </div>
        </Card.Content>
      </Card>
    </main>
  );
}
