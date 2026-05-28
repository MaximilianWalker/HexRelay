import { IconPinned, IconPinnedOff, IconX } from "@tabler/icons-react";

import type { WorkspaceTab } from "@/lib/workspace-tabs";

import styles from "./workspace-context-menu.module.css";

type WorkspaceContextMenuProps = {
  onCloseTab: (tab: WorkspaceTab) => void;
  onTogglePinned: (tab: WorkspaceTab) => void;
  position: { x: number; y: number };
  tab: WorkspaceTab;
};

export function WorkspaceContextMenu({ onCloseTab, onTogglePinned, position, tab }: WorkspaceContextMenuProps) {
  return (
    <div
      className={styles.menu}
      onClick={(event) => event.stopPropagation()}
      role="menu"
      style={{ left: position.x, top: position.y }}
    >
      <button className={styles.item} onClick={() => onTogglePinned(tab)} role="menuitem" type="button">
        {tab.pinned ? (
          <IconPinnedOff className={styles.icon} aria-hidden="true" />
        ) : (
          <IconPinned className={styles.icon} aria-hidden="true" />
        )}
        {tab.pinned ? "Unpin tab" : "Pin tab"}
      </button>
      <button className={`${styles.item} ${styles.danger}`} onClick={() => onCloseTab(tab)} role="menuitem" type="button">
        <IconX className={styles.icon} aria-hidden="true" />
        Close tab
      </button>
    </div>
  );
}
