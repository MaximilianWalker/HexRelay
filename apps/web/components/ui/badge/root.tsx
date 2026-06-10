import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type BadgeTone = "neutral" | "accent" | "muted" | "success" | "warning" | "danger";
type BadgeSize = "sm" | "md" | "lg";
type BadgeShape = "default" | "counter";

const toneClass: Record<BadgeTone, string> = {
  neutral: "",
  accent: styles.badgeAccent,
  muted: styles.badgeMuted,
  success: styles.badgeSuccess,
  warning: styles.badgeWarning,
  danger: styles.badgeDanger,
};

export function Root({
  children,
  className,
  icon,
  shape = "default",
  size = "md",
  tone = "neutral",
  ...props
}: HTMLAttributes<HTMLSpanElement> & {
  icon?: ReactNode;
  shape?: BadgeShape;
  size?: BadgeSize;
  tone?: BadgeTone;
}) {
  return (
    <span
      className={cx(
        styles.badge,
        size === "sm" && styles.badgeSm,
        size === "lg" && styles.badgeLg,
        shape === "counter" && styles.badgeCounter,
        toneClass[tone],
        className,
      )}
      {...props}
    >
      {icon}
      {children}
    </span>
  );
}
