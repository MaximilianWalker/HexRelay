import Link from "next/link";
import type { ComponentProps, MouseEvent, ReactNode } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./control.module.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";
export type ButtonIconSize = "sm" | "md" | "lg";
export type ButtonIconPosition = "start" | "end";
export type ButtonShape = "default" | "icon";
export type ButtonAlign = "start" | "center" | "end" | "stretch";
export type ButtonTone = "neutral" | "accent" | "success" | "danger" | "muted";
export type ButtonPressedTone = "accent" | "danger";

const variantClass: Record<ButtonVariant, string> = {
  primary: styles.buttonPrimary,
  secondary: styles.buttonSecondary,
  ghost: styles.buttonGhost,
  danger: styles.buttonDanger,
};

type ButtonStyleProps = {
  icon?: ReactNode;
  iconPosition?: ButtonIconPosition;
  iconSize?: ButtonIconSize;
  loading?: boolean;
  pressed?: boolean;
  pressedTone?: ButtonPressedTone;
  shape?: ButtonShape;
  size?: ButtonSize;
  tone?: ButtonTone;
  variant?: ButtonVariant;
  align?: ButtonAlign;
};

function buttonClassName({
  align,
  className,
  iconSize,
  pressed,
  pressedTone = "accent",
  shape = "default",
  size = "md",
  tone = "neutral",
  variant = "secondary",
}: ButtonStyleProps & {
  className?: string;
}) {
  return cx(
    styles.button,
    variantClass[variant],
    size === "sm" && styles.buttonSm,
    size === "lg" && styles.buttonLg,
    shape === "icon" && styles.buttonIcon,
    iconSize === "sm" && styles.buttonIconSizeSm,
    iconSize === "md" && styles.buttonIconSizeMd,
    iconSize === "lg" && styles.buttonIconSizeLg,
    align === "start" && styles.alignStart,
    align === "center" && styles.alignCenter,
    align === "end" && styles.alignEnd,
    align === "stretch" && styles.alignStretch,
    tone === "accent" && styles.buttonToneAccent,
    tone === "success" && styles.buttonToneSuccess,
    tone === "danger" && styles.buttonToneDanger,
    tone === "muted" && styles.buttonToneMuted,
    pressed && styles.buttonPressed,
    pressed && pressedTone === "danger" && styles.buttonPressedDanger,
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
  align,
  children,
  className,
  icon,
  iconPosition = "start",
  iconSize,
  loading,
  pressed,
  pressedTone,
  shape = "default",
  size = "md",
  tone,
  type = "button",
  variant = "secondary",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & ButtonStyleProps) {
  return (
    <button
      {...props}
      aria-busy={loading || undefined}
      aria-pressed={pressed}
      className={buttonClassName({ align, className, iconSize, pressed, pressedTone, shape, size, tone, variant })}
      type={type}
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
  align,
  children,
  className,
  disabled,
  href,
  icon,
  iconPosition = "start",
  iconSize,
  loading,
  onClick,
  pressed,
  pressedTone,
  shape,
  size,
  tabIndex,
  tone,
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
      {...props}
      aria-busy={loading || undefined}
      aria-disabled={disabled || undefined}
      aria-pressed={pressed}
      className={buttonClassName({ align, className, iconSize, pressed, pressedTone, shape, size, tone, variant })}
      data-disabled={disabled ? "true" : undefined}
      href={disabled ? "#" : href}
      onClick={handleClick}
      tabIndex={disabled ? -1 : tabIndex}
    >
      {renderButtonContent(children, icon, iconPosition)}
    </Link>
  );
}
