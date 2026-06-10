import Link from "next/link";
import type { ComponentProps, MouseEvent, ReactNode } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export type Variant = "primary" | "secondary" | "ghost" | "danger";
export type Size = "sm" | "md" | "lg";
export type IconSize = "sm" | "md" | "lg";
export type IconPosition = "start" | "end";
export type Shape = "default" | "icon";
export type Align = "start" | "center" | "end" | "stretch";
export type Tone = "neutral" | "accent" | "success" | "danger" | "muted";
export type PressedTone = "accent" | "danger";

const variantClass: Record<Variant, string> = {
  primary: styles.buttonPrimary,
  secondary: styles.buttonSecondary,
  ghost: styles.buttonGhost,
  danger: styles.buttonDanger,
};

type StyleProps = {
  icon?: ReactNode;
  iconPosition?: IconPosition;
  iconSize?: IconSize;
  loading?: boolean;
  pressed?: boolean;
  pressedTone?: PressedTone;
  shape?: Shape;
  size?: Size;
  tone?: Tone;
  variant?: Variant;
  align?: Align;
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
}: StyleProps & {
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

function renderContent(children: ReactNode, icon: ReactNode, iconPosition: IconPosition) {
  return (
    <>
      {icon && iconPosition === "start" ? icon : null}
      {children}
      {icon && iconPosition === "end" ? icon : null}
    </>
  );
}

export function Root({
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
}: ButtonHTMLAttributes<HTMLButtonElement> & StyleProps) {
  return (
    <button
      {...props}
      aria-busy={loading || undefined}
      aria-pressed={pressed}
      className={buttonClassName({ align, className, iconSize, pressed, pressedTone, shape, size, tone, variant })}
      type={type}
    >
      {renderContent(children, icon, iconPosition)}
    </button>
  );
}

type LinkButtonProps = Omit<ComponentProps<typeof Link>, "className" | "children"> &
  StyleProps & {
    children: ReactNode;
    className?: string;
    disabled?: boolean;
  };

export function LinkButton({
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
}: LinkButtonProps) {
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
      {renderContent(children, icon, iconPosition)}
    </Link>
  );
}
