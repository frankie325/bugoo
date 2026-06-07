# Selection Popup UI Interactions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按设计图实现划词翻译弹窗的前端界面与功能交互，让弹窗支持加载、成功、失败、已加入、生词复习状态，以及复制、发音、加入生词本、标签选择、更多操作和重试。

**Architecture:** 保留 `SelectionPopupPage` 作为数据与生命周期容器，继续使用当前 `resolveWord`、`addWord`、`speakText`、`getTags` 等 API。把弹窗展示拆到 `src/pages/SelectionPopup/components/`，并新增状态推导 helper，避免 UI 组件直接理解 Tauri 请求细节。HeroUI v3 组件必须参考本地 `.heroui-docs/react` 文档，使用 compound API，例如 `Card.Header`、`Card.Content`、`Card.Footer`、`Popover.Trigger`、`Popover.Content`、`Skeleton`。

**Tech Stack:** React 19 + TypeScript + HeroUI v3 + lucide-react + Tauri invoke + `@tauri-apps/plugin-clipboard-manager` + Vitest + Testing Library + pnpm。

---

## File Structure

- Modify: `src/pages/SelectionPopup/index.tsx`
  - 继续负责读取选中文本、调用翻译解析、自动关闭、保存生词。
  - 新增复制、发音、重试、标签选择、更多菜单交互回调。
- Modify: `src/pages/SelectionPopup/SelectionText.tsx`
  - 改为轻量入口组件，组合 `SelectionPopupCard` 并传入 view model。
- Create: `src/pages/SelectionPopup/selectionPopupState.ts`
  - 负责从 `text / resolvedWord / isResolving / resolveError` 推导 UI 状态。
- Create: `src/pages/SelectionPopup/components/SelectionPopupCard.tsx`
  - 弹窗卡片主体，按设计图组织 header/body/footer。
- Create: `src/pages/SelectionPopup/components/PopupHeader.tsx`
  - 单词标题、音标、发音、收藏、更多操作入口。
- Create: `src/pages/SelectionPopup/components/MeaningList.tsx`
  - 多词性释义展示，词性使用高亮 chip。
- Create: `src/pages/SelectionPopup/components/ExamplePreview.tsx`
  - 例句与译文展示。
- Create: `src/pages/SelectionPopup/components/PopupFooter.tsx`
  - 复制、发音、加入生词本、已加入按钮状态。
- Create: `src/pages/SelectionPopup/components/LoadingState.tsx`
  - 设计图中的骨架屏。
- Create: `src/pages/SelectionPopup/components/ErrorState.tsx`
  - 翻译失败、无效文本、内容过长等状态。
- Create: `src/pages/SelectionPopup/components/ReviewStatusCard.tsx`
  - 已在生词本中时展示复习状态。
- Create: `src/pages/SelectionPopup/components/MoreActionsPopover.tsx`
  - 更多菜单：重新翻译、打开主窗口、隐藏此词、反馈翻译问题。
- Create: `src/pages/SelectionPopup/components/TagSelectorPopover.tsx`
  - 标签选择面板，支持选择现有标签和快速创建标签。
- Create: `src/pages/SelectionPopup/components/TagChipList.tsx`
  - 弹窗卡片中的标签 chip 列表和添加入口。
- Test: `src/pages/SelectionPopup/__test__/selectionPopupState.test.ts`
  - 状态推导单元测试。
- Test: `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`
  - 保留并扩展现有弹窗集成测试。
- Test: `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`
  - 纯组件交互测试。

---

## Design Rules

- 弹窗宽度：`w-[320px] max-w-[320px]`。
- 圆角：`rounded-[12px]`。
- 主色：绿色 `#22C55E`，用于加入生词本按钮、已加入状态、发音图标。
- 文本颜色：主文本 `#111827`，次文本 `#6B7280`，边框 `#E5E7EB`，背景 `#FFFFFF`。
- 内容高度：`max-h-[420px]`，正文超出后内部滚动，不撑大窗口。
- 卡片结构：Header 显示单词、音标、发音、收藏、更多；Content 显示翻译、词性释义、例句、标签或状态；Footer 显示复制、发音、加入生词本。
- 所有图标按钮使用 `lucide-react`，并保留 `aria-label`。
- 不新增后端命令。本次“打开主窗口”使用已有 `open_float_window` 命令；“隐藏此词”关闭当前弹窗并在本次前端会话内记录隐藏文本；“反馈翻译问题”复制反馈信息到剪贴板并提示用户已复制。

---

### Task 1: Add Selection Popup State Helper

**Files:**
- Create: `src/pages/SelectionPopup/selectionPopupState.ts`
- Test: `src/pages/SelectionPopup/__test__/selectionPopupState.test.ts`

