import type { ReactNode } from "react";

import { SettingStatusBadge, type SettingStatus } from "./setting-status";
import styles from "./settings-ui.module.css";

export function SettingRow({
  children,
  description,
  label,
  status,
}: {
  children: ReactNode;
  description: string;
  label: string;
  status: SettingStatus;
}) {
  return (
    <div className={styles.settingRow}>
      <div className={styles.settingCopy}>
        <div className={styles.settingHeading}>
          <p className={styles.settingLabel}>{label}</p>
          <SettingStatusBadge status={status} />
        </div>
        <p className={styles.settingDescription}>{description}</p>
      </div>
      <div className={styles.settingControl}>{children}</div>
    </div>
  );
}
