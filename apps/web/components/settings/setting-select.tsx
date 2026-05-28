import type { ReactNode, SelectHTMLAttributes } from "react";

import { SelectField } from "@/components/ui/field";
import { cx } from "@/lib/ui/cx";

import styles from "./settings-ui.module.css";

type SettingSelectProps = SelectHTMLAttributes<HTMLSelectElement> & {
  children: ReactNode;
};

export function SettingSelect({
  children,
  className,
  ...props
}: SettingSelectProps) {
  return (
    <SelectField className={cx(styles.settingSelect, className)} {...props}>
      {children}
    </SelectField>
  );
}
