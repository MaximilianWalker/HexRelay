import type { TextareaHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

export function TextArea({
  "aria-invalid": ariaInvalid,
  className,
  invalid,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { invalid?: boolean }) {
  return (
    <textarea
      aria-invalid={ariaInvalid ?? (invalid ? true : undefined)}
      className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)}
      data-invalid={invalid ? "true" : undefined}
      {...props}
    />
  );
}
