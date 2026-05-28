import type { InputHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

export function TextInput({
  className,
  invalid,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return <input className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}
