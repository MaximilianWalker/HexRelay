import type { HTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./chat.module.css";

export function PresenceDot({
  className,
  status,
  ...props
}: HTMLAttributes<HTMLSpanElement> & {
  status: "online" | "away";
}) {
  return (
    <span
      className={cx(styles.presenceDot, status === "online" ? styles.presenceOnline : styles.presenceAway, className)}
      role="img"
      {...props}
    />
  );
}
