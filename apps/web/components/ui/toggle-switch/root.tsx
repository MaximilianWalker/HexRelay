import type { ButtonHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type RootProps = {
  checked: boolean;
  className?: string;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onChange">;

export function Root({ checked, className, label, onChange, ...props }: RootProps) {
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
