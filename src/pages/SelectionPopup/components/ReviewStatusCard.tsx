import { CheckCircle2 } from "lucide-react";

type ReviewStatusCardProps = {
  nextReviewText: string;
};

export function ReviewStatusCard({ nextReviewText }: ReviewStatusCardProps) {
  return (
    <div className="border-accent bg-accent-soft rounded-lg border px-3 py-2 text-xs">
      <div className="text-accent flex items-center gap-1.5 font-semibold">
        <CheckCircle2 className="size-3.5" aria-hidden="true" />
        <span>已在生词本中</span>
      </div>
      <p className="text-muted mt-2">记忆强度：★★★☆☆</p>
      <p className="text-muted mt-1">下次复习：{nextReviewText}</p>
    </div>
  );
}
