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
          <span className="text-accent font-semibold uppercase">
            [{meaning.partOfSpeech}]
          </span>
          <span className="text-foreground flex-1">
            {meaning.translations.join("；")}
          </span>
        </div>
      ))}
    </div>
  );
}
