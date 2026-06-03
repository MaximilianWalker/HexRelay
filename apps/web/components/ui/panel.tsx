import type { HTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type PanelVariant = "surface" | "raised" | "danger";
type PanelPadding = "none" | "sm" | "md" | "lg";

export function Panel({
  className,
  padding = "md",
  variant = "surface",
  ...props
}: HTMLAttributes<HTMLElement> & {
  padding?: PanelPadding;
  variant?: PanelVariant;
}) {
  return (
    <section
      className={cx(
        styles.panel,
        variant === "raised" && styles.panelRaised,
        variant === "danger" && styles.panelDanger,
        padding === "sm" && styles.panelPaddingSm,
        padding === "md" && styles.panelPaddingMd,
        padding === "lg" && styles.panelPaddingLg,
        className,
      )}
      {...props}
    />
  );
}
