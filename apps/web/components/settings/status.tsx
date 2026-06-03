import { Badge } from "@/components/ui/badge";

import styles from "./styles.module.css";

export type Status = "Live" | "Review" | "Locked" | "Dev only";

export function StatusBadge({ status }: { status: Status }) {
  const tone = status === "Live" ? "success" : status === "Review" ? "warning" : status === "Dev only" ? "accent" : "muted";

  return (
    <Badge className={styles.status} tone={tone}>
      {status}
    </Badge>
  );
}
