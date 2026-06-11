import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./empty-state.module.css";

export function EmptyState({
  action,
  children,
  className,
  title,
  ...props
}: HTMLAttributes<HTMLElement> & {
  action?: ReactNode;
  title?: string;
}) {
  return (
    <section className={cx(styles.emptyState, className)} {...props}>
      {title ? <p className={styles.emptyStateTitle}>{title}</p> : null}
      {children}
      {action}
    </section>
  );
}
