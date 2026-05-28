import type { SelectHTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

export function SelectField({
  className,
  invalid,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { invalid?: boolean }) {
  return <select className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}
