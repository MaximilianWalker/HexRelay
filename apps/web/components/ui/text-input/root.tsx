import type { InputHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "../field/styles.module.css";

export function Root({
  "aria-invalid": ariaInvalid,
  className,
  invalid,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return (
    <input
      aria-invalid={ariaInvalid ?? (invalid ? true : undefined)}
      className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)}
      data-invalid={invalid ? "true" : undefined}
      {...props}
    />
  );
}
