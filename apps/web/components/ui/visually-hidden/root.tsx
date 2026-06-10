import type { HTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export function Root({ className, ...props }: HTMLAttributes<HTMLSpanElement>) {
  return <span className={cx(styles.visuallyHidden, className)} {...props} />;
}
