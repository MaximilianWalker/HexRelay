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

import { Button } from "@/components/ui/button";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { TextInput } from "@/components/ui/field";
import { Toolbar } from "@/components/ui/toolbar";
import type { HubLayout } from "@/lib/hub-state";

import styles from "./hubs.module.css";

export function HubToolbar({
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
      <Toolbar actions={actions}>
        <Button icon={<IconPinned className={styles.icon} aria-hidden="true" />} onClick={onPinnedChange} pressed={pinnedOnly}>
          Pinned
        </Button>
        <Button icon={<IconMessageCircle className={styles.icon} aria-hidden="true" />} onClick={onUnreadChange} pressed={unreadOnly}>
          Unread
        </Button>
        <Button icon={<IconVolumeOff className={styles.icon} aria-hidden="true" />} onClick={onMutedChange} pressed={mutedOnly}>
          Muted
        </Button>
        <SegmentedControl
          label="View mode"
          onChange={onLayoutChange}
          options={[
            { id: "list", label: "List", icon: <IconList className={styles.icon} aria-hidden="true" /> },
            { id: "cards", label: "Cards", icon: <IconLayoutGrid className={styles.icon} aria-hidden="true" /> },
          ]}
          value={layout}
        />
      </Toolbar>
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
