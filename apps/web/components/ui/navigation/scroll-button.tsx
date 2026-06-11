import { IconChevronLeft, IconChevronRight } from "@tabler/icons-react";
import type { ButtonHTMLAttributes } from "react";

import styles from "./scroll-button.module.css";

type Direction = "next" | "previous";
type Appearance = "framed" | "plain";

type ScrollButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label" | "children" | "className"> & {
  appearance?: Appearance;
  direction: Direction;
  label: string;
};

export function ScrollButton({
  appearance = "framed",
  direction,
  label,
  type = "button",
  ...props
}: ScrollButtonProps) {
  const Icon = direction === "previous" ? IconChevronLeft : IconChevronRight;

  return (
    <button
      {...props}
      aria-label={label}
      className={styles.scrollButton}
      data-scroll-button-appearance={appearance}
      type={type}
    >
      <Icon aria-hidden="true" className={styles.scrollIcon} />
    </button>
  );
}
