import { Badge } from "@/components/ui/badge";

import styles from "./settings-ui.module.css";

export type SettingStatus = "Live" | "Review" | "Locked" | "Dev only";

export function SettingStatusBadge({ status }: { status: SettingStatus }) {
  const tone = status === "Live" ? "success" : status === "Review" ? "warning" : status === "Dev only" ? "accent" : "muted";

  return (
    <Badge className={styles.status} tone={tone}>
      {status}
    </Badge>
  );
}
