import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

type ListActionButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-pressed"> & {
  active?: boolean;
  badge?: ReactNode;
  badgeLabel?: string;
  icon: ReactNode;
};

export function ListActionButton({
  active = false,
  badge,
  badgeLabel,
  children,
  className,
  icon,
  ...props
}: ListActionButtonProps) {
  return (
    <button
      aria-pressed={active}
      className={cx(styles.listAction, className)}
      data-active={active ? "true" : undefined}
      type="button"
      {...props}
    >
      <span className={styles.listActionIcon} aria-hidden="true">
        {icon}
      </span>
      <span className={styles.listActionLabel}>{children}</span>
      {badge ? (
        <span aria-label={badgeLabel} className={styles.listActionBadge}>
          {badge}
        </span>
      ) : null}
    </button>
  );
}
