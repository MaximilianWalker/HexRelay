import type { ComponentType } from "react";

import type { ButtonTone } from "@/components/ui/button";
import { IconButton } from "@/components/ui/icon-button";

type Tone = "mic" | "sound" | "stream" | "leave" | "more";

type ActionButtonProps = {
  active?: boolean;
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

const toneValue: Record<Tone, ButtonTone> = {
  leave: "danger",
  mic: "success",
  more: "muted",
  sound: "accent",
  stream: "accent",
};

export function ActionButton({
  active,
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
  const dangerPressed = muted || tone === "leave";

  return (
    <IconButton
      aria-controls={controls}
      aria-expanded={expanded}
      aria-haspopup={hasPopup}
      align="stretch"
      data-profile-action="true"
      disabled={disabled}
      iconSize="lg"
      label={label}
      onClick={onClick}
      pressed={active}
      pressedTone={dangerPressed ? "danger" : "accent"}
      title={label}
      tone={muted ? "danger" : toneValue[tone]}
    >
      <Icon aria-hidden={true} />
    </IconButton>
  );
}