- [ ] **Step 1: Write the failing state tests**

Create `src/pages/SelectionPopup/__test__/selectionPopupState.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { getSelectionPopupState } from "../selectionPopupState";
import type { ResolvedWord } from "../../../lib/api";

const resolvedWord: ResolvedWord = {
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

describe("getSelectionPopupState", () => {
  it("returns empty when selected text is blank", () => {
    expect(getSelectionPopupState({ text: " ", resolvedWord: null, isResolving: false, resolveError: null })).toBe("empty");
  });

  it("returns tooLong when selected text exceeds the popup limit", () => {
    expect(getSelectionPopupState({ text: "a".repeat(51), resolvedWord: null, isResolving: false, resolveError: null })).toBe("tooLong");
  });

  it("returns loading while resolving without a result", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: null, isResolving: true, resolveError: null })).toBe("loading");
  });

  it("returns saved when the resolved word already has wordId", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: { ...resolvedWord, wordId: "word_1" }, isResolving: false, resolveError: null })).toBe("saved");
  });

  it("returns success when the resolved word is not saved", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord, isResolving: false, resolveError: null })).toBe("success");
  });

  it("returns error when resolveError exists", () => {
    expect(getSelectionPopupState({ text: "serendipity", resolvedWord: null, isResolving: false, resolveError: "翻译失败" })).toBe("error");
  });
});
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/selectionPopupState.test.ts
```

Expected: FAIL because `../selectionPopupState` does not exist.

- [ ] **Step 3: Implement the state helper**

Create `src/pages/SelectionPopup/selectionPopupState.ts`:

```ts
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
```

