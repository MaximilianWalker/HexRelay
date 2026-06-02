"use client";

import { useEffect, useRef, useState } from "react";
import {
  IconDots,
  IconMicrophone,
  IconMicrophoneOff,
  IconPhoneOff,
  IconScreenShare,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";

import type { NavLayout } from "@/lib/workspace-preferences";

import { WorkspaceProfileActionButton } from "./workspace-profile-action-button";
import { WorkspaceProfileMenu } from "./workspace-profile-menu";
import type { WorkspaceProfilePlacement } from "./workspace-profile-types";
import styles from "./workspace-profile-actions.module.css";

type WorkspaceProfileActionsProps = {
  collapsed: boolean;
  microphoneMuted: boolean;
  navLayout: NavLayout;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetMicrophoneMuted: (muted: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  onSetSoundMuted: (muted: boolean) => void;
  placement: WorkspaceProfilePlacement;
  soundMuted: boolean;
  voiceActionsAvailable?: boolean;
};

export function WorkspaceProfileActions({
  collapsed,
  microphoneMuted,
  navLayout,
  onOpenAudioDevices,
  onSetCollapsed,
  onSetMicrophoneMuted,
  onSetNavLayout,
  onSetSoundMuted,
  placement,
  soundMuted,
  voiceActionsAvailable = false,
}: WorkspaceProfileActionsProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!menuOpen) {
      return;
    }

    function closeFromOutside(event: globalThis.PointerEvent): void {
      const root = rootRef.current;
      if (root && event.target instanceof Node && root.contains(event.target)) {
        return;
      }

      setMenuOpen(false);
    }

    function closeFromKeyboard(event: globalThis.KeyboardEvent): void {
      if (event.key === "Escape") {
        setMenuOpen(false);
      }
    }

    document.addEventListener("pointerdown", closeFromOutside);
    document.addEventListener("keydown", closeFromKeyboard);

    return () => {
      document.removeEventListener("pointerdown", closeFromOutside);
      document.removeEventListener("keydown", closeFromKeyboard);
    };
  }, [menuOpen]);

  const MicrophoneIcon = microphoneMuted ? IconMicrophoneOff : IconMicrophone;
  const SoundIcon = soundMuted ? IconVolumeOff : IconVolume;

  return (
    <div
      className={styles.actions}
      data-collapsed={collapsed}
      data-placement={placement}
      ref={rootRef}
      role="group"
      aria-label="Profile actions"
    >
      <WorkspaceProfileActionButton
        active={microphoneMuted}
        className={styles.action}
        icon={MicrophoneIcon}
        label={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
        muted={microphoneMuted}
        onClick={() => onSetMicrophoneMuted(!microphoneMuted)}
        tone="mic"
      />
      <WorkspaceProfileActionButton
        active={soundMuted}
        className={styles.action}
        icon={SoundIcon}
        label={soundMuted ? "Unmute sound" : "Mute sound"}
        muted={soundMuted}
        onClick={() => onSetSoundMuted(!soundMuted)}
        tone="sound"
      />
      <WorkspaceProfileActionButton
        className={styles.action}
        disabled={!voiceActionsAvailable}
        icon={IconScreenShare}
        label="Start stream"
        tone="stream"
      />
      <WorkspaceProfileActionButton
        className={styles.action}
        disabled={!voiceActionsAvailable}
        icon={IconPhoneOff}
        label="Leave voice"
        tone="leave"
      />
      <WorkspaceProfileActionButton
        active={menuOpen}
        className={styles.action}
        controls={menuOpen ? "profile-more-menu" : undefined}
        expanded={menuOpen}
        hasPopup="dialog"
        icon={IconDots}
        label="More profile actions"
        onClick={() => setMenuOpen((open) => !open)}
        tone="more"
      />
      {menuOpen ? (
        <WorkspaceProfileMenu
          collapsed={collapsed}
          navLayout={navLayout}
          onClose={() => setMenuOpen(false)}
          onOpenAudioDevices={onOpenAudioDevices}
          onSetCollapsed={onSetCollapsed}
          onSetNavLayout={onSetNavLayout}
          placement={placement}
        />
      ) : null}
    </div>
  );
}
