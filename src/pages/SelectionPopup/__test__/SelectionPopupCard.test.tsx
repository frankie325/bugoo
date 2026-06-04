import { render, screen } from "@testing-library/react";

// Polyfill ResizeObserver for jsdom (required by HeroUI v3 ScrollShadow and Tooltip)
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
}
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { MeaningList } from "../components/MeaningList";
import { ExamplePreview } from "../components/ExamplePreview";
import { LoadingState } from "../components/LoadingState";
import { ErrorState } from "../components/ErrorState";
import { ReviewStatusCard } from "../components/ReviewStatusCard";
import { PopupFooter } from "../Footer";
import { MoreActionsPopover } from "../Header/MoreAction";

describe("selection popup presentational pieces", () => {
  it("renders meanings grouped by part of speech", () => {
    render(<MeaningList meanings={[{ partOfSpeech: "noun", translations: ["意外发现", "好运"] }]} />);

    expect(screen.getByText("[noun]")).toBeTruthy();
    expect(screen.getByText("意外发现；好运")).toBeTruthy();
  });

  it("renders all example sentences and translations", () => {
    render(
      <ExamplePreview
        examples={[
          { sentence: "I found this job by pure serendipity.", translation: "我纯粹是意外得到这份工作的。" },
          { sentence: "What a fortunate coincidence.", translation: "真是幸运的巧合。" },
        ]}
      />,
    );

    expect(screen.getByText("例句")).toBeTruthy();
    expect(screen.getByText("I found this job by pure serendipity.")).toBeTruthy();
    expect(screen.getByText("我纯粹是意外得到这份工作的。")).toBeTruthy();
    expect(screen.getByText("What a fortunate coincidence.")).toBeTruthy();
    expect(screen.getByText("真是幸运的巧合。")).toBeTruthy();
  });

  it("renders loading skeleton labels", () => {
    const { container } = render(<LoadingState />);

    expect(container.querySelector('[aria-label="正在加载翻译结果"]')).toBeTruthy();
  });

  it("renders retry button in error state", () => {
    render(<ErrorState title="翻译失败" description="请稍后重试" actionLabel="重试" onAction={() => undefined} />);

    expect(screen.getByText("翻译失败")).toBeTruthy();
    expect(screen.getByRole("button", { name: "重试" })).toBeTruthy();
  });

  it("renders saved review state", () => {
    render(<ReviewStatusCard nextReviewText="明天 18:00" />);

    expect(screen.getByText("已在生词本中")).toBeTruthy();
    expect(screen.getByText("下次复习：明天 18:00")).toBeTruthy();
  });

  it("runs footer actions", async () => {
    const user = userEvent.setup();
    const calls: string[] = [];

    render(
      <PopupFooter
        isSaved={false}
        isSavingWord={false}
        canAddWord
        onCopy={() => calls.push("copy")}
        onSpeak={() => calls.push("speak")}
        onAddWord={() => calls.push("add")}
      />,
    );

    await user.click(screen.getByRole("button", { name: "复制" }));
    await user.click(screen.getByRole("button", { name: "发音" }));
    await user.click(screen.getByRole("button", { name: "加入生词本" }));

    expect(calls).toEqual(["copy", "speak", "add"]);
  });

  it("renders saved footer state", () => {
    render(
      <PopupFooter
        isSaved
        isSavingWord={false}
        canAddWord={false}
        onCopy={() => undefined}
        onSpeak={() => undefined}
        onAddWord={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "已加入" })).toBeTruthy();
  });

  it("renders the more actions trigger", () => {
    render(
      <MoreActionsPopover
        onRetry={() => undefined}
        onOpenMainWindow={() => undefined}
        onCopyFeedback={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "更多操作" })).toBeTruthy();
  });
});
