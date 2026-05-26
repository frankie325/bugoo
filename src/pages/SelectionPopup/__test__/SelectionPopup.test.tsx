import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { SelectionPopupPage } from "../index";

const listenMock = vi.fn();
const invokeMock = vi.fn();
const CLOSE_POPUP_COMMAND = "close_selection_popup";
const CURSOR_INSIDE_POPUP_COMMAND = "is_cursor_inside_selection_popup";
const GET_POPUP_TEXT_COMMAND = "get_selection_popup_text";

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function renderPopup(initialEntry: string) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route path="/selection-popup" element={<SelectionPopupPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

async function flushMicrotasks() {
  await Promise.resolve();
}

async function flushPromiseChain() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function advance(ms: number) {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(ms);
  });
}

function closePopupCalls() {
  return invokeMock.mock.calls.filter(
    (call) => call[0] === CLOSE_POPUP_COMMAND,
  ).length;
}

describe("SelectionPopupPage", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    listenMock.mockReset();
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === CURSOR_INSIDE_POPUP_COMMAND) {
        return Promise.resolve(false);
      }
      return Promise.resolve(undefined);
    });
  });

  afterEach(() => {
    cleanup();
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it("renders selected text from the query string", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup?text=hello%20world");

    expect(screen.getByText("hello world")).toBeTruthy();
  });

  it("renders a safe empty state when text is missing", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup");

    expect(screen.getByText("未读取到选中文本")).toBeTruthy();
  });

  it("updates displayed text from the Tauri event", async () => {
    let eventHandler: ((event: { payload: string }) => void) | undefined;
    listenMock.mockImplementationOnce((_eventName, handler) => {
      eventHandler = handler as (event: { payload: string }) => void;
      return Promise.resolve(() => undefined);
    });

    renderPopup("/selection-popup?text=old");
    await flushMicrotasks();
    expect(eventHandler).toBeDefined();

    act(() => {
      eventHandler?.({ payload: "new text" });
    });
    expect(screen.getByText("new text")).toBeTruthy();
  });

  it("hydrates text from backend after listener registration", async () => {
    listenMock.mockResolvedValueOnce(() => undefined);
    invokeMock.mockImplementation((command: string) => {
      if (command === GET_POPUP_TEXT_COMMAND) {
        return Promise.resolve("first payload");
      }
      if (command === CURSOR_INSIDE_POPUP_COMMAND) {
        return Promise.resolve(false);
      }
      return Promise.resolve(undefined);
    });

    renderPopup("/selection-popup");
    await flushPromiseChain();

    expect(screen.getByText("first payload")).toBeTruthy();
  });

  it("auto closes after 2 seconds when mouse never enters", async () => {
    listenMock.mockResolvedValueOnce(() => undefined);
    renderPopup("/selection-popup?text=hello");
    await flushMicrotasks();

    await advance(2000);
    expect(invokeMock).toHaveBeenCalledWith(CLOSE_POPUP_COMMAND);
  });

  it("restarts auto close timer after text update event", async () => {
    let eventHandler: ((event: { payload: string }) => void) | undefined;
    listenMock.mockImplementationOnce((_eventName, handler) => {
      eventHandler = handler as (event: { payload: string }) => void;
      return Promise.resolve(() => undefined);
    });

    renderPopup("/selection-popup?text=old");
    await flushMicrotasks();
    expect(eventHandler).toBeDefined();

    await advance(1500);
    eventHandler?.({ payload: "new text" });
    await advance(1500);
    expect(closePopupCalls()).toBe(0);

    await advance(500);
    expect(invokeMock).toHaveBeenCalledWith(CLOSE_POPUP_COMMAND);
  });

  it("keeps popup open while backend reports cursor is inside", async () => {
    listenMock.mockResolvedValueOnce(() => undefined);
    invokeMock.mockImplementation((command: string) => {
      if (command === CURSOR_INSIDE_POPUP_COMMAND) {
        return Promise.resolve(true);
      }
      return Promise.resolve(undefined);
    });

    renderPopup("/selection-popup?text=hello");
    await flushMicrotasks();

    await advance(2000);
    expect(closePopupCalls()).toBe(0);
  });

  it("rechecks every 2 seconds while backend keeps reporting cursor inside", async () => {
    listenMock.mockResolvedValueOnce(() => undefined);
    invokeMock.mockImplementation((command: string) => {
      if (command === CURSOR_INSIDE_POPUP_COMMAND) {
        return Promise.resolve(true);
      }
      return Promise.resolve(undefined);
    });

    renderPopup("/selection-popup?text=hello");
    await flushMicrotasks();

    await advance(6000);

    expect(closePopupCalls()).toBe(0);
    const cursorChecks = invokeMock.mock.calls.filter(
      (call) => call[0] === CURSOR_INSIDE_POPUP_COMMAND,
    ).length;
    expect(cursorChecks).toBe(3);
  });

  it("closes on the next 2 second check when cursor leaves popup", async () => {
    listenMock.mockResolvedValueOnce(() => undefined);
    const cursorInsideResults = [true, false];
    invokeMock.mockImplementation((command: string) => {
      if (command === CURSOR_INSIDE_POPUP_COMMAND) {
        const result = cursorInsideResults.shift() ?? false;
        return Promise.resolve(result);
      }
      return Promise.resolve(undefined);
    });

    renderPopup("/selection-popup?text=hello");
    await flushMicrotasks();

    await advance(2000);
    expect(closePopupCalls()).toBe(0);

    await advance(2000);
    expect(invokeMock).toHaveBeenCalledWith(CLOSE_POPUP_COMMAND);
  });
});
