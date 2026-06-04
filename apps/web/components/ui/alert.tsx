import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type AlertTone = "info" | "success" | "warning" | "danger";

const toneClass: Record<AlertTone, string> = {
  info: "",
  success: styles.alertSuccess,
  warning: styles.alertWarning,
  danger: styles.alertDanger,
};

export function Alert({
  children,
  className,
  icon,
  tone = "info",
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  icon?: ReactNode;
  tone?: AlertTone;
}) {
  return (
    <div className={cx(styles.alert, toneClass[tone], className)} {...props}>
      {icon}
      {children}
    </div>
  );
}
