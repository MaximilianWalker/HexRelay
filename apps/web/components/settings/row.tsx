import type { ReactNode } from "react";

import { StatusBadge, type Status } from "./status";
import styles from "./styles.module.css";

export function Row({
  children,
  description,
  label,
  status,
}: {
  children: ReactNode;
  description: string;
  label: string;
  status: Status;
}) {
  return (
    <div className={styles.settingRow}>
      <div className={styles.settingCopy}>
        <div className={styles.settingHeading}>
          <p className={styles.settingLabel}>{label}</p>
          <StatusBadge status={status} />
        </div>
        <p className={styles.settingDescription}>{description}</p>
      </div>
      <div className={styles.settingControl}>{children}</div>
    </div>
  );
}
