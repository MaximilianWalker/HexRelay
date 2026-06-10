"use client";

import Link from "next/link";
import type {
  AnchorHTMLAttributes,
  AriaAttributes,
  ButtonHTMLAttributes,
  ComponentProps,
  HTMLAttributes,
  KeyboardEvent,
  ReactNode,
} from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

export type ListSize = "sm" | "md" | "lg";
export type ListTone = "neutral" | "danger";
export type ListIconColor = "default" | "accent" | "danger" | "muted";
type ListCurrent = AriaAttributes["aria-current"] | boolean;
type ListRootElement = "div" | "nav";

const rowSizeClass: Record<ListSize, string | undefined> = {
  lg: styles.listRowLg,
  md: undefined,
  sm: styles.listRowSm,
};

const primarySizeClass: Record<ListSize, string | undefined> = {
  lg: styles.listPrimaryLg,
  md: undefined,
  sm: styles.listPrimarySm,
};

const iconColorClass: Record<ListIconColor, string | undefined> = {
  accent: styles.listIconAccent,
  danger: styles.listIconDanger,
  default: undefined,
  muted: styles.listIconMuted,
};

type ListProps = HTMLAttributes<HTMLElement> & {
  as?: ListRootElement;
  keyboardNavigation?: boolean;
  panel?: boolean;
};

type ListContentProps = {
  active?: boolean;
  end?: ReactNode;
  icon?: ReactNode;
  iconColor?: ListIconColor;
  name?: ReactNode;
  size?: ListSize;
  tone?: ListTone;
};

type ListButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-current" | "children" | "name"> &
  ListContentProps & {
    children?: ReactNode;
    current?: ListCurrent;
    pressed?: boolean;
  };

type ListLinkProps = Omit<ComponentProps<typeof Link>, "aria-current" | "children" | "className"> &
  ListContentProps & {
    children?: ReactNode;
    className?: string;
    current?: ListCurrent;
    disabled?: boolean;
  };

type ListRowProps = Omit<HTMLAttributes<HTMLDivElement>, "children"> &
  ListContentProps & {
    children?: ReactNode;
  };

function getAriaCurrent(current: ListCurrent | undefined): AriaAttributes["aria-current"] | undefined {
  if (current === true) {
    return "page";
  }

  return current || undefined;
}

function renderListContent({
  children,
  icon,
  iconColor = "default",
  name,
}: {
  children?: ReactNode;
  icon?: ReactNode;
  iconColor?: ListIconColor;
  name?: ReactNode;
}) {
  const label = name ?? children;

  return (
    <>
      {icon ? (
        <span aria-hidden="true" className={cx(styles.listIcon, iconColorClass[iconColor])}>
          {icon}
        </span>
      ) : null}
      <span className={styles.listName}>{label}</span>
    </>
  );
}

function renderListEnd(end: ReactNode) {
  return end ? <span className={styles.listEnd}>{end}</span> : null;
}

export function List({
  as: Component = "div",
  children,
  className,
  keyboardNavigation = true,
  onKeyDown,
  panel = true,
  role,
  ...props
}: ListProps) {
  function handleKeyDown(event: KeyboardEvent<HTMLElement>) {
    onKeyDown?.(event);

    if (
      !keyboardNavigation ||
      event.defaultPrevented ||
      (event.key !== "ArrowDown" && event.key !== "ArrowUp")
    ) {
      return;
    }

    const items = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>(
        '[data-list-primary="true"]:not(:disabled):not([aria-disabled="true"])',
      ),
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

  const resolvedRole = role ?? (Component === "nav" ? undefined : "list");

  return (
    <Component
      className={cx(styles.list, className)}
      data-list-panel={panel ? "true" : "false"}
      onKeyDown={handleKeyDown}
      role={resolvedRole}
      {...props}
    >
      {children}
    </Component>
  );
}

export function ListButton({
  active = false,
  children,
  className,
  current,
  end,
  icon,
  iconColor,
  name,
  pressed,
  role,
  size = "md",
  tone = "neutral",
  ...props
}: ListButtonProps) {
  const checkedRole = role === "menuitemcheckbox" || role === "checkbox";

  return (
    <div
      className={cx(styles.listItem, rowSizeClass[size], tone === "danger" && styles.listItemDanger)}
      data-active={active ? "true" : undefined}
      data-disabled={props.disabled ? "true" : undefined}
    >
      <button
        {...props}
        aria-checked={checkedRole ? pressed : undefined}
        aria-current={getAriaCurrent(current)}
        aria-pressed={!checkedRole ? pressed : undefined}
        className={cx(
          styles.listPrimary,
          styles.listButton,
          primarySizeClass[size],
          tone === "danger" && styles.listPrimaryDanger,
          className,
        )}
        data-active={active ? "true" : undefined}
        data-list-primary="true"
        role={role}
        type={props.type ?? "button"}
      >
        {renderListContent({ children, icon, iconColor, name })}
      </button>
      {renderListEnd(end)}
    </div>
  );
}

export function ListLink({
  active = false,
  children,
  className,
  current,
  disabled,
  end,
  href,
  icon,
  iconColor,
  name,
  onClick,
  size = "md",
  tabIndex,
  tone = "neutral",
  ...props
}: ListLinkProps) {
  function handleClick(event: Parameters<NonNullable<AnchorHTMLAttributes<HTMLAnchorElement>["onClick"]>>[0]) {
    if (disabled) {
      event.preventDefault();
      return;
    }

    onClick?.(event);
  }

  return (
    <div
      className={cx(styles.listItem, rowSizeClass[size], tone === "danger" && styles.listItemDanger)}
      data-active={active ? "true" : undefined}
      data-disabled={disabled ? "true" : undefined}
    >
      <Link
        {...props}
        aria-current={getAriaCurrent(current)}
        aria-disabled={disabled || undefined}
        className={cx(
          styles.listPrimary,
          styles.listLink,
          primarySizeClass[size],
          tone === "danger" && styles.listPrimaryDanger,
          className,
        )}
        data-active={active ? "true" : undefined}
        data-list-primary="true"
        href={disabled ? "#" : href}
        onClick={handleClick}
        tabIndex={disabled ? -1 : tabIndex}
      >
        {renderListContent({ children, icon, iconColor, name })}
      </Link>
      {renderListEnd(end)}
    </div>
  );
}

export function ListRow({
  active = false,
  children,
  className,
  end,
  icon,
  iconColor,
  name,
  size = "md",
  tone = "neutral",
  ...props
}: ListRowProps) {
  return (
    <div
      className={cx(styles.listRow, rowSizeClass[size], tone === "danger" && styles.listItemDanger, className)}
      data-active={active ? "true" : undefined}
      {...props}
    >
      {renderListContent({ children, icon, iconColor, name })}
      {renderListEnd(end)}
    </div>
  );
}
