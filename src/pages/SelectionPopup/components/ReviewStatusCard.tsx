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
