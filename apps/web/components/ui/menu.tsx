import type { ButtonHTMLAttributes, CSSProperties, HTMLAttributes, KeyboardEvent, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

type MenuPosition = "absolute" | "fixed" | "static";
type MenuTone = "neutral" | "danger";

export function Menu({
  children,
  className,
  onKeyDown,
  position = "fixed",
  role = "menu",
  style,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  position?: MenuPosition;
  style?: CSSProperties;
}) {
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
      data-position={position}
      onKeyDown={handleKeyDown}
      role={role}
      style={style}
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
  tone = "neutral",
  trailing,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  icon?: ReactNode;
  pressed?: boolean;
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
      className={cx(styles.menuItem, tone === "danger" && styles.menuItemDanger, className)}
      role={nativeButtonRole ? undefined : itemRole}
      type="button"
    >
      {icon ? <span className={styles.menuIcon}>{icon}</span> : null}
      <span className={styles.menuItemLabel}>{children}</span>
      {trailing ? <span className={styles.menuItemTrailing}>{trailing}</span> : null}
    </button>
  );
}
