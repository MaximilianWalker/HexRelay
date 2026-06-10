import type { ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export function Root({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx(styles.dialogActions, className)}>{children}</div>;
}
