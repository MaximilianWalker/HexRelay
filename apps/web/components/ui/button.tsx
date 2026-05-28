import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md" | "icon";

const variantClass: Record<ButtonVariant, string> = {
  primary: styles.buttonPrimary,
  secondary: styles.buttonSecondary,
  ghost: styles.buttonGhost,
  danger: styles.buttonDanger,
};

export function Button({
  children,
  className,
  icon,
  loading,
  pressed,
  size = "md",
  variant = "secondary",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  icon?: ReactNode;
  loading?: boolean;
  pressed?: boolean;
  size?: ButtonSize;
  variant?: ButtonVariant;
}) {
  return (
    <button
      aria-busy={loading || undefined}
      aria-pressed={pressed}
      className={cx(
        styles.button,
        variantClass[variant],
        size === "sm" && styles.buttonSm,
        size === "icon" && styles.buttonIcon,
        pressed && styles.buttonPressed,
        className,
      )}
      type="button"
      {...props}
    >
      {icon}
      {children}
    </button>
  );
}
