import type { TextareaHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

export function TextArea({
  className,
  invalid,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { invalid?: boolean }) {
  return <textarea className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}
