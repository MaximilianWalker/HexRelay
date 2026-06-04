import type { ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import { useToggleButton } from "./behavior";
import styles from "./control.module.css";

export type ToggleGroupOption<T extends string> = {
  disabled?: boolean;
  icon?: ReactNode;
  id: T;
  label: string;
};

type ToggleGroupProps<T extends string> = {
  label: string;
  onChange: (value: T) => void;
  options: ToggleGroupOption<T>[];
  value: T;
};

type ToggleGroupButtonProps<T extends string> = {
  active: boolean;
  onChange: (value: T) => void;
  option: ToggleGroupOption<T>;
};

function ToggleGroupButton<T extends string>({ active, onChange, option }: ToggleGroupButtonProps<T>) {
  const buttonProps = useToggleButton({
    disabled: option.disabled,
    onPressedChange: () => onChange(option.id),
    pressed: active,
  });

  return (
    <button
      className={cx(styles.buttonGroupButton, active && styles.buttonGroupButtonActive)}
      type="button"
      {...buttonProps}
    >
      {option.icon}
      {option.label}
    </button>
  );
}

export function ToggleGroup<T extends string>({ label, onChange, options, value }: ToggleGroupProps<T>) {
  return (
    <div aria-label={label} className={styles.buttonGroup} role="group">
      {options.map((option) => (
        <ToggleGroupButton active={option.id === value} key={option.id} onChange={onChange} option={option} />
      ))}
    </div>
  );
}
