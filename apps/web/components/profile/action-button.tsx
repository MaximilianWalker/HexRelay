import type { ComponentType } from "react";

import { Button } from "@/components/ui/button";
import { cx } from "@/lib/ui/cx";

import styles from "./action-button.module.css";

type Tone = "mic" | "sound" | "stream" | "leave" | "more";

type ActionButtonProps = {
  active?: boolean;
  className?: string;
  controls?: string;
  disabled?: boolean;
  expanded?: boolean;
  hasPopup?: "dialog" | "menu";
  icon: ComponentType<{ "aria-hidden"?: boolean; className?: string }>;
  label: string;
  muted?: boolean;
  onClick?: () => void;
  tone: Tone;
};

const toneClass: Record<Tone, string> = {
  leave: styles.toneLeave,
  mic: styles.toneMic,
  more: styles.toneMore,
  sound: styles.toneSound,
  stream: styles.toneStream,
};

export function ActionButton({
  active,
  className,
  controls,
  disabled,
  expanded,
  hasPopup,
  icon: Icon,
  label,
  muted,
  onClick,
  tone,
}: ActionButtonProps) {
  return (
    <Button
      aria-controls={controls}
      aria-expanded={expanded}
      aria-haspopup={hasPopup}
      aria-label={label}
      className={cx(styles.button, toneClass[tone], active && styles.active, muted && styles.muted, className)}
      disabled={disabled}
      onClick={onClick}
      pressed={active}
      size="icon"
      title={label}
    >
      <Icon className={styles.icon} aria-hidden={true} />
    </Button>
  );
}
