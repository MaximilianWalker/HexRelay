import type { ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

export type SegmentedControlOption<T extends string> = {
  icon?: ReactNode;
  id: T;
  label: string;
};

export function SegmentedControl<T extends string>({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  onChange: (value: T) => void;
  options: SegmentedControlOption<T>[];
  value: T;
}) {
  return (
    <div aria-label={label} className={styles.segmentedControl} role="group">
      {options.map((option) => (
        <button
          aria-pressed={option.id === value}
          className={cx(styles.segmentedButton, option.id === value && styles.segmentedButtonActive)}
          key={option.id}
          onClick={() => onChange(option.id)}
          type="button"
        >
          {option.icon}
          {option.label}
        </button>
      ))}
    </div>
  );
}
