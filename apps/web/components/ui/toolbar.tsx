import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export function Toolbar({
  actions,
  children,
  className,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  actions?: ReactNode;
}) {
  return (
    <div className={cx(styles.toolbar, className)} {...props}>
      <div className={styles.toolbarPrimary}>{children}</div>
      {actions ? <div className={styles.toolbarActions}>{actions}</div> : null}
    </div>
  );
}
