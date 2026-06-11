import type { ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import { useToggleButton } from "../buttons/behavior";
import styles from "./toggle-group.module.css";

export type ToggleGroupOption<T extends string> = {
  disabled?: boolean;
  icon?: ReactNode;
  id: T;
  label: string;
};
export type ToggleGroupSize = "sm" | "md" | "lg";

type RootProps<T extends string> = {
  label: string;
  onChange: (value: T) => void;
  options: ToggleGroupOption<T>[];
  size?: ToggleGroupSize;
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
      className={cx(styles.toggleGroupButton, active && styles.toggleGroupButtonActive)}
      type="button"
      {...buttonProps}
    >
      {option.icon}
      {option.label}
    </button>
  );
}

export function ToggleGroup<T extends string>({ label, onChange, options, size = "md", value }: RootProps<T>) {
  return (
    <div
      aria-label={label}
      className={cx(styles.toggleGroup, size === "sm" && styles.toggleGroupSm, size === "lg" && styles.toggleGroupLg)}
      role="group"
    >
      {options.map((option) => (
        <ToggleGroupButton active={option.id === value} key={option.id} onChange={onChange} option={option} />
      ))}
    </div>
  );
}
