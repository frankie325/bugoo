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
