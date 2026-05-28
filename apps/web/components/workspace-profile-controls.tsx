"use client";

import type { NavLayout } from "@/lib/workspace-preferences";

import { WorkspaceProfileActions } from "./workspace-profile-actions";
import { WorkspaceProfileCard } from "./workspace-profile-card";
import type { WorkspaceProfile, WorkspaceProfilePlacement } from "./workspace-profile-types";
import styles from "./workspace-profile-controls.module.css";

type WorkspaceProfileControlsProps = {
  collapsed: boolean;
  microphoneMuted: boolean;
  navLayout: NavLayout;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  onSetMicrophoneMuted: (muted: boolean) => void;
  onSetSoundMuted: (muted: boolean) => void;
  placement: WorkspaceProfilePlacement;
  profile: WorkspaceProfile;
  soundMuted: boolean;
};

export function WorkspaceProfileControls({
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
}: WorkspaceProfileControlsProps) {
  return (
    <div className={styles.root} data-profile-collapsed={collapsed} data-profile-placement={placement}>
      <WorkspaceProfileCard collapsed={collapsed} profile={profile} />
      <WorkspaceProfileActions
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
      />
    </div>
  );
}
