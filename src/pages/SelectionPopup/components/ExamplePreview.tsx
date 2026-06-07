import type { TranslationExample } from "../../../lib/api";
import { List } from "lucide-react";

type ExamplePreviewProps = {
  examples: TranslationExample[];
  highlightText?: string;
};

function HighlightedText({
  text,
  highlight,
}: {
  text: string;
  highlight?: string;
}) {
  const trimmed = highlight?.trim();
  if (!trimmed) {
    return <>{text}</>;
  }

  const escaped = trimmed.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const parts = text.split(new RegExp(`(${escaped})`, "gi"));

  return (
    <>
      {parts.map((part, index) =>
        index % 2 === 1 ? (
          <span key={index} className="text-accent">
            {part}
          </span>
        ) : (
          <span key={index}>{part}</span>
        ),
      )}
    </>
  );
}

export function ExamplePreview({ examples, highlightText }: ExamplePreviewProps) {
  if (examples.length === 0) {
    return null;
  }

  return (
    <div className="mt-3 flex flex-col">
      <h2 className="mb-1 flex items-center gap-1 text-sm font-bold">
        <List strokeWidth={3} size={14} />
        例句
      </h2>
      <div className="flex flex-col gap-1.5">
        {examples.map((example, index) => (
          <div
            key={`${example.sentence}-${index}`}
            className="rounded-lg bg-background px-3 py-2 text-xs leading-5"
          >
            <p className="text-foreground/80">
              <HighlightedText
                text={example.sentence}
                highlight={highlightText}
              />
            </p>
            <p className="text-muted mt-1">{example.translation}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
