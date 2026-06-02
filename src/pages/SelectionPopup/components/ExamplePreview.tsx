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
