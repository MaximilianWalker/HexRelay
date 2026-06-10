import type { ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import { useToggleButton } from "../behavior";
import styles from "./styles.module.css";

export type Option<T extends string> = {
  disabled?: boolean;
  icon?: ReactNode;
  id: T;
  label: string;
};
export type Size = "sm" | "md" | "lg";

type RootProps<T extends string> = {
  label: string;
  onChange: (value: T) => void;
  options: Option<T>[];
  size?: Size;
  value: T;
};

type ToggleGroupButtonProps<T extends string> = {
  active: boolean;
  onChange: (value: T) => void;
  option: Option<T>;
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

export function Root<T extends string>({ label, onChange, options, size = "md", value }: RootProps<T>) {
  return (
    <div
      aria-label={label}
      className={cx(styles.buttonGroup, size === "sm" && styles.buttonGroupSm, size === "lg" && styles.buttonGroupLg)}
      role="group"
    >
      {options.map((option) => (
        <ToggleGroupButton active={option.id === value} key={option.id} onChange={onChange} option={option} />
      ))}
    </div>
  );
}
