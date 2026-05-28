import type { ButtonHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

type ToggleSwitchProps = {
  checked: boolean;
  className?: string;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onChange">;

export function ToggleSwitch({ checked, className, label, onChange, ...props }: ToggleSwitchProps) {
  return (
    <button
      aria-checked={checked}
      aria-label={label}
      className={cx(styles.toggle, checked && styles.toggleOn, className)}
      onClick={() => onChange?.(!checked)}
      role="switch"
      type="button"
      {...props}
    >
      <span aria-hidden="true" className={styles.toggleTrack}>
        <span className={styles.toggleThumb} />
      </span>
    </button>
  );
}
