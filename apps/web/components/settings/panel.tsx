import type { ReactNode } from "react";

import styles from "./styles.module.css";

export function Panel({
  category,
  children,
  label,
}: {
  category?: { label: string };
  children: ReactNode;
  label?: string;
}) {
  const ariaLabel = label ?? category?.label;

  return (
    <section aria-label={ariaLabel} className={styles.panel}>
      <div className={styles.settingList}>{children}</div>
    </section>
  );
}