- [ ] **Step 4: Run test and verify it passes**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/selectionPopupState.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/pages/SelectionPopup/selectionPopupState.ts src/pages/SelectionPopup/__test__/selectionPopupState.test.ts
git commit -m "test: add selection popup state helper"
```

---

### Task 2: Build Presentational Pieces

**Files:**
- Create: `src/pages/SelectionPopup/components/MeaningList.tsx`
- Create: `src/pages/SelectionPopup/components/ExamplePreview.tsx`
- Create: `src/pages/SelectionPopup/components/LoadingState.tsx`
- Create: `src/pages/SelectionPopup/components/ErrorState.tsx`
- Create: `src/pages/SelectionPopup/components/ReviewStatusCard.tsx`
- Test: `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`

- [ ] **Step 1: Write the failing presentational component tests**

Create `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`:

```tsx
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

    expect(screen.getByText("noun")).toBeInTheDocument();
    expect(screen.getByText("意外发现；好运")).toBeInTheDocument();
  });

  it("renders the first example sentence and translation", () => {
    render(<ExamplePreview examples={[{ sentence: "I found this job by pure serendipity.", translation: "我纯粹是意外得到这份工作的。" }]} />);

    expect(screen.getByText("例句")).toBeInTheDocument();
    expect(screen.getByText("I found this job by pure serendipity.")).toBeInTheDocument();
    expect(screen.getByText("我纯粹是意外得到这份工作的。")).toBeInTheDocument();
  });

  it("renders loading skeleton labels", () => {
    render(<LoadingState word="serendipity" />);

    expect(screen.getByText("serendipity")).toBeInTheDocument();
    expect(screen.getByLabelText("正在加载翻译结果")).toBeInTheDocument();
  });

  it("renders retry button in error state", () => {
    render(<ErrorState title="翻译失败" description="请稍后重试" actionLabel="重试" onAction={() => undefined} />);

    expect(screen.getByText("翻译失败")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "重试" })).toBeInTheDocument();
  });

  it("renders saved review state", () => {
    render(<ReviewStatusCard nextReviewText="明天 18:00" />);

    expect(screen.getByText("已在生词本中")).toBeInTheDocument();
    expect(screen.getByText("下次复习：明天 18:00")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: FAIL because the component files do not exist.

- [ ] **Step 3: Implement `MeaningList`**

Create `src/pages/SelectionPopup/components/MeaningList.tsx`:

```tsx
import type { WordMeaning } from "../../../lib/api";

type MeaningListProps = {
  meanings: WordMeaning[];
};

export function MeaningList({ meanings }: MeaningListProps) {
  if (meanings.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col gap-1.5">
      {meanings.map((meaning, index) => (
        <div key={`${meaning.partOfSpeech}-${index}`} className="flex items-start gap-2 text-xs leading-5">
          <span className="mt-0.5 rounded bg-[#DCFCE7] px-1.5 py-0.5 text-[10px] font-semibold uppercase text-[#16A34A]">
            {meaning.partOfSpeech}
          </span>
          <span className="flex-1 text-[#111827]">
            {meaning.translations.join("；")}
          </span>
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Implement `ExamplePreview`**

Create `src/pages/SelectionPopup/components/ExamplePreview.tsx`:

```tsx
import type { TranslationExample } from "../../../lib/api";

type ExamplePreviewProps = {
  examples: TranslationExample[];
};

export function ExamplePreview({ examples }: ExamplePreviewProps) {
  const firstExample = examples[0];

  if (!firstExample) {
    return null;
  }

  return (
    <div className="rounded-lg bg-[#F9FAFB] px-3 py-2 text-xs leading-5">
      <p className="mb-1 font-medium text-[#6B7280]">例句</p>
      <p className="text-[#4B5563]">{firstExample.sentence}</p>
      <p className="mt-1 text-[#9CA3AF]">{firstExample.translation}</p>
    </div>
  );
}
```

- [ ] **Step 5: Implement `LoadingState`**

Create `src/pages/SelectionPopup/components/LoadingState.tsx`:

```tsx
import { Skeleton } from "@heroui/react";

type LoadingStateProps = {
  word: string;
};

export function LoadingState({ word }: LoadingStateProps) {
  return (
    <div aria-label="正在加载翻译结果" className="flex w-full flex-col gap-3">
      <div>
        <p className="text-base font-semibold text-[#111827]">{word}</p>
        <Skeleton className="mt-2 h-3 w-24 rounded-full" />
      </div>
      <div className="flex flex-col gap-2">
        <Skeleton className="h-3 w-full rounded-full" />
        <Skeleton className="h-3 w-11/12 rounded-full" />
        <Skeleton className="h-3 w-8/12 rounded-full" />
      </div>
      <Skeleton className="h-12 w-full rounded-lg" />
    </div>
  );
}
```

- [ ] **Step 6: Implement `ErrorState`**

Create `src/pages/SelectionPopup/components/ErrorState.tsx`:

```tsx
import { Button } from "@heroui/react";
import { AlertCircle } from "lucide-react";

type ErrorStateProps = {
  title: string;
  description: string;
  actionLabel: string;
  onAction: () => void;
};

export function ErrorState({
  title,
  description,
  actionLabel,
  onAction,
}: ErrorStateProps) {
  return (
    <div className="flex min-h-24 flex-col items-center justify-center gap-2 text-center">
      <AlertCircle className="size-5 text-[#EF4444]" aria-hidden="true" />
      <div>
        <p className="text-sm font-semibold text-[#111827]">{title}</p>
        <p className="mt-1 text-xs text-[#6B7280]">{description}</p>
      </div>
      <Button variant="ghost" size="sm" onPress={onAction}>
        {actionLabel}
      </Button>
    </div>
  );
}
```

- [ ] **Step 7: Implement `ReviewStatusCard`**

Create `src/pages/SelectionPopup/components/ReviewStatusCard.tsx`:

```tsx
import { CheckCircle2 } from "lucide-react";

type ReviewStatusCardProps = {
  nextReviewText: string;
};

export function ReviewStatusCard({ nextReviewText }: ReviewStatusCardProps) {
  return (
    <div className="rounded-lg border border-[#BBF7D0] bg-[#F0FDF4] px-3 py-2 text-xs">
      <div className="flex items-center gap-1.5 font-semibold text-[#16A34A]">
        <CheckCircle2 className="size-3.5" aria-hidden="true" />
        <span>已在生词本中</span>
      </div>
      <p className="mt-2 text-[#4B5563]">记忆强度：★★★☆☆</p>
      <p className="mt-1 text-[#4B5563]">下次复习：{nextReviewText}</p>
    </div>
  );
}
```

- [ ] **Step 8: Run presentational tests and commit**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: PASS.

Commit:

```bash
git add src/pages/SelectionPopup/components src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
git commit -m "test: add selection popup presentational states"
```

---

### Task 3: Build Popup Header, Footer, Tags, and Popovers

**Files:**
- Create: `src/pages/SelectionPopup/components/PopupHeader.tsx`
- Create: `src/pages/SelectionPopup/components/PopupFooter.tsx`
- Create: `src/pages/SelectionPopup/components/TagChipList.tsx`
- Create: `src/pages/SelectionPopup/components/TagSelectorPopover.tsx`
- Create: `src/pages/SelectionPopup/components/MoreActionsPopover.tsx`
- Modify: `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`

- [ ] **Step 1: Add failing interaction tests for footer and menu controls**

Append these tests to `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`:

```tsx
import userEvent from "@testing-library/user-event";
import { PopupFooter } from "../components/PopupFooter";
import { MoreActionsPopover } from "../components/MoreActionsPopover";

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

  expect(screen.getByRole("button", { name: "已加入" })).toBeDisabled();
});

it("runs more menu actions", async () => {
  const user = userEvent.setup();
  const calls: string[] = [];

  render(
    <MoreActionsPopover
      onRetry={() => calls.push("retry")}
      onOpenMainWindow={() => calls.push("open")}
      onHideWord={() => calls.push("hide")}
      onCopyFeedback={() => calls.push("feedback")}
    />,
  );

  await user.click(screen.getByRole("button", { name: "更多操作" }));
  await user.click(screen.getByRole("menuitem", { name: "重新翻译" }));
  await user.click(screen.getByRole("button", { name: "更多操作" }));
  await user.click(screen.getByRole("menuitem", { name: "打开主窗口" }));

  expect(calls).toEqual(["retry", "open"]);
});
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: FAIL because footer and popover components do not exist.

- [ ] **Step 3: Implement `MoreActionsPopover`**

Create `src/pages/SelectionPopup/components/MoreActionsPopover.tsx`:

```tsx
import { Button, Popover } from "@heroui/react";
import { BookOpen, EyeOff, MessageSquareWarning, MoreHorizontal, RefreshCw } from "lucide-react";

type MoreActionsPopoverProps = {
  onRetry: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
};

export function MoreActionsPopover({
  onRetry,
  onOpenMainWindow,
  onHideWord,
  onCopyFeedback,
}: MoreActionsPopoverProps) {
  return (
    <Popover placement="bottom end">
      <Popover.Trigger>
        <Button variant="ghost" size="sm" isIconOnly aria-label="更多操作">
          <MoreHorizontal className="size-4" aria-hidden="true" />
        </Button>
      </Popover.Trigger>
      <Popover.Content>
        <Popover.Dialog className="w-40 rounded-lg bg-white p-1 shadow-xl">
          <div role="menu" className="flex flex-col">
            <button role="menuitem" type="button" onClick={onRetry} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <RefreshCw className="size-3.5" aria-hidden="true" />
              重新翻译
            </button>
            <button role="menuitem" type="button" onClick={onOpenMainWindow} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <BookOpen className="size-3.5" aria-hidden="true" />
              打开主窗口
            </button>
            <button role="menuitem" type="button" onClick={onHideWord} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <EyeOff className="size-3.5" aria-hidden="true" />
              隐藏此词
            </button>
            <button role="menuitem" type="button" onClick={onCopyFeedback} className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-[#374151] hover:bg-[#F3F4F6]">
              <MessageSquareWarning className="size-3.5" aria-hidden="true" />
              反馈翻译问题
            </button>
          </div>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
```

- [ ] **Step 4: Implement `PopupHeader`**

Create `src/pages/SelectionPopup/components/PopupHeader.tsx`:

```tsx
import { Button } from "@heroui/react";
import { Star, Volume2 } from "lucide-react";
import { MoreActionsPopover } from "./MoreActionsPopover";

type PopupHeaderProps = {
  word: string;
  phonetic: string | null;
  onSpeak: () => void;
  onRetry: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
};

export function PopupHeader({
  word,
  phonetic,
  onSpeak,
  onRetry,
  onOpenMainWindow,
  onHideWord,
  onCopyFeedback,
}: PopupHeaderProps) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <h2 className="truncate text-base font-semibold leading-6 text-[#111827]">{word}</h2>
          <Button variant="ghost" size="sm" isIconOnly aria-label="播放发音" onPress={onSpeak}>
            <Volume2 className="size-4 text-[#22C55E]" aria-hidden="true" />
          </Button>
        </div>
        {phonetic && (
          <p className="mt-0.5 text-xs text-[#6B7280]">{phonetic}</p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-0.5">
        <Button variant="ghost" size="sm" isIconOnly aria-label="收藏">
          <Star className="size-4 text-[#6B7280]" aria-hidden="true" />
        </Button>
        <MoreActionsPopover
          onRetry={onRetry}
          onOpenMainWindow={onOpenMainWindow}
          onHideWord={onHideWord}
          onCopyFeedback={onCopyFeedback}
        />
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Implement `PopupFooter`**

Create `src/pages/SelectionPopup/components/PopupFooter.tsx`:

```tsx
import { Button } from "@heroui/react";
import { Check, Clipboard, Plus, Volume2 } from "lucide-react";

type PopupFooterProps = {
  isSaved: boolean;
  isSavingWord: boolean;
  canAddWord: boolean;
  onCopy: () => void;
  onSpeak: () => void;
  onAddWord: () => void;
};

export function PopupFooter({
  isSaved,
  isSavingWord,
  canAddWord,
  onCopy,
  onSpeak,
  onAddWord,
}: PopupFooterProps) {
  return (
    <div className="flex items-center justify-between gap-2 border-t border-[#E5E7EB] pt-2">
      <div className="flex items-center gap-1">
        <Button variant="ghost" size="sm" onPress={onCopy}>
          <Clipboard className="size-3.5" aria-hidden="true" />
          复制
        </Button>
        <Button variant="ghost" size="sm" onPress={onSpeak}>
          <Volume2 className="size-3.5 text-[#22C55E]" aria-hidden="true" />
          发音
        </Button>
      </div>
      <Button
        size="sm"
        className={isSaved ? "bg-[#DCFCE7] text-[#16A34A]" : "bg-[#22C55E] text-white"}
        isPending={isSavingWord}
        isDisabled={isSaved || !canAddWord}
        onPress={onAddWord}
      >
        {isSaved ? <Check className="size-3.5" aria-hidden="true" /> : <Plus className="size-3.5" aria-hidden="true" />}
        {isSaved ? "已加入" : "加入生词本"}
      </Button>
    </div>
  );
}
```

- [ ] **Step 6: Implement `TagChipList` and `TagSelectorPopover`**

Create `src/pages/SelectionPopup/components/TagChipList.tsx`:

```tsx
import { Button } from "@heroui/react";
import { Plus } from "lucide-react";
import type { TagItem } from "../../../types/tag";

type TagChipListProps = {
  selectedTags: TagItem[];
  onAddPress: () => void;
};

export function TagChipList({ selectedTags, onAddPress }: TagChipListProps) {
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {selectedTags.map((tag) => (
        <span key={tag.id} className="rounded-full bg-[#EFF6FF] px-2 py-0.5 text-[11px] font-medium text-[#2563EB]">
          {tag.name}
        </span>
      ))}
      <Button variant="ghost" size="sm" isIconOnly aria-label="选择标签" onPress={onAddPress}>
        <Plus className="size-3.5" aria-hidden="true" />
      </Button>
    </div>
  );
}
```

Create `src/pages/SelectionPopup/components/TagSelectorPopover.tsx`:

```tsx
import { Button, Popover } from "@heroui/react";
import { X } from "lucide-react";
import type { TagItem } from "../../../types/tag";

