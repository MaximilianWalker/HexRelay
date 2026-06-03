"use client";

import type { ReactNode } from "react";
import {
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconPinned,
  IconSearch,
  IconVolumeOff,
} from "@tabler/icons-react";

import { SegmentedControl } from "@/components/ui/segmented-control";
import { TextInput } from "@/components/ui/text-input";
import { ToggleButton } from "@/components/ui/toggle-button";
import { Toolbar as UiToolbar } from "@/components/ui/toolbar";
import type { HubLayout } from "@/lib/hub-state";

import styles from "./styles.module.css";

export function Toolbar({
  actions,
  layout,
  mutedOnly,
  onLayoutChange,
  onMutedChange,
  onPinnedChange,
  onSearchChange,
  onUnreadChange,
  pinnedOnly,
  search,
  searchLabel,
  unreadOnly,
}: {
  actions?: ReactNode;
  layout: HubLayout;
  mutedOnly: boolean;
  onLayoutChange: (layout: HubLayout) => void;
  onMutedChange: () => void;
  onPinnedChange: () => void;
  onSearchChange: (value: string) => void;
  onUnreadChange: () => void;
  pinnedOnly: boolean;
  search: string;
  searchLabel: string;
  unreadOnly: boolean;
}) {
  return (
    <div className={styles.hubToolbarStack}>
      <UiToolbar actions={actions}>
        <ToggleButton
          icon={<IconPinned className={styles.icon} aria-hidden="true" />}
          onPressedChange={() => onPinnedChange()}
          pressed={pinnedOnly}
        >
          Pinned
        </ToggleButton>
        <ToggleButton
          icon={<IconMessageCircle className={styles.icon} aria-hidden="true" />}
          onPressedChange={() => onUnreadChange()}
          pressed={unreadOnly}
        >
          Unread
        </ToggleButton>
        <ToggleButton
          icon={<IconVolumeOff className={styles.icon} aria-hidden="true" />}
          onPressedChange={() => onMutedChange()}
          pressed={mutedOnly}
        >
          Muted
        </ToggleButton>
        <SegmentedControl
          label="View mode"
          onChange={onLayoutChange}
          options={[
            { id: "list", label: "List", icon: <IconList className={styles.icon} aria-hidden="true" /> },
            { id: "cards", label: "Cards", icon: <IconLayoutGrid className={styles.icon} aria-hidden="true" /> },
          ]}
          value={layout}
        />
      </UiToolbar>
      <div className={styles.searchWrap}>
        <IconSearch className={styles.searchIcon} aria-hidden="true" />
        <TextInput
          aria-label={searchLabel}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder={searchLabel}
          value={search}
        />
      </div>
    </div>
  );
}
