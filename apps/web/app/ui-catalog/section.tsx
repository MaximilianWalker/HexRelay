import type { ReactNode } from "react";

import styles from "./styles.module.css";

export function Section({
  children,
  description,
  id,
  title,
  visible = true,
}: {
  children: ReactNode;
  description?: string;
  id: string;
  title: string;
  visible?: boolean;
}) {
  if (!visible) {
    return null;
  }

  return (
    <section className={styles.section} id={id}>
      <div className={styles.sectionHeader}>
        <h2>{title}</h2>
        {description ? <p>{description}</p> : null}
      </div>
      {children}
    </section>
  );
}
