import type { SelectHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "../field/styles.module.css";

export function Root({
  "aria-invalid": ariaInvalid,
  className,
  invalid,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { invalid?: boolean }) {
  return (
    <select
      aria-invalid={ariaInvalid ?? (invalid ? true : undefined)}
      className={cx(styles.fieldControl, styles.selectControl, invalid && styles.fieldControlInvalid, className)}
      data-invalid={invalid ? "true" : undefined}
      {...props}
    />
  );
}
