import { Skeleton } from "@heroui/react";

export function LoadingState() {
  return (
    <div aria-label="正在加载翻译结果" className="flex w-full flex-col gap-3">
      <Skeleton className="h-3 w-1/5 rounded-lg" />
      <Skeleton className="h-3 w-1/2 rounded-lg" />
      <div className="flex flex-col gap-2">
        <Skeleton className="h-3 w-full rounded-full" />
        <Skeleton className="h-3 w-11/12 rounded-full" />
        <Skeleton className="h-3 w-8/12 rounded-full" />
      </div>
      <Skeleton className="h-12 w-full rounded-lg" />
      <Skeleton className="h-12 w-full rounded-lg" />
      <Skeleton className="h-12 w-full rounded-lg" />
    </div>
  );
}
