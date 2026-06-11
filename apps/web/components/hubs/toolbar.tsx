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

import { ToggleGroup } from "@/components/ui/toggles/toggle-group";
import { TextInput } from "@/components/ui/forms/text-input";
import { ToggleButton } from "@/components/ui/toggles/toggle-button";
import { Toolbar as UiToolbar } from "@/components/ui/surfaces/toolbar";
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
          icon={<IconPinned aria-hidden="true" />}
          onPressedChange={() => onPinnedChange()}
          pressed={pinnedOnly}
        >
          Pinned
        </ToggleButton>
        <ToggleButton
          icon={<IconMessageCircle aria-hidden="true" />}
          onPressedChange={() => onUnreadChange()}
          pressed={unreadOnly}
        >
          Unread
        </ToggleButton>
        <ToggleButton
          icon={<IconVolumeOff aria-hidden="true" />}
          onPressedChange={() => onMutedChange()}
          pressed={mutedOnly}
        >
          Muted
        </ToggleButton>
        <ToggleGroup
          label="View mode"
          onChange={onLayoutChange}
          options={[
            { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
            { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
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
