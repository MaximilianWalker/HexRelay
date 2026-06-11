import { IconMessageCircle } from "@tabler/icons-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/buttons/button";
import type { MessageBubbleSize, MessageLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

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
          align="center"
          disabled={loadingOlder}
          icon={<IconMessageCircle aria-hidden="true" />}
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
