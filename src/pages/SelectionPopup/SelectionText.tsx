import { Card } from "@heroui/react";

type SelectionTextProps = {
  text: string;
};

export function SelectionText({ text }: SelectionTextProps) {
  const displayText = text.trim();

  return (
    <Card className="h-full w-full border border-divider bg-background shadow-lg">
      <div className="flex h-full min-h-24 max-h-40 w-80 max-w-80 items-center p-4">
        {displayText ? (
          <p className="max-h-32 w-full overflow-hidden break-words text-sm leading-6 text-foreground">
            {displayText}
          </p>
        ) : (
          <p className="w-full text-center text-sm text-foreground-500">
            未读取到选中文本
          </p>
        )}
      </div>
    </Card>
  );
}
