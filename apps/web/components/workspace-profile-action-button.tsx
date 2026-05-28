import type { ComponentType } from "react";

import { Button } from "@/components/ui/button";
import { cx } from "@/lib/ui/cx";

import styles from "./workspace-profile-action-button.module.css";

type WorkspaceProfileActionTone = "mic" | "sound" | "stream" | "leave" | "more";

type WorkspaceProfileActionButtonProps = {
  active?: boolean;
  className?: string;
  controls?: string;
  expanded?: boolean;
  hasPopup?: "dialog" | "menu";
  icon: ComponentType<{ "aria-hidden"?: boolean; className?: string }>;
  label: string;
  muted?: boolean;
  onClick?: () => void;
  tone: WorkspaceProfileActionTone;
};

const toneClass: Record<WorkspaceProfileActionTone, string> = {
  leave: styles.toneLeave,
  mic: styles.toneMic,
  more: styles.toneMore,
  sound: styles.toneSound,
  stream: styles.toneStream,
};

export function WorkspaceProfileActionButton({
  active,
  className,
  controls,
  expanded,
  hasPopup,
  icon: Icon,
  label,
  muted,
  onClick,
  tone,
}: WorkspaceProfileActionButtonProps) {
  return (
    <Button
      aria-controls={controls}
      aria-expanded={expanded}
      aria-haspopup={hasPopup}
      aria-label={label}
      className={cx(styles.button, toneClass[tone], active && styles.active, muted && styles.muted, className)}
      onClick={onClick}
      pressed={active}
      size="icon"
      title={label}
    >
      <Icon className={styles.icon} aria-hidden={true} />
    </Button>
  );
}
