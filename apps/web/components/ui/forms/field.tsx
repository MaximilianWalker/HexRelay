import type { LabelHTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./field.module.css";

export function Field({
  children,
  className,
  error,
  helper,
  label,
  ...props
}: LabelHTMLAttributes<HTMLLabelElement> & {
  error?: string;
  helper?: ReactNode;
  label: ReactNode;
}) {
  return (
    <label className={cx(styles.field, className)} {...props}>
      <span className={styles.fieldLabel}>{label}</span>
      {children}
      {error ? <span className={cx(styles.fieldHelper, styles.fieldError)}>{error}</span> : null}
      {helper && !error ? <span className={styles.fieldHelper}>{helper}</span> : null}
    </label>
  );
}
