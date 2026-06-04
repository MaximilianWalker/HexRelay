import { Badge } from "@/components/ui/badge";

export type Status = "Live" | "Review" | "Locked" | "Dev only";

export function StatusBadge({ status }: { status: Status }) {
  const tone = status === "Live" ? "success" : status === "Review" ? "warning" : status === "Dev only" ? "accent" : "muted";

  return (
    <Badge tone={tone}>
      {status}
    </Badge>
  );
}
