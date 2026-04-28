import { AlertTriangle } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

type EmptyStateProps = {
  title: string;
  body: string;
  kicker?: string;
};

export function EmptyState({ title, body, kicker = "Foundation placeholder" }: EmptyStateProps) {
  return (
    <section className="empty-state" aria-live="polite">
      <p className="empty-kicker">{kicker}</p>
      <h2>{title}</h2>
      <p>{body}</p>
    </section>
  );
}

export function LoadingState({ label, compact = false }: { label: string; compact?: boolean }) {
  return (
    <div className={cn("loading-state", compact && "compact")} role="status">
      <Skeleton className="size-4 rounded-full" />
      <span>{label}</span>
    </div>
  );
}

export function ErrorState({
  title,
  message,
  compact = false,
}: {
  title: string;
  message: string;
  compact?: boolean;
}) {
  return (
    <Alert className={compact ? "error-state compact" : "error-state"} variant="destructive">
      <AlertTriangle aria-hidden="true" />
      <AlertTitle>{title}</AlertTitle>
      <AlertDescription>{message}</AlertDescription>
    </Alert>
  );
}
