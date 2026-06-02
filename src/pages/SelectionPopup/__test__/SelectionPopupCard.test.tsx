import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { MeaningList } from "../components/MeaningList";
import { ExamplePreview } from "../components/ExamplePreview";
import { LoadingState } from "../components/LoadingState";
import { ErrorState } from "../components/ErrorState";
import { ReviewStatusCard } from "../components/ReviewStatusCard";
import { PopupFooter } from "../components/PopupFooter";
import { MoreActionsPopover } from "../components/MoreActionsPopover";
import { SelectionPopupCard } from "../components/SelectionPopupCard";
import type { ResolvedWord } from "../../../lib/api";

const cardWord: ResolvedWord = {
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

describe("selection popup presentational pieces", () => {
  it("renders meanings grouped by part of speech", () => {
    render(<MeaningList meanings={[{ partOfSpeech: "noun", translations: ["意外发现", "好运"] }]} />);

    expect(screen.getByText("noun")).toBeTruthy();
    expect(screen.getByText("意外发现；好运")).toBeTruthy();
  });

  it("renders the first example sentence and translation", () => {
    render(<ExamplePreview examples={[{ sentence: "I found this job by pure serendipity.", translation: "我纯粹是意外得到这份工作的。" }]} />);

    expect(screen.getByText("例句")).toBeTruthy();
    expect(screen.getByText("I found this job by pure serendipity.")).toBeTruthy();
    expect(screen.getByText("我纯粹是意外得到这份工作的。")).toBeTruthy();
  });

  it("renders loading skeleton labels", () => {
    const { container } = render(<LoadingState word="serendipity" />);

    expect(screen.getByText("serendipity")).toBeTruthy();
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
        onHideWord={() => undefined}
        onCopyFeedback={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "更多操作" })).toBeTruthy();
  });

  it("renders full success card", () => {
    render(
      <SelectionPopupCard
        text="serendipity"
        state="success"
        resolvedWord={cardWord}
        selectedTags={[]}
        isSavingWord={false}
        onRetry={() => undefined}
        onCopy={() => undefined}
        onSpeak={() => undefined}
        onAddWord={() => undefined}
        onOpenMainWindow={() => undefined}
        onHideWord={() => undefined}
        onCopyFeedback={() => undefined}
        onOpenTagSelector={() => undefined}
      />,
    );

    expect(screen.getAllByText("serendipity").length).toBeGreaterThan(0);
    expect(screen.getAllByText("意外发现的好运").length).toBeGreaterThan(0);
    expect(screen.getAllByRole("button", { name: "加入生词本" }).length).toBeGreaterThan(0);
  });

  it("renders loading card", () => {
    const { container } = render(
      <SelectionPopupCard
        text="serendipity"
        state="loading"
        resolvedWord={null}
        selectedTags={[]}
        isSavingWord={false}
        onRetry={() => undefined}
        onCopy={() => undefined}
        onSpeak={() => undefined}
        onAddWord={() => undefined}
        onOpenMainWindow={() => undefined}
        onHideWord={() => undefined}
        onCopyFeedback={() => undefined}
        onOpenTagSelector={() => undefined}
      />,
    );

    expect(container.querySelector('[aria-label="正在加载翻译结果"]')).toBeTruthy();
  });
});
