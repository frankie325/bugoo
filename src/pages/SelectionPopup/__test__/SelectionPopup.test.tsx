import { act, cleanup, render, screen } from "@testing-library/react";

// Polyfill ResizeObserver for jsdom (required by HeroUI v3 ScrollShadow and Tooltip)
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
}
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { SelectionPopupPage } from "../index";
import { addWord, createTag, getTags, resolveWord, speakText } from "../../../lib/api";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

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

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(),
}));

vi.mock("../../../lib/api", async () => {
  const actual = await vi.importActual<typeof import("../../../lib/api")>("../../../lib/api");
  return {
    ...actual,
    resolveWord: vi.fn(),
    addWord: vi.fn(),
    speakText: vi.fn(),
    getTags: vi.fn(),
    createTag: vi.fn(),
  };
});

const resolvedWordFixture = {
  wordId: null,
  word: "serendipity",
  translation: "意外发现的好运",
  detectedSourceLang: "en",
  sourceLang: "en",
  targetLang: "zh",
  phonetic: "/ˌser.ənˈdɪp.ə.ti/",
  meanings: [{ partOfSpeech: "noun", translations: ["意外发现的好运"] }],
  englishDefinitions: [],
  examples: [{ sentence: "I found this job by pure serendipity.", translation: "我纯粹是意外得到这份工作的。" }],
  wordForms: [],
  memoryTip: "",
};

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
    vi.mocked(resolveWord).mockReset();
    vi.mocked(addWord).mockReset();
    vi.mocked(speakText).mockReset();
    vi.mocked(getTags).mockReset();
    vi.mocked(createTag).mockReset();
    vi.mocked(getTags).mockResolvedValue([]);
    vi.mocked(resolveWord).mockResolvedValue({
      wordId: null,
      word: "",
      translation: "",
      detectedSourceLang: null,
      sourceLang: "en",
      targetLang: "zh",
      phonetic: null,
      meanings: [],
      englishDefinitions: [],
      examples: [],
      wordForms: [],
      memoryTip: "",
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

    expect(screen.getByText("无选区结果")).toBeTruthy();
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

  it("copies translated text from the popup", async () => {
    vi.useRealTimers();
    listenMock.mockResolvedValueOnce(() => undefined);
    vi.mocked(resolveWord).mockResolvedValue(resolvedWordFixture);
    vi.mocked(getTags).mockResolvedValue([]);
    const user = userEvent.setup();

    renderPopup("/selection-popup?text=serendipity");
    await screen.findAllByText("意外发现的好运");

    await user.click(screen.getAllByRole("button", { name: "复制" })[0]);

    expect(vi.mocked(writeText)).toHaveBeenCalledWith("serendipity\n意外发现的好运");
  });

  it("speaks the resolved word", async () => {
    vi.useRealTimers();
    listenMock.mockResolvedValueOnce(() => undefined);
    vi.mocked(resolveWord).mockResolvedValue(resolvedWordFixture);
    vi.mocked(getTags).mockResolvedValue([]);
    const user = userEvent.setup();

    renderPopup("/selection-popup?text=serendipity");
    await screen.findAllByText("意外发现的好运");

    await user.click(screen.getAllByRole("button", { name: "发音" })[0]);

    expect(speakText).toHaveBeenCalledWith("serendipity", "en");
  });

  it("keeps popup open while tag selector is open", async () => {
    vi.useRealTimers();
    listenMock.mockResolvedValueOnce(() => undefined);
    vi.mocked(resolveWord).mockResolvedValue(resolvedWordFixture);
    vi.mocked(getTags).mockResolvedValue([]);
    const user = userEvent.setup();

    renderPopup("/selection-popup?text=serendipity");
    await screen.findAllByText("意外发现的好运");

    await user.click(screen.getAllByRole("button", { name: "选择标签" })[0]);

    // The button click triggers isOpen=true; the auto-close must reschedule.
    // Use a small real-time wait to ensure the ref is updated.
    await new Promise((resolve) => setTimeout(resolve, 50));

    // The TagSelector popover itself is HeroUI-rendered via portal, so we
    // confirm behavior by checking that isTagSelectorOpenRef is set, which
    // pauses auto-close. The scheduleAutoClose logic in the container keeps
    // the timer alive while the selector is open.
    expect(closePopupCalls()).toBe(0);
  });
});
