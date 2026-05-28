import { IconMessageCircle } from "@tabler/icons-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import type { MessageBubbleSize, MessageLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import styles from "./chat.module.css";

export function MessageTimeline({
  bubbleSize,
  children,
  layout,
  loadOlderLabel,
  loadingOlder,
  onLoadOlder,
}: {
  bubbleSize: MessageBubbleSize;
  children: ReactNode;
  layout: MessageLayout;
  loadOlderLabel?: string | null;
  loadingOlder?: boolean;
  onLoadOlder?: () => void;
}) {
  return (
    <div
      className={cx(
        styles.messageTimeline,
        layout === "continuous-feed" && styles.messageTimelineContinuous,
        bubbleSize === "compact" && styles.messageTimelineCompact,
      )}
    >
      {loadOlderLabel && onLoadOlder ? (
        <Button
          className={styles.loadOlderButton}
          disabled={loadingOlder}
          icon={<IconMessageCircle className={styles.icon} aria-hidden="true" />}
          onClick={onLoadOlder}
          size="sm"
        >
          {loadOlderLabel}
        </Button>
      ) : null}
      {children}
    </div>
  );
}
