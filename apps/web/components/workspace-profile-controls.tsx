"use client";

import { useEffect, useRef, useState } from "react";
import type { ComponentType } from "react";
import {
  IconArrowsExchange,
  IconChevronRight,
  IconDots,
  IconFocusCentered,
  IconMicrophone,
  IconMicrophoneOff,
  IconPhoneOff,
  IconScreenShare,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";

import type { NavLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import { Avatar } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import styles from "./workspace-profile-controls.module.css";

export type WorkspaceProfile = {
  active: boolean;
  initials: string;
  name: string;
  status: string;
};

type ProfilePlacement = "sidebar" | "topbar";

type WorkspaceProfileControlsProps = {
  collapsed: boolean;
  microphoneMuted: boolean;
  navLayout: NavLayout;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  onSetMicrophoneMuted: (muted: boolean) => void;
  onSetSoundMuted: (muted: boolean) => void;
  placement: ProfilePlacement;
  profile: WorkspaceProfile;
  soundMuted: boolean;
};

type ActionButtonProps = {
  active?: boolean;
  className?: string;
  controls?: string;
  expanded?: boolean;
  hasPopup?: "dialog" | "menu";
  icon: ComponentType<{ "aria-hidden"?: boolean; className?: string }>;
  label: string;
  onClick?: () => void;
};

const NAV_LAYOUT_OPTIONS: Array<{ label: string; value: NavLayout }> = [
  { label: "Sidebar", value: "sidebar" },
  { label: "Topbar", value: "topbar" },
];

function ActionButton({
  active,
  className,
  controls,
  expanded,
  hasPopup,
  icon: Icon,
  label,
  onClick,
}: ActionButtonProps) {
  return (
    <Button
      aria-controls={controls}
      aria-expanded={expanded}
      aria-haspopup={hasPopup}
      aria-label={label}
      className={cx(styles.actionButton, active && styles.actionButtonActive, className)}
      onClick={onClick}
      pressed={active}
      size="icon"
      title={label}
    >
      <Icon className={styles.actionIcon} aria-hidden={true} />
    </Button>
  );
}

function ProfileCard({ collapsed, profile }: { collapsed: boolean; profile: WorkspaceProfile }) {
  return (
    <div className={styles.profile} title={profile.name}>
      <div className={styles.avatarFrame}>
        <Avatar className={styles.avatar} kind="user" size="sm" text={profile.initials} />
        <span className={cx(styles.presence, profile.active && styles.presenceActive)} aria-hidden="true" />
      </div>
      {!collapsed ? (
        <div className={styles.details}>
          <p className={styles.eyebrow}>{profile.active ? "Active profile" : "Profile setup"}</p>
          <p className={styles.name}>{profile.name}</p>
          <p className={styles.status}>
            <span className={cx(styles.statusDot, profile.active && styles.statusDotActive)} aria-hidden="true" />
            {profile.status}
          </p>
        </div>
      ) : null}
    </div>
  );
}

type ProfileMenuProps = {
  collapsed: boolean;
  navLayout: NavLayout;
  onClose: () => void;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
};

function ProfileMenu({
  collapsed,
  navLayout,
  onClose,
  onOpenAudioDevices,
  onSetCollapsed,
  onSetNavLayout,
}: ProfileMenuProps) {
  function selectNavLayout(nextLayout: NavLayout): void {
    onSetNavLayout(nextLayout);
    onClose();
  }

  return (
    <div aria-label="Profile actions menu" className={styles.menu} id="profile-more-menu" role="dialog">
      <button
        aria-pressed={collapsed}
        className={styles.menuItem}
        onClick={() => onSetCollapsed(!collapsed)}
        type="button"
      >
        <IconFocusCentered className={styles.menuIcon} aria-hidden="true" />
        <span>Compact mode</span>
        <span className={cx(styles.switch, collapsed && styles.switchOn)} aria-hidden="true">
          <span />
        </span>
      </button>

      <div className={styles.layoutItem} role="group" aria-label="Navigation layout">
        <IconArrowsExchange className={styles.menuIcon} aria-hidden="true" />
        <span>Navigation</span>
        <div className={styles.layoutChoices}>
          {NAV_LAYOUT_OPTIONS.map((option) => (
            <button
              aria-pressed={navLayout === option.value}
              className={cx(styles.layoutChoice, navLayout === option.value && styles.layoutChoiceActive)}
              key={option.value}
              onClick={() => selectNavLayout(option.value)}
              type="button"
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      <button
        className={styles.menuItem}
        onClick={() => {
          onOpenAudioDevices();
          onClose();
        }}
        type="button"
      >
        <IconMicrophone className={styles.menuIcon} aria-hidden="true" />
        <span>Audio devices</span>
        <IconChevronRight className={styles.menuChevron} aria-hidden="true" />
      </button>
    </div>
  );
}

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

  const SoundIcon = soundMuted ? IconVolumeOff : IconVolume;
  const MicrophoneIcon = microphoneMuted ? IconMicrophoneOff : IconMicrophone;

  return (
    <div className={styles.root} data-collapsed={collapsed} data-placement={placement} ref={rootRef}>
      <ProfileCard collapsed={collapsed} profile={profile} />
      <div className={styles.actions} role="group" aria-label="Profile actions">
        <ActionButton
          active={microphoneMuted}
          className={cx(styles.actionMic, microphoneMuted && styles.actionMuted)}
          icon={MicrophoneIcon}
          label={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
          onClick={() => onSetMicrophoneMuted(!microphoneMuted)}
        />
        <ActionButton
          active={soundMuted}
          className={cx(styles.actionSound, soundMuted && styles.actionMuted)}
          icon={SoundIcon}
          label={soundMuted ? "Unmute sound" : "Mute sound"}
          onClick={() => onSetSoundMuted(!soundMuted)}
        />
        <ActionButton className={styles.actionStream} icon={IconScreenShare} label="Start stream" />
        <ActionButton className={styles.actionLeave} icon={IconPhoneOff} label="Leave voice" />
        <ActionButton
          active={menuOpen}
          className={styles.actionMore}
          controls={menuOpen ? "profile-more-menu" : undefined}
          expanded={menuOpen}
          hasPopup="dialog"
          icon={IconDots}
          label="More profile actions"
          onClick={() => setMenuOpen((open) => !open)}
        />
        {menuOpen ? (
          <ProfileMenu
            collapsed={collapsed}
            navLayout={navLayout}
            onClose={() => setMenuOpen(false)}
            onOpenAudioDevices={onOpenAudioDevices}
            onSetCollapsed={onSetCollapsed}
            onSetNavLayout={onSetNavLayout}
          />
        ) : null}
      </div>
    </div>
  );
}
