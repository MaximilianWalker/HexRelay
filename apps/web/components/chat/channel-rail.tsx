import type { HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./chat.module.css";

export function ChannelRail({
  children,
  className,
  title,
  ...props
}: HTMLAttributes<HTMLElement> & {
  title: string;
}) {
  return (
    <aside className={cx(styles.channelRail, className)} {...props}>
      <div className={styles.channelRailHeader}>
        <h3>{title}</h3>
      </div>
      <div className={styles.channelStack}>{children}</div>
    </aside>
  );
}

export function ChannelStack({ children }: { children: ReactNode }) {
  return <div className={styles.channelStack}>{children}</div>;
}
