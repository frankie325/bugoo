import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, type RefObject } from "react";
import type { ResolvedWord } from "../../lib/api";
import type { SelectionPopupState } from "./selectionPopupState";

const RESIZE_POPUP_COMMAND = "resize_selection_popup";
const POPUP_DEFAULT_HEIGHT = 300;
const POPUP_MIN_HEIGHT = 250;
const POPUP_MAX_HEIGHT = 500;

type SelectionPopupResizeRefs = {
  containerRef: RefObject<HTMLElement | null>;
  headerRef: RefObject<HTMLElement | null>;
  middleContentRef: RefObject<HTMLElement | null>;
  footerRef: RefObject<HTMLElement | null>;
};

type UseSelectionPopupResizeInput = SelectionPopupResizeRefs & {
  popupState: SelectionPopupState;
  resolvedWord: ResolvedWord | null;
  resolveError: string | null;
  text: string;
};

function readElementHeight(element: HTMLElement | null) {
  return element?.getBoundingClientRect().height ?? 0;
}

function readVerticalPadding(element: HTMLElement | null) {
  if (!element) {
    return 0;
  }

  const style = window.getComputedStyle(element);
  return (
    Number.parseFloat(style.paddingTop || "0") +
    Number.parseFloat(style.paddingBottom || "0")
  );
}

function calculatePopupContentHeight({
  containerRef,
  headerRef,
  middleContentRef,
  footerRef,
}: SelectionPopupResizeRefs) {
  const measuredHeight = Math.ceil(
    readVerticalPadding(containerRef.current) +
      readElementHeight(headerRef.current) +
      readElementHeight(middleContentRef.current) +
      readElementHeight(footerRef.current),
  );

  return Math.min(Math.max(measuredHeight, POPUP_MIN_HEIGHT), POPUP_MAX_HEIGHT);
}

function canMeasurePopupContent({
  popupState,
  resolvedWord,
}: Pick<UseSelectionPopupResizeInput, "popupState" | "resolvedWord">) {
  if (popupState === "loading") {
    return true;
  }

  if (popupState === "success" || popupState === "saved") {
    return Boolean(resolvedWord);
  }

  return true;
}

export function useSelectionPopupResize({
  popupState,
  resolvedWord,
  resolveError,
  text,
  containerRef,
  headerRef,
  middleContentRef,
  footerRef,
}: UseSelectionPopupResizeInput) {
  const resizePopupToRenderedContent = useCallback(() => {
    const height =
      popupState === "loading"
        ? POPUP_DEFAULT_HEIGHT
        : calculatePopupContentHeight({
            containerRef,
            headerRef,
            middleContentRef,
            footerRef,
          });

    invoke(RESIZE_POPUP_COMMAND, { height }).catch((error) => {
      console.warn("Failed to resize selection popup", error);
    });
  }, [containerRef, footerRef, headerRef, middleContentRef, popupState]);

  useEffect(() => {
    if (!canMeasurePopupContent({ popupState, resolvedWord })) {
      return;
    }

    const frame = window.requestAnimationFrame(resizePopupToRenderedContent);

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [popupState, resizePopupToRenderedContent, resolvedWord, resolveError, text]);
}
