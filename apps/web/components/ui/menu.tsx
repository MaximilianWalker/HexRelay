import type { ButtonHTMLAttributes, HTMLAttributes, KeyboardEvent, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

type MenuSize = "sm" | "md" | "lg";
type MenuTone = "neutral" | "danger";

const itemSizeClass: Record<MenuSize, string | undefined> = {
  lg: styles.menuItemLg,
  md: undefined,
  sm: styles.menuItemSm,
};

const rowSizeClass: Record<MenuSize, string | undefined> = {
  lg: styles.menuRowLg,
  md: undefined,
  sm: styles.menuRowSm,
};

export function Menu({
  children,
  className,
  onKeyDown,
  role = "menu",
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    onKeyDown?.(event);

    if (event.defaultPrevented || (event.key !== "ArrowDown" && event.key !== "ArrowUp")) {
      return;
    }

    const items = Array.from(
      event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled), [role="menuitem"]:not([aria-disabled="true"])'),
    );
    if (items.length === 0) {
      return;
    }

    const activeIndex = items.findIndex((item) => item === document.activeElement);
    const direction = event.key === "ArrowDown" ? 1 : -1;
    const nextIndex = activeIndex === -1 ? 0 : (activeIndex + direction + items.length) % items.length;

    event.preventDefault();
    items[nextIndex]?.focus();
  }

  return (
    <div
      className={cx(styles.menu, className)}
      onKeyDown={handleKeyDown}
      role={role}
      {...props}
    >
      {children}
    </div>
  );
}

export function MenuItem({
  children,
  className,
  icon,
  pressed,
  role,
  size = "md",
  tone = "neutral",
  trailing,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  icon?: ReactNode;
  pressed?: boolean;
  size?: MenuSize;
  tone?: MenuTone;
  trailing?: ReactNode;
}) {
  const itemRole = role ?? (pressed === undefined ? "menuitem" : "menuitemcheckbox");
  const nativeButtonRole = itemRole === "button";

  return (
    <button
      {...props}
      aria-checked={!nativeButtonRole ? pressed : undefined}
      aria-pressed={nativeButtonRole ? pressed : undefined}
      className={cx(styles.menuItem, itemSizeClass[size], tone === "danger" && styles.menuItemDanger, className)}
      role={nativeButtonRole ? undefined : itemRole}
      type="button"
    >
      {icon ? <span className={styles.menuIcon}>{icon}</span> : null}
      <span className={styles.menuItemLabel}>{children}</span>
      {trailing ? <span className={styles.menuItemTrailing}>{trailing}</span> : null}
    </button>
  );
}

export function MenuRow({
  children,
  className,
  icon,
  size = "md",
  trailing,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  icon?: ReactNode;
  size?: MenuSize;
  trailing?: ReactNode;
}) {
  return (
    <div className={cx(styles.menuRow, rowSizeClass[size], className)} {...props}>
      {icon ? <span className={styles.menuIcon}>{icon}</span> : null}
      <span className={styles.menuItemLabel}>{children}</span>
      {trailing ? <span className={styles.menuItemTrailing}>{trailing}</span> : null}
    </div>
  );
}
