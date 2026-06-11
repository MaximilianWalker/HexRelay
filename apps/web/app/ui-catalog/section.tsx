import type { ReactNode } from "react";

import styles from "./styles.module.css";

export function Section({
  children,
  description,
  hideHeader = false,
  id,
  title,
  visible = true,
}: {
  children: ReactNode;
  description?: string;
  hideHeader?: boolean;
  id: string;
  title: string;
  visible?: boolean;
}) {
  if (!visible) {
    return null;
  }

  return (
    <section aria-label={hideHeader ? title : undefined} className={styles.section} id={id}>
      {hideHeader ? null : (
        <div className={styles.sectionHeader}>
          <h2>{title}</h2>
          {description ? <p>{description}</p> : null}
        </div>
      )}
      {children}
    </section>
  );
}
