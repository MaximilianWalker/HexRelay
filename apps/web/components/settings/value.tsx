import type { ReactNode } from "react";

import styles from "./styles.module.css";

export function Value({ children }: { children: ReactNode }) {
  return <span className={styles.readOnlyValue}>{children}</span>;
}
