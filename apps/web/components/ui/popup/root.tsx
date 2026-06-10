import type { CSSProperties, HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export type Placement =
  | "bottom-center"
  | "bottom-end"
  | "bottom-start"
  | "center"
  | "left-center"
  | "left-end"
  | "left-start"
  | "right-center"
  | "right-end"
  | "right-start"
  | "top-center"
  | "top-end"
  | "top-start";
type PopupPosition = "absolute" | "fixed" | "static";

export function Root({
  children,
  className,
  placement,
  position = "absolute",
  style,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  children: ReactNode;
  placement?: Placement;
  position?: PopupPosition;
  style?: CSSProperties;
}) {
  return (
    <div
      className={cx(styles.popup, className)}
      data-placement={placement}
      data-position={position}
      style={style}
      {...props}
    >
      {children}
    </div>
  );
}
