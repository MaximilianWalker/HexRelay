import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

type BadgeTone = "neutral" | "accent" | "muted" | "success" | "warning" | "danger";

const toneClass: Record<BadgeTone, string> = {
  neutral: "",
  accent: styles.badgeAccent,
  muted: styles.badgeMuted,
  success: styles.badgeSuccess,
  warning: styles.badgeWarning,
  danger: styles.badgeDanger,
};

export function Badge({
  children,
  className,
  icon,
  tone = "neutral",
  ...props
}: HTMLAttributes<HTMLSpanElement> & {
  icon?: ReactNode;
  tone?: BadgeTone;
}) {
  return (
    <span className={cx(styles.badge, toneClass[tone], className)} {...props}>
      {icon}
      {children}
    </span>
  );
}
