import type { ReactNode } from "react";

import styles from "./styles.module.css";

export function Example({
  children,
  title,
  wide,
}: {
  children: ReactNode;
  title: string;
  wide?: boolean;
}) {
  return (
    <div className={wide ? `${styles.example} ${styles.exampleWide}` : styles.example}>
      <h3>{title}</h3>
      {children}
    </div>
  );
}
