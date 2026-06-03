import Link from "next/link";
import type { ComponentProps, MouseEvent, ReactNode } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "icon";
export type ButtonIconPosition = "start" | "end";

const variantClass: Record<ButtonVariant, string> = {
  primary: styles.buttonPrimary,
  secondary: styles.buttonSecondary,
  ghost: styles.buttonGhost,
  danger: styles.buttonDanger,
};

type ButtonStyleProps = {
  icon?: ReactNode;
  iconPosition?: ButtonIconPosition;
  loading?: boolean;
  pressed?: boolean;
  size?: ButtonSize;
  variant?: ButtonVariant;
};

function buttonClassName({
  className,
  pressed,
  size = "md",
  variant = "secondary",
}: ButtonStyleProps & {
  className?: string;
}) {
  return cx(
    styles.button,
    variantClass[variant],
    size === "sm" && styles.buttonSm,
    size === "icon" && styles.buttonIcon,
    pressed && styles.buttonPressed,
    className,
  );
}

function renderButtonContent(children: ReactNode, icon: ReactNode, iconPosition: ButtonIconPosition) {
  return (
    <>
      {icon && iconPosition === "start" ? icon : null}
      {children}
      {icon && iconPosition === "end" ? icon : null}
    </>
  );
}

export function Button({
  children,
  className,
  icon,
  iconPosition = "start",
  loading,
  pressed,
  size = "md",
  variant = "secondary",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & ButtonStyleProps) {
  return (
    <button
      aria-busy={loading || undefined}
      aria-pressed={pressed}
      className={buttonClassName({ className, pressed, size, variant })}
      type="button"
      {...props}
    >
      {renderButtonContent(children, icon, iconPosition)}
    </button>
  );
}

type ButtonLinkProps = Omit<ComponentProps<typeof Link>, "className" | "children"> &
  ButtonStyleProps & {
    children: ReactNode;
    className?: string;
    disabled?: boolean;
  };

export function ButtonLink({
  children,
  className,
  disabled,
  icon,
  iconPosition = "start",
  loading,
  onClick,
  pressed,
  size,
  tabIndex,
  variant,
  ...props
}: ButtonLinkProps) {
  function handleClick(event: MouseEvent<HTMLAnchorElement>) {
    if (disabled) {
      event.preventDefault();
      return;
    }

    onClick?.(event);
  }

  return (
    <Link
      aria-busy={loading || undefined}
      aria-disabled={disabled || undefined}
      aria-pressed={pressed}
      className={buttonClassName({ className, pressed, size, variant })}
      data-disabled={disabled ? "true" : undefined}
      onClick={handleClick}
      tabIndex={disabled ? -1 : tabIndex}
      {...props}
    >
      {renderButtonContent(children, icon, iconPosition)}
    </Link>
  );
}
