import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { SelectionText } from "./SelectionText";

const TEXT_UPDATED_EVENT = "selection-popup://text-updated";
const CLOSE_POPUP_COMMAND = "close_selection_popup";
const GET_POPUP_TEXT_COMMAND = "get_selection_popup_text";
const CURSOR_INSIDE_POPUP_COMMAND = "is_cursor_inside_selection_popup";
const AUTO_CLOSE_DELAY_MS = 2000;

export function SelectionPopupPage() {
  const location = useLocation();
  const initialText = useMemo(() => {
    const params = new URLSearchParams(location.search);
    return params.get("text") ?? "";
  }, [location.search]);
  const [text, setText] = useState(initialText);
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
      <SelectionText text={text} />
    </main>
  );
}
