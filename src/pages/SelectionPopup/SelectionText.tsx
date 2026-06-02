import { Button, Card } from "@heroui/react";
import type { ResolvedWord } from "../../lib/api";

type SelectionTextProps = {
  text: string;
  resolvedWord: ResolvedWord | null;
  isResolving: boolean;
  isSavingWord: boolean;
  resolveError: string | null;
  onAddWord: () => void;
};

export function SelectionText({
  text,
  resolvedWord,
  isResolving,
  isSavingWord,
  resolveError,
  onAddWord,
}: SelectionTextProps) {
  const displayText = text.trim();
  const showResult = Boolean(displayText) && (isResolving || resolvedWord || resolveError);

  return (
    <Card className="flex w-80 max-w-80 flex-col gap-3 border border-divider bg-background p-4 shadow-lg">
      <div className="flex items-center gap-2">
        <span className="text-base font-semibold text-foreground">
          {resolvedWord?.word || displayText || "未读取到选中文本"}
        </span>
        {resolvedWord?.phonetic && (
          <span className="text-xs text-foreground-400">{resolvedWord.phonetic}</span>
        )}
      </div>

      {showResult && (
        <div className="flex flex-col gap-2 text-sm">
          {isResolving && !resolvedWord && (
            <p className="text-foreground-400">查询中…</p>
          )}

          {resolveError && (
            <p className="text-danger-500">{resolveError}</p>
          )}

          {resolvedWord && (
            <>
              <p className="text-foreground-700">{resolvedWord.translation}</p>

              {resolvedWord.meanings.length > 0 && (
                <ul className="flex list-disc flex-col gap-1 pl-5 text-foreground-600">
                  {resolvedWord.meanings.map((meaning, index) => (
                    <li key={`${meaning.partOfSpeech}-${index}`}>
                      <span className="font-medium">{meaning.partOfSpeech}</span>
                      {": "}
                      {meaning.translations.join("；")}
                    </li>
                  ))}
                </ul>
              )}

              {resolvedWord.examples.length > 0 && (
                <div className="flex flex-col gap-1 rounded bg-foreground-50 p-2">
                  {resolvedWord.examples.slice(0, 2).map((example, index) => (
                    <div key={`${example.sentence}-${index}`} className="text-xs">
                      <p className="text-foreground-700">{example.sentence}</p>
                      <p className="text-foreground-400">{example.translation}</p>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}

          <div className="mt-1 flex items-center justify-end gap-2">
            {resolvedWord?.wordId ? (
              <span className="text-xs text-success">已加入生词本</span>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                isPending={isSavingWord}
                isDisabled={!resolvedWord || isResolving}
                onPress={onAddWord}
              >
                加入生词本
              </Button>
            )}
          </div>
        </div>
      )}

      {!showResult && displayText && (
        <p className="max-h-32 w-full overflow-hidden break-words text-sm leading-6 text-foreground">
          {displayText}
        </p>
      )}
    </Card>
  );
}
