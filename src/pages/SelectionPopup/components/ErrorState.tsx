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
      <AlertCircle className="text-danger size-5" aria-hidden="true" />
      <div>
        <p className="text-foreground text-sm font-semibold">{title}</p>
        <p className="text-muted mt-1 text-xs">{description}</p>
      </div>
      <Button variant="ghost" size="sm" onPress={onAction}>
        {actionLabel}
      </Button>
    </div>
  );
}