type TagSelectorPopoverProps = {
  isOpen: boolean;
  tags: TagItem[];
  selectedTagIds: string[];
  onOpenChange: (isOpen: boolean) => void;
  onToggleTag: (tagId: string) => void;
  onCreateTag: () => void;
};

export function TagSelectorPopover({
  isOpen,
  tags,
  selectedTagIds,
  onOpenChange,
  onToggleTag,
  onCreateTag,
}: TagSelectorPopoverProps) {
  return (
    <Popover isOpen={isOpen} onOpenChange={onOpenChange} placement="right">
      <Popover.Trigger>
        <span />
      </Popover.Trigger>
      <Popover.Content>
        <Popover.Dialog className="w-64 rounded-xl bg-white p-3 shadow-xl">
          <div className="mb-3 flex items-center justify-between">
            <p className="text-sm font-semibold text-[#111827]">选择标签</p>
            <Button variant="ghost" size="sm" isIconOnly aria-label="关闭标签选择" onPress={() => onOpenChange(false)}>
              <X className="size-4" aria-hidden="true" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2">
            {tags.map((tag) => {
              const selected = selectedTagIds.includes(tag.id);
              return (
                <button
                  key={tag.id}
                  type="button"
                  onClick={() => onToggleTag(tag.id)}
                  className={selected ? "rounded-full bg-[#DCFCE7] px-2.5 py-1 text-xs font-medium text-[#16A34A]" : "rounded-full bg-[#F3F4F6] px-2.5 py-1 text-xs font-medium text-[#6B7280]"}
                >
                  {tag.name}
                </button>
              );
            })}
          </div>
          <Button className="mt-3" variant="ghost" size="sm" onPress={onCreateTag}>
            新建标签
          </Button>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
```

- [ ] **Step 7: Run component tests and commit**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: PASS.

Commit:

```bash
git add src/pages/SelectionPopup/components src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
git commit -m "feat: add selection popup actions"
```

---

### Task 4: Compose the Full Popup Card

**Files:**
- Create: `src/pages/SelectionPopup/components/SelectionPopupCard.tsx`
- Modify: `src/pages/SelectionPopup/SelectionText.tsx`
- Modify: `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`

- [ ] **Step 1: Add failing full card tests**

Append to `src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx`:

```tsx
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

  expect(screen.getByText("serendipity")).toBeInTheDocument();
  expect(screen.getByText("意外发现的好运")).toBeInTheDocument();
  expect(screen.getByText("加入生词本")).toBeInTheDocument();
});

