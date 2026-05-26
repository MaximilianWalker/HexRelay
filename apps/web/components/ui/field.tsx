import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  LabelHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

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

export function TextInput({ className, invalid, ...props }: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return <input className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}

export function TextArea({
  className,
  invalid,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { invalid?: boolean }) {
  return <textarea className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}

export function SelectField({
  className,
  invalid,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { invalid?: boolean }) {
  return <select className={cx(styles.fieldControl, invalid && styles.fieldControlInvalid, className)} {...props} />;
}

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

export function ToggleSwitch({
  checked,
  className,
  label,
  onChange,
  ...props
}: {
  checked: boolean;
  className?: string;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onChange">) {
  return (
    <button
      aria-checked={checked}
      aria-label={label}
      className={cx(styles.toggle, checked && styles.toggleOn, className)}
      onClick={() => onChange?.(!checked)}
      role="switch"
      type="button"
      {...props}
    >
      <span className={styles.toggleTrack}>
        <span className={styles.toggleThumb} />
      </span>
      <span>{checked ? "On" : "Off"}</span>
    </button>
  );
}
