import { BrandLogo } from "@/components/brand-logo";
import { cx } from "@/lib/ui/cx";

import styles from "./brand-lockup.module.css";

type BrandLockupSize = "md" | "lg";

export function BrandLockup({
  className,
  collapsed = false,
  name = "HexRelay",
  size = "md",
}: {
  className?: string;
  collapsed?: boolean;
  name?: string;
  size?: BrandLockupSize;
}) {
  return (
    <div
      className={cx(styles.root, className)}
      data-collapsed={collapsed ? "true" : undefined}
      data-size={size}
      aria-label={name}
    >
      <BrandLogo className={styles.mark} aria-hidden="true" focusable="false" />
      <span className={styles.text}>
        <span className={styles.name}>{name}</span>
      </span>
    </div>
  );
}
