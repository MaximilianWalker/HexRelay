import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

type ListActionSize = "sm" | "md" | "lg";

const sizeClass: Record<ListActionSize, string | undefined> = {
  lg: styles.listActionLg,
  md: undefined,
  sm: styles.listActionSm,
};

type ListActionButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-pressed"> & {
  active?: boolean;
  badge?: ReactNode;
  badgeLabel?: string;
  icon: ReactNode;
  size?: ListActionSize;
};

export function ListActionButton({
  active = false,
  badge,
  badgeLabel,
  children,
  className,
  icon,
  size = "md",
  ...props
}: ListActionButtonProps) {
  return (
    <button
      aria-pressed={active}
      className={cx(styles.listAction, sizeClass[size], className)}
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
