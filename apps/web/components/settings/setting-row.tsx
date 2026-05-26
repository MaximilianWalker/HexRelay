import type { ReactNode } from "react";

import { ToggleSwitch } from "@/components/ui/field";

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

export function ToggleControl({
  checked,
  disabled,
  label,
  onChange,
}: {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
}) {
  return <ToggleSwitch checked={checked} disabled={disabled} label={label} onChange={onChange} />;
}

export function ReadOnlyValue({ children }: { children: ReactNode }) {
  return <span className={styles.readOnlyValue}>{children}</span>;
}
