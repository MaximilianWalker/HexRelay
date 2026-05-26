import type { HTMLAttributes } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./ui.module.css";

type AvatarKind = "user" | "server";
type AvatarSize = "sm" | "md" | "lg";

export function Avatar({
  className,
  kind = "user",
  label,
  size = "md",
  text,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  kind?: AvatarKind;
  label?: string;
  size?: AvatarSize;
  text: string;
}) {
  return (
    <div
      aria-label={label}
      className={cx(
        styles.avatar,
        kind === "user" ? styles.avatarUser : styles.avatarServer,
        size === "sm" && styles.avatarSm,
        size === "lg" && styles.avatarLg,
        className,
      )}
      role={label ? "img" : undefined}
      {...props}
    >
      {text}
    </div>
  );
}
