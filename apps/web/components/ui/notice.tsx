import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

type NoticeTone = "info" | "success" | "warning" | "danger";

const toneClass: Record<NoticeTone, string> = {
  info: "",
  success: styles.noticeSuccess,
  warning: styles.noticeWarning,
  danger: styles.noticeDanger,
};

export function Notice({
  children,
  className,
  icon,
  tone = "info",
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  icon?: ReactNode;
  tone?: NoticeTone;
}) {
  return (
    <div className={cx(styles.notice, toneClass[tone], className)} {...props}>
      {icon}
      {children}
    </div>
  );
}
