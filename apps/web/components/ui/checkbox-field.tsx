import type { InputHTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export function CheckboxField({
  children,
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { children: ReactNode }) {
  return (
    <label className={cx(styles.checkboxField, className)}>
      <input type="checkbox" {...props} />
      <span>{children}</span>
    </label>
  );
}
