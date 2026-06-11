import type { HTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./visually-hidden.module.css";

export function VisuallyHidden({ className, ...props }: HTMLAttributes<HTMLSpanElement>) {
  return <span className={cx(styles.visuallyHidden, className)} {...props} />;
}
