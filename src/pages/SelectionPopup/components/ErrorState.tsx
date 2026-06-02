import { Button } from "@heroui/react";
import { AlertCircle } from "lucide-react";

type ErrorStateProps = {
  title: string;
  description: string;
  actionLabel: string;
  onAction: () => void;
};

export function ErrorState({
  title,
  description,
  actionLabel,
  onAction,
}: ErrorStateProps) {
  return (
    <div className="flex min-h-24 flex-col items-center justify-center gap-2 text-center">
      <AlertCircle className="size-5 text-[#EF4444]" aria-hidden="true" />
      <div>
        <p className="text-sm font-semibold text-[#111827]">{title}</p>
        <p className="mt-1 text-xs text-[#6B7280]">{description}</p>
      </div>
      <Button variant="ghost" size="sm" onPress={onAction}>
        {actionLabel}
      </Button>
    </div>
  );
}
