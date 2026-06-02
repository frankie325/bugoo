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
