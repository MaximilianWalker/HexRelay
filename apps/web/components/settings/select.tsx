import type { ReactNode, SelectHTMLAttributes } from "react";

import { SelectField } from "@/components/ui/forms/select-field";
import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type SelectProps = SelectHTMLAttributes<HTMLSelectElement> & {
  children: ReactNode;
};

export function Select({
  children,
  className,
  ...props
}: SelectProps) {
  return (
    <SelectField className={cx(styles.settingSelect, className)} {...props}>
      {children}
    </SelectField>
  );
}
