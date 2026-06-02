import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { MeaningList } from "../components/MeaningList";
import { ExamplePreview } from "../components/ExamplePreview";
import { LoadingState } from "../components/LoadingState";
import { ErrorState } from "../components/ErrorState";
import { ReviewStatusCard } from "../components/ReviewStatusCard";

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
});
