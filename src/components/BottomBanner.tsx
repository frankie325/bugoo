import { Button } from '@heroui/react';

interface BottomBannerProps {
  dueCount: number;
  totalCount: number;
}

export function BottomBanner({ dueCount, totalCount }: BottomBannerProps) {
  const progress = totalCount > 0 ? Math.round((dueCount / totalCount) * 100) : 0;

  return (
    <div className="h-14 border-t border-divider px-4 flex items-center justify-between bg-primary-50">
      <div className="flex items-center gap-2">
        <span>🔥</span>
        <span className="text-sm">
          今日学习进度 {dueCount} / {totalCount}
        </span>
        <div className="w-24 h-2 bg-foreground-200 rounded-full overflow-hidden">
          <div
            className="h-full bg-primary"
            style={{ width: `${progress}%` }}
          />
        </div>
        <span className="text-sm text-foreground-500">{progress}%</span>
      </div>
      <Button size="sm">
        开始复习 +5 XP
      </Button>
    </div>
  );
}