it("renders loading card", () => {
  render(
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

  expect(screen.getByLabelText("正在加载翻译结果")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: FAIL because `SelectionPopupCard` does not exist.

- [ ] **Step 3: Implement `SelectionPopupCard`**

Create `src/pages/SelectionPopup/components/SelectionPopupCard.tsx`:

```tsx
import { Card } from "@heroui/react";
import type { ResolvedWord } from "../../../lib/api";
import type { TagItem } from "../../../types/tag";
import type { SelectionPopupState } from "../selectionPopupState";
import { ErrorState } from "./ErrorState";
import { ExamplePreview } from "./ExamplePreview";
import { LoadingState } from "./LoadingState";
import { MeaningList } from "./MeaningList";
import { PopupFooter } from "./PopupFooter";
import { PopupHeader } from "./PopupHeader";
import { ReviewStatusCard } from "./ReviewStatusCard";
import { TagChipList } from "./TagChipList";

type SelectionPopupCardProps = {
  text: string;
  state: SelectionPopupState;
  resolvedWord: ResolvedWord | null;
  selectedTags: TagItem[];
  isSavingWord: boolean;
  onRetry: () => void;
  onCopy: () => void;
  onSpeak: () => void;
  onAddWord: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
  onOpenTagSelector: () => void;
};

export function SelectionPopupCard({
  text,
  state,
  resolvedWord,
  selectedTags,
  isSavingWord,
  onRetry,
  onCopy,
  onSpeak,
  onAddWord,
  onOpenMainWindow,
  onHideWord,
  onCopyFeedback,
  onOpenTagSelector,
}: SelectionPopupCardProps) {
  const displayText = resolvedWord?.word || text.trim() || "未读取到选中文本";
  const isSaved = state === "saved";
  const canAddWord = state === "success" && Boolean(resolvedWord);

  return (
    <Card className="w-[320px] max-w-[320px] rounded-[12px] border border-[#E5E7EB] bg-white shadow-xl">
      <Card.Content className="flex max-h-[420px] flex-col gap-3 overflow-y-auto p-3">
        {state === "loading" ? (
          <LoadingState word={displayText} />
        ) : state === "empty" ? (
          <ErrorState title="无选区结果" description="请重新选择需要翻译的内容" actionLabel="关闭" onAction={onHideWord} />
        ) : state === "tooLong" ? (
          <ErrorState title="内容过长" description="划词弹窗适合翻译 50 个字符以内的短文本" actionLabel="重新翻译" onAction={onRetry} />
        ) : state === "error" ? (
          <ErrorState title="翻译失败" description="请检查网络或稍后重试" actionLabel="重试" onAction={onRetry} />
        ) : (
          <>
            <PopupHeader
              word={displayText}
              phonetic={resolvedWord?.phonetic ?? null}
              onSpeak={onSpeak}
              onRetry={onRetry}
              onOpenMainWindow={onOpenMainWindow}
              onHideWord={onHideWord}
              onCopyFeedback={onCopyFeedback}
            />
            {resolvedWord && (
              <>
                <p className="text-sm font-medium leading-6 text-[#111827]">
                  {resolvedWord.translation}
                </p>
                <MeaningList meanings={resolvedWord.meanings} />
                <ExamplePreview examples={resolvedWord.examples} />
                {isSaved ? (
                  <ReviewStatusCard nextReviewText="明天 18:00" />
                ) : (
                  <TagChipList selectedTags={selectedTags} onAddPress={onOpenTagSelector} />
                )}
              </>
            )}
            <PopupFooter
              isSaved={isSaved}
              isSavingWord={isSavingWord}
              canAddWord={canAddWord}
              onCopy={onCopy}
              onSpeak={onSpeak}
              onAddWord={onAddWord}
            />
          </>
        )}
      </Card.Content>
    </Card>
  );
}
```

- [ ] **Step 4: Replace `SelectionText` with the new card entry**

Modify `src/pages/SelectionPopup/SelectionText.tsx`:

```tsx
import type { ResolvedWord } from "../../lib/api";
import type { TagItem } from "../../types/tag";
import type { SelectionPopupState } from "./selectionPopupState";
import { SelectionPopupCard } from "./components/SelectionPopupCard";

type SelectionTextProps = {
  text: string;
  state: SelectionPopupState;
  resolvedWord: ResolvedWord | null;
  selectedTags: TagItem[];
  isSavingWord: boolean;
  onRetry: () => void;
  onCopy: () => void;
  onSpeak: () => void;
  onAddWord: () => void;
  onOpenMainWindow: () => void;
  onHideWord: () => void;
  onCopyFeedback: () => void;
  onOpenTagSelector: () => void;
};

export function SelectionText(props: SelectionTextProps) {
  return <SelectionPopupCard {...props} />;
}
```

- [ ] **Step 5: Run component tests and commit**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
```

Expected: PASS.

Commit:

```bash
git add src/pages/SelectionPopup/SelectionText.tsx src/pages/SelectionPopup/components/SelectionPopupCard.tsx src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx
git commit -m "feat: compose selection popup card"
```

---

### Task 5: Wire Container Interactions

**Files:**
- Modify: `src/pages/SelectionPopup/index.tsx`
- Modify: `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`

- [ ] **Step 1: Extend integration test mocks**

At the top of `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`, mock clipboard and TTS-related APIs:

```tsx
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(),
}));

vi.mock("../../lib/api", async () => {
  const actual = await vi.importActual<typeof import("../../lib/api")>("../../lib/api");
  return {
    ...actual,
    resolveWord: vi.fn(),
    addWord: vi.fn(),
    speakText: vi.fn(),
    getTags: vi.fn(),
    createTag: vi.fn(),
  };
});
```

Add integration tests:

```tsx
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { addWord, createTag, getTags, resolveWord, speakText } from "../../lib/api";

it("copies translated text from the popup", async () => {
  vi.mocked(resolveWord).mockResolvedValue(resolvedWord);
  vi.mocked(getTags).mockResolvedValue([]);
  const user = userEvent.setup();

  renderSelectionPopup("/selection-popup?text=serendipity");
  await screen.findByText("意外发现的好运");
  await user.click(screen.getByRole("button", { name: "复制" }));

  expect(writeText).toHaveBeenCalledWith("serendipity\n意外发现的好运");
});

it("speaks the resolved word", async () => {
  vi.mocked(resolveWord).mockResolvedValue(resolvedWord);
  vi.mocked(getTags).mockResolvedValue([]);
  const user = userEvent.setup();

  renderSelectionPopup("/selection-popup?text=serendipity");
  await screen.findByText("意外发现的好运");
  await user.click(screen.getByRole("button", { name: "发音" }));

  expect(speakText).toHaveBeenCalledWith("serendipity", "en");
});

it("adds resolved word with selected tags", async () => {
  vi.mocked(resolveWord).mockResolvedValue(resolvedWord);
  vi.mocked(getTags).mockResolvedValue([{ id: "tag_1", name: "TOEFL", color: "#2563EB", sort_order: 0, created_at: 1, updated_at: 1 }]);
  vi.mocked(addWord).mockResolvedValue({ ...resolvedWord, wordId: "word_1", createdAt: 1, updatedAt: 1 });
  const user = userEvent.setup();

  renderSelectionPopup("/selection-popup?text=serendipity");
  await screen.findByText("意外发现的好运");
  await user.click(screen.getByRole("button", { name: "选择标签" }));
  await user.click(screen.getByRole("button", { name: "TOEFL" }));
  await user.click(screen.getByRole("button", { name: "加入生词本" }));

  expect(addWord).toHaveBeenCalledWith(expect.objectContaining({ tags: "TOEFL" }));
  expect(await screen.findByRole("button", { name: "已加入" })).toBeDisabled();
});
```

- [ ] **Step 2: Run integration tests and verify they fail**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: FAIL because `SelectionPopupPage` does not yet pass new props or wire actions.

- [ ] **Step 3: Implement container state and handlers**

Modify imports in `src/pages/SelectionPopup/index.tsx`:

```tsx
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
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
```

Add state:

```tsx
const [tags, setTags] = useState<TagItem[]>([]);
const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);
const [isTagSelectorOpen, setIsTagSelectorOpen] = useState(false);
```

Add derived values:

```tsx
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
```

Load tags:

```tsx
useEffect(() => {
  getTags()
    .then(setTags)
    .catch((error) => {
      console.warn("Failed to load selection popup tags", error);
    });
}, []);
```

Reset selected tags on new text:

```tsx
useEffect(() => {
  setSelectedTagIds([]);
  setIsTagSelectorOpen(false);
}, [text]);
```

Add handlers:

```tsx
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
  invoke("open_float_window").catch((error) => {
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
```

Update `handleAddWord` so it saves selected tag names:

```tsx
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
```

Pass new props:

```tsx
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
```

Render `TagSelectorPopover` next to `SelectionText`:

```tsx
<TagSelectorPopover
  isOpen={isTagSelectorOpen}
  tags={tags}
  selectedTagIds={selectedTagIds}
  onOpenChange={setIsTagSelectorOpen}
  onToggleTag={handleToggleTag}
  onCreateTag={handleCreateTag}
/>
```

- [ ] **Step 4: Run integration tests and commit**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: PASS.

Commit:

```bash
git add src/pages/SelectionPopup/index.tsx src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
git commit -m "feat: wire selection popup interactions"
```

---

### Task 6: Preserve Auto-Close and Text Update Behavior

**Files:**
- Modify: `src/pages/SelectionPopup/index.tsx`
- Modify: `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`

- [ ] **Step 1: Add tests for auto-close while interacting**

Append to `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`:

```tsx
it("keeps popup open while tag selector is open", async () => {
  vi.useFakeTimers();
  vi.mocked(resolveWord).mockResolvedValue(resolvedWord);
  vi.mocked(getTags).mockResolvedValue([{ id: "tag_1", name: "TOEFL", color: "#2563EB", sort_order: 0, created_at: 1, updated_at: 1 }]);
  const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

  renderSelectionPopup("/selection-popup?text=serendipity");
  await screen.findByText("意外发现的好运");
  await user.click(screen.getByRole("button", { name: "选择标签" }));

  vi.advanceTimersByTime(2000);

  expect(invokeMock).not.toHaveBeenCalledWith("close_selection_popup");
  vi.useRealTimers();
});
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: FAIL because auto-close does not yet account for open popovers.

- [ ] **Step 3: Pause auto-close while tag selector is open**

Modify `scheduleAutoClose` inside `src/pages/SelectionPopup/index.tsx`:

```tsx
const isTagSelectorOpenRef = useRef(false);

useEffect(() => {
  isTagSelectorOpenRef.current = isTagSelectorOpen;
}, [isTagSelectorOpen]);

const scheduleAutoClose = useCallback((delayMs = AUTO_CLOSE_DELAY_MS) => {
  clearCloseTimer();
  closeTimerRef.current = setTimeout(async () => {
    if (isTagSelectorOpenRef.current) {
      scheduleAutoClose(AUTO_CLOSE_DELAY_MS);
      return;
    }

    const isInside = await isCursorInsidePopup();
    if (isInside) {
      scheduleAutoClose(AUTO_CLOSE_DELAY_MS);
      return;
    }
    closePopup();
  }, delayMs);
}, [clearCloseTimer, closePopup, isCursorInsidePopup]);
```

- [ ] **Step 4: Run tests and commit**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: PASS.

Commit:

```bash
git add src/pages/SelectionPopup/index.tsx src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
git commit -m "fix: preserve selection popup interactions"
```

---

### Task 7: Full Verification and Visual QA

**Files:**
- Verify only, then commit any final polish if tests reveal gaps.

- [ ] **Step 1: Run all SelectionPopup tests**

Run:

```bash
pnpm test -- src/pages/SelectionPopup/__test__/selectionPopupState.test.ts src/pages/SelectionPopup/__test__/SelectionPopupCard.test.tsx src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: all listed tests PASS.

- [ ] **Step 2: Run frontend build**

Run:

```bash
pnpm build
```

Expected: TypeScript compilation and Vite build complete successfully.

- [ ] **Step 3: Run local dev server for visual QA**

Run:

```bash
pnpm dev --host 127.0.0.1 --port 1420
```

Expected: Vite reports a local URL on `http://127.0.0.1:1420/`.

- [ ] **Step 4: Open the selection popup route**

Use the in-app browser to open:

```text
http://127.0.0.1:1420/selection-popup?text=serendipity
```

Expected visual checks:
- 卡片宽度约 320px，圆角和阴影与设计图接近。
- 默认状态展示单词、音标、发音、收藏、更多、翻译、词性释义、例句、标签、复制、发音、加入生词本。
- 加载状态显示骨架屏。
- 失败状态显示错误标题和重试按钮。
- 点击“选择标签”显示标签浮层。
- 点击“更多操作”显示菜单项。
- 点击“加入生词本”后按钮变为“已加入”。

- [ ] **Step 5: Commit final polish**

If Step 1-4 required minor fixes, commit them:

```bash
git add src/pages/SelectionPopup
git commit -m "feat: polish selection popup UI"
```

If there were no final fixes, do not create an empty commit.

---

## Self-Review

- Spec coverage:
  - 设计图默认卡片：Task 2-4 覆盖 header、翻译、词性、例句、标签、footer。
  - 状态展示：Task 1、Task 2、Task 4 覆盖 loading、success、saved、error、empty、tooLong。
  - 交互操作：Task 3、Task 5 覆盖复制、发音、加入生词本、标签选择、更多菜单、重试、打开主窗口、隐藏、反馈。
  - 自动关闭：Task 6 保留现有自动关闭并避免打开标签浮层时误关闭。
  - 测试和验证：Task 1-7 均有命令和期望输出。
- Placeholder scan:
  - 本计划没有使用占位式待补内容。
  - 每个新增文件都有明确职责和实现代码。
- Type consistency:
  - `SelectionPopupState` 在 Task 1 定义，Task 4 使用。
  - `ResolvedWord`、`WordMeaning`、`TagItem` 均使用当前 `src/lib/api` 和 `src/types/tag` 导出的类型。
  - `PopupFooter`、`SelectionPopupCard`、`SelectionText` 的 props 在任务之间保持一致。
