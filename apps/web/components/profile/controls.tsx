"use client";

import type { NavLayout } from "@/lib/workspace-preferences";

import { Actions } from "./actions";
import { Card } from "./card";
import type { Placement, Profile } from "./types";
import styles from "./controls.module.css";

type ControlsProps = {
  collapsed: boolean;
  microphoneMuted: boolean;
  navLayout: NavLayout;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  onSetMicrophoneMuted: (muted: boolean) => void;
  onSetSoundMuted: (muted: boolean) => void;
  placement: Placement;
  profile: Profile;
  soundMuted: boolean;
  voiceActionsAvailable?: boolean;
};

export function Controls({
  collapsed,
  microphoneMuted,
  navLayout,
  onOpenAudioDevices,
  onSetCollapsed,
  onSetMicrophoneMuted,
  onSetNavLayout,
  onSetSoundMuted,
  placement,
  profile,
  soundMuted,
  voiceActionsAvailable = false,
}: ControlsProps) {
  return (
    <div className={styles.root} data-profile-collapsed={collapsed} data-profile-placement={placement}>
      <Card collapsed={collapsed} profile={profile} />
      <Actions
        collapsed={collapsed}
        microphoneMuted={microphoneMuted}
        navLayout={navLayout}
        onOpenAudioDevices={onOpenAudioDevices}
        onSetCollapsed={onSetCollapsed}
        onSetMicrophoneMuted={onSetMicrophoneMuted}
        onSetNavLayout={onSetNavLayout}
        onSetSoundMuted={onSetSoundMuted}
        placement={placement}
        soundMuted={soundMuted}
        voiceActionsAvailable={voiceActionsAvailable}
      />
    </div>
  );
}
