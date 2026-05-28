import type { ReactNode } from "react";

import styles from "./settings-ui.module.css";

export function SettingValue({ children }: { children: ReactNode }) {
  return <span className={styles.readOnlyValue}>{children}</span>;
}
